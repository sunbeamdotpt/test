#[tokio::test]
async fn openbao_is_healthy() {
    use sunbeam_test::OpenBao;

    let container = OpenBao::default()
        .publish_ports()
        .start()
        .await
        .expect("openbao should start");

    let base_url = OpenBao::url(&container)
        .await
        .expect("openbao url should resolve");

    let url = format!("{base_url}/v1/sys/health");
    let response = reqwest::get(&url)
        .await
        .expect("health request should succeed");
    assert!(
        response.status().is_success(),
        "openbao /v1/sys/health should return 2xx, got {}",
        response.status()
    );
}
