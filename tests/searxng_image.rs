#[tokio::test]
async fn searxng_is_healthy() {
    use sunbeam_test::SearXng;

    let container = SearXng::default()
        .publish_ports()
        .start()
        .await
        .expect("searxng should start");

    let base_url = SearXng::url(&container)
        .await
        .expect("searxng url should resolve");

    let url = format!("{base_url}/healthz");
    let response = reqwest::get(&url)
        .await
        .expect("health request should succeed");
    assert!(
        response.status().is_success(),
        "searxng /healthz should return 2xx, got {}",
        response.status()
    );
}
