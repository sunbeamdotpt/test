use std::time::{Duration, SystemTime, UNIX_EPOCH};

use testcontainers::{
    core::{ContainerPort, WaitFor},
    runners::AsyncRunner,
    ContainerAsync, GenericImage, ImageExt,
};

use crate::util;

/// Minimal collector configuration: OTLP/HTTP receiver and a debug exporter
/// with detailed verbosity so received span names appear in the container logs.
const CONFIG: &str = r#"
receivers:
  otlp:
    protocols:
      http:
        endpoint: 0.0.0.0:4318
exporters:
  debug:
    verbosity: detailed
service:
  pipelines:
    traces:
      receivers: [otlp]
      exporters: [debug]
"#;

/// Returns a unique tag so parallel test runs don't race building the same image tag.
fn unique_tag() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{nanos:x}")
}

/// Testcontainers builder for the OpenTelemetry collector.
///
/// Starts a collector with an OTLP/HTTP receiver and a debug exporter that
/// dumps received spans to the container logs (stderr).
#[derive(Debug, Clone)]
pub struct OtelCollector {
    tag: String,
    config: String,
    built_tag: String,
    network: Option<String>,
    container_name: Option<String>,
    published_ports: bool,
}

impl OtelCollector {
    pub const NAME: &'static str = "otel/opentelemetry-collector";
    pub const DEFAULT_TAG: &'static str = "0.120.0";

    /// OTLP/HTTP receiver port.
    pub const OTLP_HTTP_PORT: u16 = 4318;

    /// Create a new OtelCollector builder with the default configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Override the image tag.
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tag = tag.into();
        self
    }

    /// Replace the embedded collector configuration.
    pub fn with_config(mut self, config: impl Into<String>) -> Self {
        self.config = config.into();
        self
    }

    /// Publish the OTLP/HTTP port to a random host port so the container is
    /// reachable without bridge-network access.
    pub fn publish_ports(mut self) -> Self {
        self.published_ports = true;
        self
    }

    /// Attach the container to a specific Docker network.
    ///
    /// When no network is set, Docker's default bridge network is used.
    pub fn with_network(mut self, network: impl Into<String>) -> Self {
        self.network = Some(network.into());
        self
    }

    /// Set the Docker container name so other containers can resolve it by name
    /// on the same network.
    pub fn with_container_name(mut self, name: impl Into<String>) -> Self {
        self.container_name = Some(name.into());
        self
    }

    /// Return the OTLP/HTTP endpoint URL for a container that was started with
    /// published ports (`/v1/traces` is appended by the exporter).
    pub async fn endpoint(
        container: &ContainerAsync<GenericImage>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        util::container_host_url(container, Self::OTLP_HTTP_PORT).await
    }

    /// Build a small derived image containing the config and start a container from it.
    pub async fn start(
        self,
    ) -> Result<ContainerAsync<GenericImage>, testcontainers::TestcontainersError> {
        let image_name = "sunbeam-test/otelcol";
        let descriptor = format!("{image_name}:{}", self.built_tag);

        let dockerfile = format!(
            "FROM {}:{}\nCOPY otelcol.yaml /etc/otelcol/config.yaml\nCMD [\"--config\", \"/etc/otelcol/config.yaml\"]\n",
            Self::NAME,
            self.tag
        );

        util::build_image(
            &descriptor,
            &dockerfile,
            &[("otelcol.yaml", self.config.as_bytes())],
        )
        .await
        .map_err(|e| {
            testcontainers::TestcontainersError::other(format!("build otelcol image: {e}"))
        })?;

        let mut image = GenericImage::new(image_name, &self.built_tag)
            .with_exposed_port(ContainerPort::Tcp(Self::OTLP_HTTP_PORT))
            .with_wait_for(WaitFor::message_on_either_std(
                "Everything is ready. Begin running and processing data.",
            ))
            .with_startup_timeout(Duration::from_secs(120));

        if let Some(network) = &self.network {
            image = image.with_network(network);
        }

        if let Some(name) = self.container_name {
            image = image.with_container_name(name);
        }

        if self.published_ports {
            image = image.with_mapped_port(0, ContainerPort::Tcp(Self::OTLP_HTTP_PORT));
        }

        image.start().await
    }
}

impl Default for OtelCollector {
    fn default() -> Self {
        Self {
            tag: Self::DEFAULT_TAG.to_owned(),
            config: CONFIG.to_owned(),
            built_tag: unique_tag(),
            network: None,
            container_name: None,
            published_ports: false,
        }
    }
}
