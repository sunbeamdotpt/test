use std::time::Duration;

use testcontainers::{
    core::ContainerPort, runners::AsyncRunner, ContainerAsync, GenericImage, ImageExt,
};

/// Testcontainers builder for OpenSearch.
///
/// Defaults to the `opensearchproject/opensearch:3` image used by `../sbbb`.
/// Security is disabled and a single-node cluster is configured so the
/// container is usable for integration tests without TLS or authentication.
#[derive(Debug, Clone)]
pub struct OpenSearch {
    tag: String,
    admin_password: String,
}

impl OpenSearch {
    pub const NAME: &'static str = "opensearchproject/opensearch";
    pub const DEFAULT_TAG: &'static str = "3";

    /// REST API port.
    pub const REST_PORT: u16 = 9200;
    /// Inter-node transport port.
    pub const TRANSPORT_PORT: u16 = 9300;

    /// Create a new OpenSearch builder with the default configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Override the image tag.
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tag = tag.into();
        self
    }

    /// Override the initial admin password.
    pub fn with_admin_password(mut self, password: impl Into<String>) -> Self {
        self.admin_password = password.into();
        self
    }

    /// Start a single-node OpenSearch container with security disabled.
    pub async fn start(
        self,
    ) -> Result<ContainerAsync<GenericImage>, testcontainers::TestcontainersError> {
        GenericImage::new(Self::NAME, &self.tag)
            .with_exposed_port(ContainerPort::Tcp(Self::REST_PORT))
            .with_exposed_port(ContainerPort::Tcp(Self::TRANSPORT_PORT))
            .with_env_var("discovery.type", "single-node")
            .with_env_var("DISABLE_SECURITY_PLUGIN", "true")
            .with_env_var(
                "OPENSEARCH_INITIAL_ADMIN_PASSWORD",
                self.admin_password.clone(),
            )
            .with_env_var("OPENSEARCH_JAVA_OPTS", "-Xms512m -Xmx512m")
            .with_env_var("bootstrap.memory_lock", "true")
            .with_network("default")
            .with_startup_timeout(Duration::from_secs(180))
            .start()
            .await
    }
}

impl Default for OpenSearch {
    fn default() -> Self {
        Self {
            tag: Self::DEFAULT_TAG.to_owned(),
            admin_password: "MyS+ongPwd123".to_owned(),
        }
    }
}
