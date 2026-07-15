use std::time::Duration;

use sunbeam_test::OtelCollector;

/// Encode a minimal OTLP `ExportTraceServiceRequest` holding a single span by
/// hand (protobuf wire format), so the test doesn't need an OTLP client.
fn encode_probe_request(name: &str) -> Vec<u8> {
    fn field(tag: u8, data: &[u8]) -> Vec<u8> {
        let mut v = vec![tag, data.len() as u8];
        v.extend_from_slice(data);
        v
    }

    let trace_id = [1u8; 16];
    let span_id = [2u8; 8];

    // Span: trace_id = 1, span_id = 2, name = 3.
    let mut span = field(0x0A, &trace_id);
    span.extend(field(0x12, &span_id));
    span.extend(field(0x1A, name.as_bytes()));
    // ScopeSpans: spans = 2.
    let scope_spans = field(0x12, &span);
    // ResourceSpans: scope_spans = 2.
    let resource_spans = field(0x12, &scope_spans);
    // ExportTraceServiceRequest: resource_spans = 1.
    field(0x0A, &resource_spans)
}

#[tokio::test]
async fn otelcol_receives_spans_over_otlp_http() {
    let container = OtelCollector::default()
        .publish_ports()
        .start()
        .await
        .expect("otel collector should start");

    let endpoint = OtelCollector::endpoint(&container)
        .await
        .expect("collector endpoint should resolve");
    let addr = endpoint
        .trim_start_matches("http://")
        .to_string();

    // The published port can take a moment to become reachable (e.g. lima's
    // host-side forwarder); poll until TCP connects succeed.
    let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
    loop {
        match tokio::net::TcpStream::connect(&addr).await {
            Ok(_) => break,
            Err(_) if tokio::time::Instant::now() < deadline => {
                tokio::time::sleep(Duration::from_millis(250)).await;
            }
            Err(e) => panic!("collector port {addr} should be reachable: {e}"),
        }
    }

    let span_name = "sunbeam-test-probe";
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{endpoint}/v1/traces"))
        .header("content-type", "application/x-protobuf")
        .body(encode_probe_request(span_name))
        .send()
        .await
        .expect("OTLP request should send");
    assert!(
        resp.status().is_success(),
        "collector should accept the span, got {}",
        resp.status()
    );

    // The debug exporter (verbosity: detailed) dumps received spans to stderr.
    let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
    loop {
        let logs = container
            .stderr_to_vec()
            .await
            .expect("collector logs should be readable");
        let logs = String::from_utf8_lossy(&logs);
        if logs.contains(span_name) {
            break;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "collector did not log the probe span; logs:\n{logs}"
        );
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}
