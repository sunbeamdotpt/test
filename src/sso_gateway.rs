use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use testcontainers::{
    core::ContainerPort,
    runners::AsyncRunner,
    ContainerAsync, GenericImage, ImageExt,
};
use tokio::time::{sleep, Instant};

use crate::{Hydra, Keto, Kratos, Postgres};

/// Kratos configuration tailored for sso-gateway tests.
///
/// The identity schema includes the extra traits the gateway expects (`userName`, `name`,
/// `active`, `tenant_id`) and the registration flow creates a session automatically.
const KRATOS_CONFIG: &str = r#"version: v0.13.0
identity:
  default_schema_id: default
  schemas:
    - id: default
      url: base64://eyIkaWQiOiJodHRwczovL3NjaGVtYXMub3J5LnNoL3ByZXNldHMva3JhdG9zL3F1aWNrc3RhcnQvZW1haWwtcGFzc3dvcmQvaWRlbnRpdHkuc2NoZW1hLmpzb24iLCIkc2NoZW1hIjoiaHR0cDovL2pzb24tc2NoZW1hLm9yZy9kcmFmdC0wNy9zY2hlbWEjIiwidGl0bGUiOiJQZXJzb24iLCJ0eXBlIjoib2JqZWN0IiwicHJvcGVydGllcyI6eyJ0cmFpdHMiOnsidHlwZSI6Im9iamVjdCIsInByb3BlcnRpZXMiOnsiZW1haWwiOnsidHlwZSI6InN0cmluZyIsImZvcm1hdCI6ImVtYWlsIiwidGl0bGUiOiJFLU1haWwiLCJvcnkuc2gva3JhdG9zIjp7ImNyZWRlbnRpYWxzIjp7InBhc3N3b3JkIjp7ImlkZW50aWZpZXIiOnRydWV9fSwicmVjb3ZlcnkiOnsidmlhIjoiZW1haWwifSwidmVyaWZpY2F0aW9uIjp7InZpYSI6ImVtYWlsIn19fSwidXNlck5hbWUiOnsidHlwZSI6InN0cmluZyJ9LCJuYW1lIjp7InR5cGUiOiJvYmplY3QifSwiYWN0aXZlIjp7InR5cGUiOiJib29sZWFuIn0sInRlbmFudF9pZCI6eyJ0eXBlIjoic3RyaW5nIn19LCJyZXF1aXJlZCI6WyJlbWFpbCJdLCJhZGRpdGlvbmFsUHJvcGVydGllcyI6ZmFsc2V9fX0=
serve:
  public:
    base_url: http://localhost:4433/
  admin:
    base_url: http://localhost:4434/
selfservice:
  default_browser_return_url: http://localhost:4433/
  methods:
    password:
      enabled: true
      config:
        haveibeenpwned_enabled: false
  flows:
    registration:
      after:
        password:
          hooks:
            - hook: session
"#;

/// Keto configuration tailored for sso-gateway tests.
const KETO_CONFIG: &str = r#"dsn: memory

namespaces:
  - id: 0
    name: app
  - id: 1
    name: scim_group

serve:
  read:
    host: 0.0.0.0
    port: 4466
  write:
    host: 0.0.0.0
    port: 4467
  metrics:
    host: 0.0.0.0
    port: 4468
"#;

/// Valid ULID used for the system tenant inside the gateway container.
const SYSTEM_TENANT_ULID: &str = "01HZY9JTKKHK3Y6XJJYHZ9Q5TV";

fn unique_prefix() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("sso{nanos:x}")
}

/// Testcontainers orchestrator for the full sso-gateway stack.
///
/// Starts Postgres, Hydra, Kratos, and Keto on a private Docker network, then runs a
/// pre-built sso-gateway container against them. The only thing exposed to callers is
/// the gateway's public endpoint URL.
#[derive(Debug, Clone)]
pub struct SsoGateway {
    image_name: String,
    image_tag: String,
    postgres_tag: String,
    hydra_tag: String,
    kratos_tag: String,
    keto_tag: String,
    extra_env: HashMap<String, String>,
}

impl SsoGateway {
    pub const DEFAULT_IMAGE_NAME: &'static str = "ghcr.io/sunbeamdotpt/sso-gateway";
    pub const DEFAULT_IMAGE_TAG: &'static str = "latest";

    /// Gateway HTTP port inside the container.
    pub const PORT: u16 = 8080;

    /// Create a new orchestrator with the default pre-built gateway image.
    pub fn new() -> Self {
        Self::default()
    }

    /// Override the gateway image name and tag.
    pub fn with_image(mut self, name: impl Into<String>, tag: impl Into<String>) -> Self {
        self.image_name = name.into();
        self.image_tag = tag.into();
        self
    }

    /// Override the Postgres image tag.
    pub fn with_postgres_tag(mut self, tag: impl Into<String>) -> Self {
        self.postgres_tag = tag.into();
        self
    }

    /// Override the Hydra image tag.
    pub fn with_hydra_tag(mut self, tag: impl Into<String>) -> Self {
        self.hydra_tag = tag.into();
        self
    }

    /// Override the Kratos image tag.
    pub fn with_kratos_tag(mut self, tag: impl Into<String>) -> Self {
        self.kratos_tag = tag.into();
        self
    }

    /// Override the Keto image tag.
    pub fn with_keto_tag(mut self, tag: impl Into<String>) -> Self {
        self.keto_tag = tag.into();
        self
    }

    /// Inject an extra environment variable into the sso-gateway container.
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.extra_env.insert(key.into(), value.into());
        self
    }

    /// Start the full stack and return a handle that exposes only the gateway endpoint.
    pub async fn start(
        self,
    ) -> Result<SsoGatewayHandle, testcontainers::TestcontainersError> {
        let prefix = unique_prefix();
        let network = format!("{prefix}-net");

        let postgres_name = format!("{prefix}-postgres");
        let hydra_name = format!("{prefix}-hydra");
        let kratos_name = format!("{prefix}-kratos");
        let keto_name = format!("{prefix}-keto");
        let gateway_name = format!("{prefix}-gateway");

        // Secrets are derived from the prefix so parallel stacks do not share them and
        // so they are never the literal development values denied by the gateway.
        let state_cookie_secret = format!("sunbeam-test-{prefix}-secret-key-at-least-32-bytes");
        let bootstrap_client_secret =
            format!("sunbeam-test-{prefix}-bootstrap-secret-key");

        // Start the backing services sequentially so the shared network is created before
        // the gateway container joins it.
        let postgres = Postgres::new()
            .with_tag(&self.postgres_tag)
            .with_network(&network)
            .with_container_name(&postgres_name)
            .start()
            .await?;

        let _kratos = Kratos::new()
            .with_tag(&self.kratos_tag)
            .with_network(&network)
            .with_container_name(&kratos_name)
            .with_config(KRATOS_CONFIG)
            .start()
            .await?;

        let _keto = Keto::new()
            .with_tag(&self.keto_tag)
            .with_network(&network)
            .with_container_name(&keto_name)
            .with_config(KETO_CONFIG)
            .start()
            .await?;

        let hydra_issuer = format!("http://{hydra_name}:4444");
        let _hydra = Hydra::new()
            .with_tag(&self.hydra_tag)
            .with_network(&network)
            .with_container_name(&hydra_name)
            .with_urls_self_issuer(&hydra_issuer)
            .start()
            .await?;

        // Give the backing services a moment to finish opening their HTTP ports before
        // the gateway starts connecting to them.
        sleep(Duration::from_secs(2)).await;

        let database_url = format!(
            "postgres://ory:ory@{postgres_name}:5432/ory?sslmode=disable"
        );

        let mut image = GenericImage::new(&self.image_name, &self.image_tag)
            .with_exposed_port(ContainerPort::Tcp(Self::PORT))
            .with_mapped_port(0, ContainerPort::Tcp(Self::PORT))
            .with_container_name(&gateway_name)
            .with_network(&network)
            .with_env_var("BIND_ADDR", "0.0.0.0:8080")
            .with_env_var("DATABASE_URL", database_url)
            .with_env_var("DATABASE_SSL_REQUIRED", "false")
            .with_env_var("HYDRA_ADMIN_URL", format!("http://{hydra_name}:4445"))
            .with_env_var("HYDRA_PUBLIC_URL", format!("http://{hydra_name}:4444"))
            .with_env_var("KRATOS_ADMIN_URL", format!("http://{kratos_name}:4434"))
            .with_env_var("KRATOS_PUBLIC_URL", format!("http://{kratos_name}:4433"))
            .with_env_var("KETO_READ_URL", format!("http://{keto_name}:4466"))
            .with_env_var("KETO_WRITE_URL", format!("http://{keto_name}:4467"))
            .with_env_var("KETO_GRPC_URL", format!("http://{keto_name}:4469"))
            .with_env_var("SYSTEM_TENANT_ULID", SYSTEM_TENANT_ULID)
            .with_env_var("SYSTEM_BOOTSTRAP_CLIENT_ID", "system-bootstrap-client")
            .with_env_var("SYSTEM_BOOTSTRAP_CLIENT_SECRET", &bootstrap_client_secret)
            .with_env_var("STATE_COOKIE_SECRET", &state_cookie_secret)
            // PUBLIC_BASE_URL is intentionally a placeholder because the host port is not
            // known until after the container starts. This is fine for Connect-RPC tests;
            // browser OAuth/SAML flows that rely on redirects may need additional setup.
            .with_env_var("PUBLIC_BASE_URL", "http://127.0.0.1:8080")
            .with_env_var("ALLOWED_RETURN_TO_HOSTS", "127.0.0.1,localhost")
            .with_env_var("COOKIE_SECURE", "true")
            .with_env_var("COOKIE_SAMESITE", "Lax")
            .with_startup_timeout(Duration::from_secs(180));

        for (key, value) in &self.extra_env {
            image = image.with_env_var(key, value);
        }

        let gateway = image.start().await?;

        let host = gateway
            .get_host()
            .await
            .map_err(|e| testcontainers::TestcontainersError::other(format!("host: {e}")))?
            .to_string();
        let host_port = gateway
            .get_host_port_ipv4(ContainerPort::Tcp(Self::PORT))
            .await
            .map_err(|e| testcontainers::TestcontainersError::other(format!("port: {e}")))?;
        let endpoint = format!("http://{host}:{host_port}");

        wait_for_gateway(&endpoint)
            .await
            .map_err(|e| testcontainers::TestcontainersError::other(format!("ready: {e}")))?;

        Ok(SsoGatewayHandle {
            endpoint,
            _postgres: postgres,
            _hydra,
            _kratos,
            _keto,
            gateway,
        })
    }
}

impl Default for SsoGateway {
    fn default() -> Self {
        Self {
            image_name: Self::DEFAULT_IMAGE_NAME.to_owned(),
            image_tag: Self::DEFAULT_IMAGE_TAG.to_owned(),
            postgres_tag: Postgres::DEFAULT_TAG.to_owned(),
            hydra_tag: Hydra::DEFAULT_TAG.to_owned(),
            kratos_tag: Kratos::DEFAULT_TAG.to_owned(),
            keto_tag: Keto::DEFAULT_TAG.to_owned(),
            extra_env: HashMap::new(),
        }
    }
}

/// A running sso-gateway stack.
///
/// Dropping this value stops and removes all containers. The only public information is
/// the gateway endpoint address.
pub struct SsoGatewayHandle {
    endpoint: String,
    #[allow(dead_code)]
    _postgres: ContainerAsync<GenericImage>,
    #[allow(dead_code)]
    _hydra: ContainerAsync<GenericImage>,
    #[allow(dead_code)]
    _kratos: ContainerAsync<GenericImage>,
    #[allow(dead_code)]
    _keto: ContainerAsync<GenericImage>,
    gateway: ContainerAsync<GenericImage>,
}

impl SsoGatewayHandle {
    /// Return the gateway's public HTTP endpoint.
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// Stop the gateway container and drop the stack.
    ///
    /// The containers are removed when the handle is dropped; this method is provided
    /// as an explicit hook for tests that want to await cleanup.
    pub async fn shutdown(self) {
        drop(self.gateway);
        // Give testcontainers a moment to schedule removals before returning.
        sleep(Duration::from_millis(100)).await;
    }
}

async fn wait_for_gateway(endpoint: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()?;
    let url = format!("{endpoint}/.well-known/openid-configuration");
    let deadline = Instant::now() + Duration::from_secs(60);

    loop {
        match client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => return Ok(()),
            _ => {
                if Instant::now() >= deadline {
                    return Err(format!("gateway did not become ready at {url}").into());
                }
                sleep(Duration::from_millis(500)).await;
            }
        }
    }
}
