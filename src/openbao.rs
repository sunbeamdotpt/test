use std::time::Duration;

use testcontainers::{
    core::{ContainerPort, WaitFor},
    runners::AsyncRunner,
    ContainerAsync, GenericImage, ImageExt,
};

/// Testcontainers builder for OpenBao.
///
/// Defaults to the `openbao/openbao:2.5.1` image used by `../sbbb`.
/// The container starts in dev mode, which auto-initialises and auto-unseals
/// the server, so no manual unseal step is required.
#[derive(Debug, Clone)]
pub struct OpenBao {
    tag: String,
    root_token: String,
}

impl OpenBao {
    pub const NAME: &'static str = "openbao/openbao";
    pub const DEFAULT_TAG: &'static str = "2.5.1";

    /// API/UI port.
    pub const PORT: u16 = 8200;
    /// Default root token used in dev mode.
    pub const DEFAULT_ROOT_TOKEN: &'static str = "root";

    /// Create a new OpenBao builder with the default configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Override the image tag.
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tag = tag.into();
        self
    }

    /// Override the dev-mode root token (defaults to `root`).
    pub fn with_root_token(mut self, token: impl Into<String>) -> Self {
        self.root_token = token.into();
        self
    }

    /// Start a container from the official OpenBao image in dev mode.
    pub async fn start(self) -> Result<ContainerAsync<GenericImage>, testcontainers::TestcontainersError> {
        let root_token_arg = format!("-dev-root-token-id={}", self.root_token);
        GenericImage::new(Self::NAME, &self.tag)
            .with_exposed_port(ContainerPort::Tcp(Self::PORT))
            .with_wait_for(WaitFor::message_on_either_std(
                "OpenBao server started! Log data will stream in below:",
            ))
            .with_cmd(vec![
                "server".to_string(),
                "-dev".to_string(),
                root_token_arg,
                "-dev-listen-address=0.0.0.0:8200".to_string(),
            ])
            .with_env_var("BAO_DEV_ROOT_TOKEN_ID", self.root_token.clone())
            .with_env_var("BAO_DEV_LISTEN_ADDRESS", "0.0.0.0:8200")
            .with_network("default")
            .with_startup_timeout(Duration::from_secs(120))
            .start()
            .await
    }
}

impl Default for OpenBao {
    fn default() -> Self {
        Self {
            tag: Self::DEFAULT_TAG.to_owned(),
            root_token: Self::DEFAULT_ROOT_TOKEN.to_owned(),
        }
    }
}
