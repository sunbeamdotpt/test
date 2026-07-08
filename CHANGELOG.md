# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Calendar Versioning](https://calver.org/) (`YYYY.MM.MICRO`).

## [2026.07.2] - 2026-07-08

Cargo's semver parser normalizes this to `2026.7.2` in `Cargo.toml`; the release
/tag use the zero-padded CalVer form `2026.07.2`.

### Added

- `OpenFga` testcontainers builder for `openfga/openfga:v1.16.0`, defaulting to
  the in-memory datastore and exposing HTTP, gRPC, and metrics ports.

## [2026.07.1] - 2026-07-08

Cargo's semver parser normalizes this to `2026.7.1` in `Cargo.toml`; the release
/tag use the zero-padded CalVer form `2026.07.1`.

### Added

- Initial CalVer release of `sunbeam-test`.
- Testcontainers builders for Hydra, Kratos, Keto, Postgres, OpenBao, OpenSearch,
  Stalwart, SearXNG, Headscale, and Tuwunel.
- `SsoGateway` orchestrator that starts Postgres, Hydra, Kratos, and Keto on a
  private Docker network and runs a pre-built sso-gateway image, exposing a single
  dynamic endpoint.
- Optional `publish_ports()` / `publish_port()` setters and host-URL helpers so
  tests can use dynamic host ports instead of bridge-network IPs.
