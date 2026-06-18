#[tokio::test]
async fn searxng_is_healthy() {
    use sunbeam_test::{container_bridge_ip, SearXng};

    let container = SearXng::default()
        .start()
        .await
        .expect("searxng should start");

    let host = container_bridge_ip(container.id())
        .await
        .expect("bridge ip should resolve");

    let url = format!("http://{host}:{}/healthz", SearXng::PORT);
    let response = reqwest::get(&url)
        .await
        .expect("health request should succeed");
    assert!(
        response.status().is_success(),
        "searxng /healthz should return 2xx, got {}",
        response.status()
    );
}
