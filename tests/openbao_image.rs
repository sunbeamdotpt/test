#[tokio::test]
async fn openbao_is_healthy() {
    use sunbeam_test::{container_bridge_ip, OpenBao};

    let container = OpenBao::default()
        .start()
        .await
        .expect("openbao should start");

    let host = container_bridge_ip(container.id())
        .await
        .expect("bridge ip should resolve");

    let url = format!("http://{host}:{}/v1/sys/health", OpenBao::PORT);
    let response = reqwest::get(&url)
        .await
        .expect("health request should succeed");
    assert!(
        response.status().is_success(),
        "openbao /v1/sys/health should return 2xx, got {}",
        response.status()
    );
}
