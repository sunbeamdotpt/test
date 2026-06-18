use std::time::Duration;

#[tokio::test]
async fn stalwart_is_healthy() {
    use sunbeam_test::{container_bridge_ip, Stalwart};

    let container = Stalwart::default()
        .start()
        .await
        .expect("stalwart should start");

    let host = container_bridge_ip(container.id())
        .await
        .expect("bridge ip should resolve");

    let url = format!("http://{host}:{}/login", Stalwart::PORT);
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

    let status = last_status.expect("stalwart /login should respond");
    assert!(
        status.starts_with('2'),
        "stalwart /login should return 2xx, got {status}"
    );

    // Also verify we can extract the bootstrap admin password from the logs.
    let password = Stalwart::admin_password(&container)
        .await
        .expect("should parse admin password");
    assert!(!password.is_empty(), "admin password should not be empty");
}
