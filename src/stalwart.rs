use std::time::Duration;

use testcontainers::{
    core::{ContainerPort},
    runners::AsyncRunner,
    ContainerAsync, GenericImage, ImageExt,
};

/// Testcontainers builder for Stalwart Mail Server.
///
/// Defaults to the `stalwartlabs/stalwart:v0.15.5` image used by `../sbbb`.
/// The container starts in bootstrap mode and exposes the web admin / JMAP
/// listener on port 8080.
#[derive(Debug, Clone)]
pub struct Stalwart {
    tag: String,
}

impl Stalwart {
    pub const NAME: &'static str = "stalwartlabs/stalwart";
    pub const DEFAULT_TAG: &'static str = "v0.15.5";

    /// Web admin + JMAP port.
    pub const PORT: u16 = 8080;

    /// Create a new Stalwart builder with the default configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Override the image tag.
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tag = tag.into();
        self
    }

    /// Start a Stalwart container in bootstrap mode.
    pub async fn start(self) -> Result<ContainerAsync<GenericImage>, testcontainers::TestcontainersError> {
        GenericImage::new(Self::NAME, &self.tag)
            .with_exposed_port(ContainerPort::Tcp(Self::PORT))
            .with_env_var("STALWART_HOSTNAME", "localhost")
            .with_network("default")
            .with_startup_timeout(Duration::from_secs(90))
            .start()
            .await
    }

    /// Extract the temporary bootstrap admin password from the container logs.
    ///
    /// The v0.15.5 image prints a random password on first start. This helper
    /// parses it so tests can authenticate against the admin API if needed.
    pub async fn admin_password(
        container: &ContainerAsync<GenericImage>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let stdout = container.stdout_to_vec().await?;
        let stderr = container.stderr_to_vec().await?;
        let logs = String::from_utf8_lossy(&stdout).to_string()
            + &String::from_utf8_lossy(&stderr);

        let marker = "Your administrator account is 'admin' with password '";
        let start = logs
            .find(marker)
            .ok_or("admin password banner not found in stalwart logs")?
            + marker.len();
        let rest = &logs[start..];
        let end = rest
            .find('\'')
            .ok_or("malformed admin password banner")?;
        Ok(rest[..end].to_string())
    }
}

impl Default for Stalwart {
    fn default() -> Self {
        Self {
            tag: Self::DEFAULT_TAG.to_owned(),
        }
    }
}
