#[tokio::test]
async fn kratos_is_healthy() {
    use sunbeam_test::Kratos;

    let container = Kratos::default()
        .publish_ports()
        .start()
        .await
        .expect("kratos should start");

    let url = format!(
        "{}/health/ready",
        Kratos::public_url(&container)
            .await
            .expect("kratos url should resolve")
    );
    let response = reqwest::get(&url)
        .await
        .expect("health request should succeed");
    assert!(
        response.status().is_success(),
        "kratos /health/ready should return 2xx, got {}",
        response.status()
    );
}

#[tokio::test]
async fn hydra_is_healthy() {
    use sunbeam_test::Hydra;

    let container = Hydra::default()
        .publish_ports()
        .start()
        .await
        .expect("hydra should start");

    let url = format!(
        "{}/health/ready",
        Hydra::admin_url(&container)
            .await
            .expect("hydra url should resolve")
    );
    let response = reqwest::get(&url)
        .await
        .expect("health request should succeed");
    assert!(
        response.status().is_success(),
        "hydra /health/ready should return 2xx, got {}",
        response.status()
    );
}

#[tokio::test]
async fn keto_is_healthy() {
    use sunbeam_test::Keto;

    let container = Keto::default()
        .publish_ports()
        .start()
        .await
        .expect("keto should start");

    let url = format!(
        "{}/health/ready",
        Keto::read_url(&container)
            .await
            .expect("keto url should resolve")
    );
    let response = reqwest::get(&url)
        .await
        .expect("health request should succeed");
    assert!(
        response.status().is_success(),
        "keto /health/ready should return 2xx, got {}",
        response.status()
    );
}
