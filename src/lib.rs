//! Custom [`testcontainers`](https://docs.rs/testcontainers) helpers for the services
//! deployed in the `../sbbb` repository.
//!
//! Each module provides a small, opinionated builder that spins up a throwaway
//! container for integration tests:
//!
//! * [`Kratos`](kratos::Kratos) – Ory identity & user management
//! * [`Hydra`](hydra::Hydra) – Ory OAuth2 / OIDC provider
//! * [`Keto`](keto::Keto) – Ory authorization / permission engine
//! * [`OpenFga`](openfga::OpenFga) – OpenFGA authorization / permission engine
//! * [`Postgres`](postgres::Postgres) – PostgreSQL metadata store
//! * [`SsoGateway`](sso_gateway::SsoGateway) – full sso-gateway stack (pre-built image + deps)
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
//! log-based readiness check. When you need to reach a container from the test host,
//! call `.publish_ports()` (or `.publish_port()` for Postgres) before `.start()` and
//! then use the module's URL helper (for example [`Kratos::public_url`]).
//!
//! # Quick start
//!
//! ```rust,no_run
//! use sunbeam_test::Kratos;
//!
//! # #[tokio::main]
//! # async fn main() {
//! let container = Kratos::default()
//!     .publish_ports()
//!     .start()
//!     .await
//!     .expect("kratos should start");
//!
//! let url = Kratos::public_url(&container)
//!     .await
//!     .expect("kratos url should resolve");
//! # }
//! ```

pub mod headscale;
pub mod hydra;
pub mod keto;
pub mod kratos;
pub mod openfga;
pub mod openbao;
pub mod opensearch;
pub mod otelcol;
pub mod postgres;
pub mod searxng;
pub mod sso_gateway;
pub mod stalwart;
pub mod tuwunel;

pub(crate) mod util;

pub use headscale::Headscale;
pub use hydra::Hydra;
pub use keto::Keto;
pub use kratos::Kratos;
pub use openfga::OpenFga;
pub use openbao::OpenBao;
pub use opensearch::OpenSearch;
pub use otelcol::OtelCollector;
pub use postgres::Postgres;
pub use searxng::SearXng;
pub use sso_gateway::SsoGateway;
pub use stalwart::Stalwart;
pub use tuwunel::Tuwunel;
pub use util::{container_bridge_ip, container_host_url};
