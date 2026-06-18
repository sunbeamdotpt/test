use std::time::{Duration, SystemTime, UNIX_EPOCH};

use testcontainers::{
    core::{ContainerPort, WaitFor},
    runners::AsyncRunner,
    ContainerAsync, GenericImage, ImageExt,
};

use crate::util;

/// Default Keto configuration that lets the container start with an in-memory DSN.
const CONFIG: &str = include_str!("keto.yml");

/// Returns a unique tag so parallel test runs don't race building the same image tag.
fn unique_tag() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{nanos:x}")
}

/// Testcontainers builder for Ory Keto.
///
/// Defaults to the `oryd/keto:v26.2.0` image used by `../sbbb`.
#[derive(Debug, Clone)]
pub struct Keto {
    tag: String,
    config: String,
    built_tag: String,
}

impl Keto {
    pub const NAME: &'static str = "oryd/keto";
    pub const DEFAULT_TAG: &'static str = "v26.2.0";

    /// Read / check API port.
    pub const READ_PORT: u16 = 4466;
    /// Write / admin API port.
    pub const WRITE_PORT: u16 = 4467;
    /// Metrics port.
    pub const METRICS_PORT: u16 = 4468;

    /// Create a new Keto builder with the default configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Override the image tag.
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tag = tag.into();
        self
    }

    /// Replace the embedded `keto.yml` configuration.
    pub fn with_config(mut self, config: impl Into<String>) -> Self {
        self.config = config.into();
        self
    }

    /// Build a small derived image containing the config and start a container from it.
    pub async fn start(self) -> Result<ContainerAsync<GenericImage>, testcontainers::TestcontainersError> {
        let image_name = "sunbeam-test/keto";
        let descriptor = format!("{image_name}:{}", self.built_tag);

        let dockerfile = format!(
            "FROM {}:{}\nCOPY keto.yml /home/ory/keto.yml\nCMD [\"serve\", \"-c\", \"/home/ory/keto.yml\"]\n",
            Self::NAME,
            self.tag
        );

        util::build_image(&descriptor, &dockerfile, &[("keto.yml", self.config.as_bytes())])
            .await
            .map_err(|e| testcontainers::TestcontainersError::other(format!("build keto image: {e}")))?;

        GenericImage::new(image_name, &self.built_tag)
            .with_exposed_port(ContainerPort::Tcp(Self::READ_PORT))
            .with_exposed_port(ContainerPort::Tcp(Self::WRITE_PORT))
            .with_exposed_port(ContainerPort::Tcp(Self::METRICS_PORT))
            .with_wait_for(WaitFor::message_on_either_std("Successfully applied"))
            .with_env_var("DSN", "memory")
            .with_network("default")
            .with_startup_timeout(Duration::from_secs(120))
            .start()
            .await
    }
}

impl Default for Keto {
    fn default() -> Self {
        Self {
            tag: Self::DEFAULT_TAG.to_owned(),
            config: CONFIG.to_owned(),
            built_tag: unique_tag(),
        }
    }
}
