use std::time::Duration;

#[tokio::test]
async fn headscale_is_healthy() {
    use sunbeam_test::{container_bridge_ip, Headscale};

    let container = Headscale::default()
        .start()
        .await
        .expect("headscale should start");

    let host = container_bridge_ip(container.id())
        .await
        .expect("bridge ip should resolve");

    let url = format!("http://{host}:{}/health", Headscale::HTTP_PORT);
    let mut last_status = None;
    for _ in 0..30 {
        match reqwest::get(&url).await {
            Ok(resp) if resp.status().is_success() => {
                last_status = Some(resp.status().to_string());
                break;
            }
            other => last_status = other.map(|r| r.status().to_string()).ok(),
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }

    let status = last_status.expect("headscale /health should respond");
    assert!(
        status.starts_with('2'),
        "headscale /health should return 2xx, got {status}"
    );
}
