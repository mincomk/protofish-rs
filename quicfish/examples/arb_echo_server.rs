use quicfish::QuicUTP;
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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
        println!("New arbitrary context accepted");
        tokio::spawn(async move {
            while let Ok(v) = arb.read().await {
                println!("Received on arb: {:?}", v);
                if let Err(e) = arb.write(v).await {
                    eprintln!("Error writing to arb: {}", e);
                    break;
                }
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
