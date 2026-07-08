use std::time::Duration;

use testcontainers::{
    core::{ContainerPort, WaitFor},
    runners::AsyncRunner,
    ContainerAsync, GenericImage, ImageExt,
};

/// Testcontainers builder for SearXNG.
///
/// Defaults to the `searxng/searxng:latest` image used by `../sbbb`.
#[derive(Debug, Clone)]
pub struct SearXng {
    tag: String,
    base_url: String,
}

impl SearXng {
    pub const NAME: &'static str = "searxng/searxng";
    pub const DEFAULT_TAG: &'static str = "latest";

    /// Web / API port.
    pub const PORT: u16 = 8080;

    /// Create a new SearXNG builder with the default configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Override the image tag.
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tag = tag.into();
        self
    }

    /// Override the `BASE_URL` value (defaults to `http://localhost:8080/`).
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// Start a SearXNG container.
    pub async fn start(
        self,
    ) -> Result<ContainerAsync<GenericImage>, testcontainers::TestcontainersError> {
        GenericImage::new(Self::NAME, &self.tag)
            .with_exposed_port(ContainerPort::Tcp(Self::PORT))
            .with_wait_for(WaitFor::message_on_either_std("Started worker-1"))
            .with_env_var("BASE_URL", self.base_url.clone())
            .with_env_var("INSTANCE_NAME", "searxng")
            .with_env_var("GRANIAN_HOST", "0.0.0.0")
            .with_network("default")
            .with_startup_timeout(Duration::from_secs(120))
            .start()
            .await
    }
}

impl Default for SearXng {
    fn default() -> Self {
        Self {
            tag: Self::DEFAULT_TAG.to_owned(),
            base_url: "http://localhost:8080/".to_owned(),
        }
    }
}
