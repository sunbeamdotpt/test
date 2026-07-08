use std::time::Duration;

#[tokio::test]
async fn tuwunel_is_healthy() {
    use sunbeam_test::Tuwunel;

    let container = Tuwunel::default()
        .publish_ports()
        .start()
        .await
        .expect("tuwunel should start");

    let base_url = Tuwunel::url(&container)
        .await
        .expect("tuwunel url should resolve");

    let url = format!("{base_url}/_matrix/client/versions");
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
