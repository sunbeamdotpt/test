use std::time::Duration;

#[tokio::test]
async fn tuwunel_is_healthy() {
    use sunbeam_test::{container_bridge_ip, Tuwunel};

    let container = Tuwunel::default()
        .start()
        .await
        .expect("tuwunel should start");

    let host = container_bridge_ip(container.id())
        .await
        .expect("bridge ip should resolve");

    let url = format!("http://{host}:{}/_matrix/client/versions", Tuwunel::PORT);
    let mut last_status = None;
    for _ in 0..30 {
        match reqwest::get(&url).await {
            Ok(resp) if resp.status().is_success() => {
                let body: serde_json::Value =
                    resp.json().await.expect("versions body should be json");
                assert!(
                    body.get("versions").is_some(),
                    "tuwunel /_matrix/client/versions should contain versions"
                );
                return;
            }
            other => last_status = other.map(|r| r.status().to_string()).ok(),
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }

    panic!(
        "tuwunel did not become healthy in time, last status: {:?}",
        last_status
    );
}
