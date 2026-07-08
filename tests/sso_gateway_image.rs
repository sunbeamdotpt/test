use sunbeam_test::SsoGateway;

#[tokio::test]
#[ignore = "requires a pre-built sso-gateway image (see SsoGateway::DEFAULT_IMAGE_NAME)"]
async fn sso_gateway_stack_exposes_ready_endpoint() {
    let gateway = SsoGateway::new()
        .start()
        .await
        .expect("sso-gateway stack should start");

    let endpoint = gateway.endpoint();

    let resp = reqwest::get(format!("{endpoint}/.well-known/openid-configuration"))
        .await
        .expect("gateway should accept request");

    assert!(
        resp.status().is_success(),
        "gateway OIDC discovery should be successful: {}",
        resp.status()
    );
}
