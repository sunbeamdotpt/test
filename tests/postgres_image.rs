use std::time::Duration;

use sunbeam_test::Postgres;

#[tokio::test]
async fn postgres_publishes_port_and_accepts_connections() {
    let container = Postgres::default()
        .publish_port()
        .start()
        .await
        .expect("postgres should start");

    let url = Postgres::url(&container)
        .await
        .expect("postgres url should resolve");

    // The URL host is the Docker host; try to open a TCP connection to prove the
    // published port is reachable.
    let addr = url
        .trim_start_matches("postgres://ory:ory@")
        .trim_start_matches("postgresql://ory:ory@")
        .split('/')
        .next()
        .expect("url should contain host:port");

    let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
    loop {
        match tokio::net::TcpStream::connect(addr).await {
            Ok(_) => break,
            Err(_) if tokio::time::Instant::now() < deadline => {
                tokio::time::sleep(Duration::from_millis(250)).await;
            }
            Err(e) => panic!("postgres port {addr} should be reachable: {e}"),
        }
    }
}
