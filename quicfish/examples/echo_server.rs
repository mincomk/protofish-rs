use quicfish::QuicUTP;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();
    // 1. Generate a self-signed certificate for the example
    let (cert_pem, key_pem) = generate_self_signed_cert()?;

    // Save the certificate so the client can trust it
    std::fs::write("cert.pem", &cert_pem)?;
    println!("Generated cert.pem");

    // 2. Start the server
    // The easy function requires BufReads for cert and key
    let mut cert_reader = std::io::Cursor::new(cert_pem);
    let mut key_reader = std::io::Cursor::new(key_pem);

    let addr: SocketAddr = "127.0.0.1:4433".parse()?;
    let endpoint =
        quicfish::create_server_endpoint(addr, &mut cert_reader, &mut key_reader).await?;

    println!("Server listening on {}", endpoint.local_addr()?);

    // 3. Accept loop
    while let Some(conn) = endpoint.accept().await {
        println!("New connection accepted");

        tokio::spawn(async move {
            if let Err(e) = handle_connection(conn).await {
                eprintln!("Connection error: {}", e);
            }
        });
    }

    Ok(())
}

async fn handle_connection(quinn_conn: quinn::Connection) -> anyhow::Result<()> {
    // Upgrade the raw QUIC connection to a Protofish connection
    // true indicates this is the server side
    let utp = QuicUTP::new(quinn_conn, true);
    let pf_conn = protofish::accept(utp.into()).await?;
    println!("Protofish connection established");

    // Accept new arbitrary contexts (streams)
    while let Some(arb) = pf_conn.next_arb().await {
        tokio::spawn(async move {
            println!("New arbitrary context accepted");
            let i_bytes = arb.read().await.unwrap();
            println!("Received initial bytes on arb: {:?}", i_bytes);
            arb.write(i_bytes).await.unwrap();

            while let Ok(stream) = arb.wait_stream().await {
                println!("New stream opened");
                let (mut writer, mut reader) = stream.split();

                // Simple echo: read from reader, write to writer
                // In a real app you'd probably use copy, but let's be explicit
                let mut buf = vec![0u8; 1024];
                loop {
                    match reader.read(&mut buf).await {
                        Ok(0) => break, // EOF
                        Ok(n) => {
                            if let Err(e) = writer.write_all(&buf[..n]).await {
                                eprintln!("Write error: {}", e);
                                break;
                            }
                        }
                        Err(e) => {
                            eprintln!("Read error: {}", e);
                            break;
                        }
                    }
                }
                println!("Stream finished");
            }
        });
    }

    Ok(())
}

fn generate_self_signed_cert() -> anyhow::Result<(String, String)> {
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()])?;
    let cert_pem = cert.cert.pem();
    let key_pem = cert.key_pair.serialize_pem();
    Ok((cert_pem, key_pem))
}
