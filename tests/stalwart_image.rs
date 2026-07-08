use std::time::Duration;

#[tokio::test]
async fn stalwart_is_healthy() {
    use sunbeam_test::Stalwart;

    let container = Stalwart::default()
        .publish_ports()
        .start()
        .await
        .expect("stalwart should start");

    let base_url = Stalwart::url(&container)
        .await
        .expect("stalwart url should resolve");

    let url = format!("{base_url}/login");
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
