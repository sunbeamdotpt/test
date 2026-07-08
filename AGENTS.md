# sunbeam-test — Agent Guide

## Project purpose

`sunbeam-test` is a collection of opinionated [`testcontainers-rs`](https://docs.rs/testcontainers) builders for services deployed across Sunbeam projects. It is developed in `/Users/sienna/Development/sunbeam/test` and consumed by integration tests in sibling repositories (notably `../sso-gateway`).

## Module conventions

Every public service module follows the same shape:

- A `#[derive(Debug, Clone)]` builder struct (e.g. `Hydra`).
- Constants `NAME`, `DEFAULT_TAG`, and port constants.
- `new()` and `Default`.
- Fluent `with_*` setters.
- An async `start(self) -> Result<ContainerAsync<GenericImage>, testcontainers::TestcontainersError>`.
- Builders default to the `default` Docker network and do **not** publish ports unless explicitly asked.

If a module needs a config file baked into the image, follow `Headscale`, `Kratos`, `Keto`, or `Tuwunel`: generate a unique local image tag, call `util::build_image` with an in-memory Dockerfile, and start from the built tag.

## Networking

- Default behaviour is bridge-network access: containers attach to the `default` network
  and consumers can connect via `container_bridge_ip(container.id())` using the original
  container ports.
- Most builders expose `publish_ports()` / `publish_port()` for runtimes that need
  host-mapped ports (for example **lima-docker** on macOS, where bridge IPs are not
  reachable from the host). When published ports are used, resolve URLs via the module's
  URL helper (for example `Kratos::public_url`) or `util::container_host_url(container, port)`.
- The `SsoGateway` orchestrator creates a private Docker network per stack and uses
  deterministic container names so the gateway container can reach Postgres, Hydra,
  Kratos, and Keto by name. The gateway container itself always publishes a dynamic host
  port; callers only need `SsoGatewayHandle::endpoint()`.

## SsoGateway orchestrator

`SsoGateway` is the single-API entry point for consumers that need a running sso-gateway:

```rust
let gateway = sunbeam_test::SsoGateway::new()
    .with_image("ghcr.io/sunbeamdotpt/sso-gateway", "latest")
    .start()
    .await?;

let endpoint = gateway.endpoint();
```

- It starts Postgres, Hydra, Kratos, and Keto on a private network, then starts the supplied pre-built sso-gateway image.
- It exposes **only** `endpoint()` and `shutdown()`. Backing containers and their URLs are private.
- It always uses dynamic host ports; there is no fixed-port option.
- The default image is `ghcr.io/sunbeamdotpt/sso-gateway:latest`. Callers must ensure this image exists locally or is pullable.

## Testing

```bash
# Run tests that do not require the sso-gateway image
cargo test

# Run the ignored sso-gateway stack test (requires a pre-built image)
cargo test -- --ignored
```

Tests expect a Docker-compatible runtime at `DOCKER_HOST`. On macOS this project is
verified with **lima-docker**, whose socket is usually at
`~/.lima/docker/sock/docker.sock`. Because bridge IPs are not reachable from the macOS
host in this setup, tests use published ports and dynamic host ports.

```bash
DOCKER_HOST=unix://$HOME/.lima/docker/sock/docker.sock cargo test
```

## Adding a new module

1. Create `src/<service>.rs` matching the conventions above.
2. Add `pub mod <service>;` and `pub use <service>::<Service>;` in `src/lib.rs`.
3. Add an integration test in `tests/<service>_image.rs`.
4. Update `README.md`.
