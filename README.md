# sunbeam-test

Custom [`testcontainers-rs`](https://github.com/testcontainers/testcontainers-rs) modules for the services that power [`../sbbb`](../sbbb). These builders handle the boilerplate of starting containerised dependencies for integration tests, with defaults matching the images and versions used by the Sunbeam deployment.

## Supported services

| Module | Image default | Notes |
|--------|---------------|-------|
| `Kratos` | `oryd/kratos` | Identity server, runs `serve public` |
| `Hydra` | `oryd/hydra` | OAuth2/OIDC server, runs `serve public` |
| `Keto` | `oryd/keto` | Permission server, runs `serve` |
| `OpenBao` | `openbao/openbao` | Dev mode (auto-unsealed) with a known root token |
| `OpenSearch` | `opensearchproject/opensearch` | Single-node cluster with security disabled |
| `Stalwart` | `stalwartlabs/mail-server` | Bootstrap mode; admin password is read from container logs |
| `SearXNG` | `searxng/searxng` | Binds on `0.0.0.0:8080` so it is reachable from the bridge network |
| `Headscale` | `headscale/headscale` | Derived image with a baked-in `config.yaml` |
| `Tuwunel` | `ghcr.io/matrix-construct/tuwunel` | Derived image with a baked-in `tuwunel.toml` |

## Usage

```rust
use sunbeam_test::{container_bridge_ip, OpenBao};

#[tokio::test]
async fn openbao_is_healthy() {
    let container = OpenBao::new().start().await.unwrap();
    let host = container_bridge_ip(container.id()).await.unwrap();
    let url = format!("http://{host}:8200/v1/sys/health");

    let resp = reqwest::get(&url).await.unwrap();
    assert!(resp.status().is_success());
}
```

Most builders expose a fluent API for tags, config overrides, and credentials:

```rust
let container = sunbeam_test::OpenBao::new()
    .with_tag("2.1.0")
    .with_root_token("my-token")
    .start()
    .await
    .unwrap();
```

## Running the tests

The tests work with any Docker-compatible runtime.

- **macOS**: this project is developed and tested with [**socktainer**](https://github.com/sunbeam-pt/socktainer), which exposes a Docker socket at `~/.socktainer/container.sock`.

  ```bash
  # with socktainer running in the background
  DOCKER_HOST=unix://$HOME/.socktainer/container.sock cargo test
  ```

- **Linux / Windows / everyone else**: use Docker, Podman, Rancher Desktop, or any other runtime that exposes a Docker-compatible socket. Point `DOCKER_HOST` at it, or leave it unset if the default socket location works.

Because these modules are intended for bridge-network test environments (no published ports), tests connect to containers using `container_bridge_ip(container.id())` rather than `get_host_port_ipv4`.

## License

This project is licensed under the MIT License — see [LICENSE](LICENSE).
