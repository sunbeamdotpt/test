//! Custom [`testcontainers`](https://docs.rs/testcontainers) helpers for the services
//! deployed in the `../sbbb` repository.
//!
//! Each module provides a small, opinionated builder that spins up a throwaway
//! container for integration tests:
//!
//! * [`Kratos`](kratos::Kratos) – Ory identity & user management
//! * [`Hydra`](hydra::Hydra) – Ory OAuth2 / OIDC provider
//! * [`Keto`](keto::Keto) – Ory authorization / permission engine
//! * [`OpenBao`](openbao::OpenBao) – secrets management
//! * [`OpenSearch`](opensearch::OpenSearch) – search & analytics
//! * [`Stalwart`](stalwart::Stalwart) – mail server (JMAP/IMAP/SMTP)
//! * [`SearXng`](searxng::SearXng) – privacy metasearch
//! * [`Headscale`](headscale::Headscale) – self-hosted Tailscale control server
//! * [`Tuwunel`](tuwunel::Tuwunel) – Matrix homeserver
//!
//! The image tags match the versions pinned in the `../sbbb` deployment repository.
//!
//! The builders default to an in-memory / single-node / dev-mode configuration and a
//! log-based readiness check so they work out of the box with container runtimes (such
//! as **socktainer**) that do not publish container ports to the host. Connect to the
//! containers using their bridge IP address and the original container ports.
//!
//! # Quick start
//!
//! ```rust,no_run
//! use sunbeam_test::Kratos;
//!
//! # #[tokio::main]
//! # async fn main() {
//! let container = Kratos::default()
//!     .start()
//!     .await
//!     .expect("kratos should start");
//!
//! let host = container
//!     .get_bridge_ip_address()
//!     .await
//!     .expect("bridge ip should resolve");
//! let url = format!("http://{host}:4433/health/ready");
//! # }
//! ```

pub mod headscale;
pub mod hydra;
pub mod keto;
pub mod kratos;
pub mod openbao;
pub mod opensearch;
pub mod searxng;
pub mod stalwart;
pub mod tuwunel;

pub(crate) mod util;

pub use headscale::Headscale;
pub use hydra::Hydra;
pub use keto::Keto;
pub use kratos::Kratos;
pub use openbao::OpenBao;
pub use opensearch::OpenSearch;
pub use searxng::SearXng;
pub use stalwart::Stalwart;
pub use tuwunel::Tuwunel;
pub use util::container_bridge_ip;
