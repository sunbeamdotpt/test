use std::time::Duration;

#[tokio::test]
async fn opensearch_is_healthy() {
    use sunbeam_test::{container_bridge_ip, OpenSearch};

    let container = OpenSearch::default()
        .start()
        .await
        .expect("opensearch should start");

    let host = container_bridge_ip(container.id())
        .await
        .expect("bridge ip should resolve");

    let url = format!("http://{host}:{}/_cluster/health", OpenSearch::REST_PORT);

    let mut last_status = None;
    for _ in 0..60 {
        match reqwest::get(&url).await {
            Ok(resp) if resp.status().is_success() => {
                let body: serde_json::Value =
                    resp.json().await.expect("health body should be json");
                let status = body["status"].as_str().unwrap_or("unknown");
                if status == "green" || status == "yellow" {
                    return;
                }
                last_status = Some(status.to_string());
            }
            other => {
                last_status = other.map(|r| r.status().to_string()).ok();
            }
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }

    panic!(
        "opensearch did not become healthy in time, last status: {:?}",
        last_status
    );
}
