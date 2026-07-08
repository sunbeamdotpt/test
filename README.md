# sunbeam-test

Custom [`testcontainers-rs`](https://github.com/testcontainers/testcontainers-rs) modules for the services that power [`../sbbb`](../sbbb). These builders handle the boilerplate of starting containerised dependencies for integration tests, with defaults matching the images and versions used by the Sunbeam deployment.

## Supported services

| Module | Image default | Notes |
|--------|---------------|-------|
| `Kratos` | `oryd/kratos` | Identity server, runs `serve public` |
| `Hydra` | `oryd/hydra` | OAuth2/OIDC server, runs `serve public` |
| `Keto` | `oryd/keto` | Permission server, runs `serve` |
| `OpenFga` | `openfga/openfga` | ReBAC permission server, in-memory datastore |
| `OpenBao` | `openbao/openbao` | Dev mode (auto-unsealed) with a known root token |
| `OpenSearch` | `opensearchproject/opensearch` | Single-node cluster with security disabled |
| `Stalwart` | `stalwartlabs/mail-server` | Bootstrap mode; admin password is read from container logs |
| `SearXNG` | `searxng/searxng` | Binds on `0.0.0.0:8080` so it is reachable from the bridge network |
| `Headscale` | `headscale/headscale` | Derived image with a baked-in `config.yaml` |
| `Tuwunel` | `ghcr.io/matrix-construct/tuwunel` | Derived image with a baked-in `tuwunel.toml` |
| `Postgres` | `postgres` | `ory/ory/ory` credentials; optional published port |
| `SsoGateway` | `ghcr.io/sunbeamdotpt/sso-gateway` | Full stack (Postgres + Hydra + Kratos + Keto + gateway image) on a private network; exposes a single endpoint |

## Usage

Call `.publish_ports()` (or `.publish_port()` for `Postgres`) when you need to reach a
container from the test host. Each builder exposes a URL helper that resolves the
correct host and dynamic port for the running container:

```rust
use sunbeam_test::OpenBao;

#[tokio::test]
async fn openbao_is_healthy() {
    let container = OpenBao::new().publish_ports().start().await.unwrap();
    let url = format!("{}/v1/sys/health", OpenBao::url(&container).await.unwrap());

    let resp = reqwest::get(&url).await.unwrap();
    assert!(resp.status().is_success());
}
```

Most builders expose a fluent API for tags, config overrides, and credentials:

```rust
let container = sunbeam_test::OpenBao::new()
    .with_tag("2.1.0")
    .with_root_token("my-token")
    .publish_ports()
    .start()
    .await
    .unwrap();
```

For sso-gateway, use the orchestrator to start the whole stack from a pre-built image:

```rust
let gateway = sunbeam_test::SsoGateway::new()
    .with_image("ghcr.io/sunbeamdotpt/sso-gateway", "latest")
    .start()
    .await
    .unwrap();

let endpoint = gateway.endpoint(); // http://127.0.0.1:<random-port>
```

## Running the tests

The tests work with any Docker-compatible runtime.

- **macOS with Docker Desktop**: leave `DOCKER_HOST` unset or set it to the Docker socket.

- **macOS with lima-docker**: point `DOCKER_HOST` at the lima VM socket, for example
  `unix:///Users/sienna/.lima/docker/sock/docker.sock`. Container bridge IPs are not
  reachable from the macOS host in this setup, so the tests rely on published ports.

- **Linux / Windows / everyone else**: use Docker, Podman, Rancher Desktop, or any other
  runtime that exposes a Docker-compatible socket. Point `DOCKER_HOST` at it, or leave it
  unset if the default socket location works.

```bash
cargo test
```

Tests connect to containers using published ports and dynamic host ports rather than
bridge-network IPs.

## License

This project is licensed under the MIT License — see [LICENSE](LICENSE).
