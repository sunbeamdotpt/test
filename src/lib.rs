//! Custom [`testcontainers`](https://docs.rs/testcontainers) helpers for the Ory stack.
//!
//! Each module provides a small, opinionated builder that spins up a throwaway
//! container for integration tests:
//!
//! * [`Kratos`](kratos::Kratos) – identity & user management (defaults to `oryd/kratos:v25.4.0`)
//! * [`Hydra`](hydra::Hydra) – OAuth2 / OIDC provider (defaults to `oryd/hydra:v25.4.0`)
//! * [`Keto`](keto::Keto) – authorization / permission engine (defaults to `oryd/keto:v26.2.0`)
//!
//! The image tags match the versions pinned in the `../sbbb` deployment repository.
//!
//! The builders default to an in-memory DSN and a log-based readiness check so they
//! work out of the box with container runtimes (such as **socktainer**) that do not
//! publish container ports to the host. Connect to the containers using their bridge
//! IP address and the original container ports.
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

pub mod hydra;
pub mod keto;
pub mod kratos;

pub(crate) mod util;

pub use hydra::Hydra;
pub use keto::Keto;
pub use kratos::Kratos;
pub use util::container_bridge_ip;
