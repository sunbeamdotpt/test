#[tokio::test]
async fn openfga_is_healthy() {
    use sunbeam_test::OpenFga;

    let container = OpenFga::default()
        .publish_ports()
        .start()
        .await
        .expect("openfga should start");

    let url = format!(
        "{}/healthz",
        OpenFga::url(&container)
            .await
            .expect("openfga url should resolve")
    );
    let response = reqwest::get(&url)
        .await
        .expect("health request should succeed");
    assert!(
        response.status().is_success(),
        "openfga /healthz should return 2xx, got {}",
        response.status()
    );
}
