use crate::{
    core::{client::connect, server::accept},
    utp::tests::utp::mock_utp_pairs,
};

#[tokio::test]
async fn test_bi_handshake() {
    let (a, b) = mock_utp_pairs();

    tokio::spawn(async move {
        accept(b.into()).await.unwrap();
    });

    connect(a.into()).await.unwrap();
}
