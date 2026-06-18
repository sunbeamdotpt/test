#[tokio::test]
async fn kratos_is_healthy() {
    use sunbeam_test::{container_bridge_ip, Kratos};

    let container = Kratos::default()
        .start()
        .await
        .expect("kratos should start");

    let host = container_bridge_ip(container.id())
        .await
        .expect("bridge ip should resolve");

    let url = format!("http://{host}:{}/health/ready", Kratos::PUBLIC_PORT);
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
    use sunbeam_test::{container_bridge_ip, Hydra};

    let container = Hydra::default()
        .start()
        .await
        .expect("hydra should start");

    let host = container_bridge_ip(container.id())
        .await
        .expect("bridge ip should resolve");

    let url = format!("http://{host}:{}/health/ready", Hydra::ADMIN_PORT);
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
    use sunbeam_test::{container_bridge_ip, Keto};

    let container = Keto::default()
        .start()
        .await
        .expect("keto should start");

    let host = container_bridge_ip(container.id())
        .await
        .expect("bridge ip should resolve");

    let url = format!("http://{host}:{}/health/ready", Keto::READ_PORT);
    let response = reqwest::get(&url)
        .await
        .expect("health request should succeed");
    assert!(
        response.status().is_success(),
        "keto /health/ready should return 2xx, got {}",
        response.status()
    );
}
