use std::time::Duration;

use testcontainers::{
    core::{ContainerPort, WaitFor},
    runners::AsyncRunner,
    ContainerAsync, GenericImage, ImageExt,
};


/// Testcontainers builder for Ory Hydra.
///
/// Defaults to the `oryd/hydra:v25.4.0` image used by `../sbbb`.
#[derive(Debug, Clone)]
pub struct Hydra {
    tag: String,
    dsn: String,
}

impl Hydra {
    pub const NAME: &'static str = "oryd/hydra";
    pub const DEFAULT_TAG: &'static str = "v25.4.0";

    /// Public OAuth2 / OIDC port.
    pub const PUBLIC_PORT: u16 = 4444;
    /// Admin API port.
    pub const ADMIN_PORT: u16 = 4445;

    /// Create a new Hydra builder with the default configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Override the image tag.
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tag = tag.into();
        self
    }

    /// Override the `DSN` value (defaults to `memory`).
    pub fn with_dsn(mut self, dsn: impl Into<String>) -> Self {
        self.dsn = dsn.into();
        self
    }

    /// Start a container from the official Hydra image.
    pub async fn start(self) -> Result<ContainerAsync<GenericImage>, testcontainers::TestcontainersError> {
        GenericImage::new(Self::NAME, &self.tag)
            .with_exposed_port(ContainerPort::Tcp(Self::PUBLIC_PORT))
            .with_exposed_port(ContainerPort::Tcp(Self::ADMIN_PORT))
            .with_wait_for(WaitFor::message_on_either_std("Successfully applied"))
            .with_cmd(["serve", "all", "--dev"])
            .with_env_var("DSN", self.dsn.clone())
            .with_network("default")
            .with_startup_timeout(Duration::from_secs(120))
            .start()
            .await
    }
}

impl Default for Hydra {
    fn default() -> Self {
        Self {
            tag: Self::DEFAULT_TAG.to_owned(),
            dsn: "memory".to_owned(),
        }
    }
}
