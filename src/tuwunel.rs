use std::time::{Duration, SystemTime, UNIX_EPOCH};

use testcontainers::{
    core::ContainerPort, runners::AsyncRunner, ContainerAsync, GenericImage, ImageExt,
};

use crate::util;

/// Minimal Tuwunel configuration for integration tests.
const CONFIG: &str = r#"[global]
server_name = "localhost"
database_path = "/var/lib/tuwunel"
port = [8008]
address = "0.0.0.0"
allow_registration = true
registration_token = "sunbeam-test-token"
allow_encryption = true
allow_federation = false
log = "info"
"#;

/// Returns a unique tag so parallel test runs don't race building the same image tag.
fn unique_tag() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{nanos:x}")
}

/// Testcontainers builder for Tuwunel.
///
/// Defaults to the public `ghcr.io/matrix-construct/tuwunel:latest` image.
/// `../sbbb` uses a custom `src.DOMAIN_SUFFIX/studio/tuwunel` image; override
/// it with `.with_tag(...)` if you need to match the deployment exactly.
#[derive(Debug, Clone)]
pub struct Tuwunel {
    tag: String,
    config: String,
    built_tag: String,
}

impl Tuwunel {
    pub const NAME: &'static str = "ghcr.io/matrix-construct/tuwunel";
    pub const DEFAULT_TAG: &'static str = "latest";

    /// Client / federation port.
    pub const PORT: u16 = 8008;

    /// Create a new Tuwunel builder with the default configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Override the image tag.
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tag = tag.into();
        self
    }

    /// Replace the embedded `tuwunel.toml` configuration.
    pub fn with_config(mut self, config: impl Into<String>) -> Self {
        self.config = config.into();
        self
    }

    /// Build a small derived image containing the config and start a container from it.
    pub async fn start(
        self,
    ) -> Result<ContainerAsync<GenericImage>, testcontainers::TestcontainersError> {
        let image_name = "sunbeam-test/tuwunel";
        let descriptor = format!("{image_name}:{}", self.built_tag);

        let dockerfile = format!(
            "FROM {}:{}\nCOPY tuwunel.toml /etc/tuwunel.toml\nENV TUWUNEL_CONFIG=/etc/tuwunel.toml\n",
            Self::NAME,
            self.tag
        );

        util::build_image(
            &descriptor,
            &dockerfile,
            &[("tuwunel.toml", self.config.as_bytes())],
        )
        .await
        .map_err(|e| {
            testcontainers::TestcontainersError::other(format!("build tuwunel image: {e}"))
        })?;

        GenericImage::new(image_name, &self.built_tag)
            .with_exposed_port(ContainerPort::Tcp(Self::PORT))
            .with_network("default")
            .with_startup_timeout(Duration::from_secs(120))
            .start()
            .await
    }
}

impl Default for Tuwunel {
    fn default() -> Self {
        Self {
            tag: Self::DEFAULT_TAG.to_owned(),
            config: CONFIG.to_owned(),
            built_tag: unique_tag(),
        }
    }
}
