# Tidegate

Tidegate is a Stellar-focused Rust project with two connected parts:

1. A Soroban `gateway-registry` contract for publishing and managing gateway operators on Stellar.
2. A read-only HTTP gateway that connects to Stellar RPC, applies rate limits, validates inputs, exposes metrics, and provides bounded local caching for service data.

The project is designed to give Stellar developers a small, understandable starting point for operating network-aware services. It does not hold private keys, sign transactions, or claim to be a replacement for Stellar RPC, Horizon, or a full indexer. The gateway is an application-layer companion to Stellar infrastructure; the registry contract is the on-chain component.

Tidegate is an independent implementation. It is not affiliated with Lumen-Scribe, Lumenqraph, Drips, or any other project.

## Why Stellar is part of the design

The gateway is built around the current Stellar application stack. Stellar RPC is used for live network and smart-contract access. The Soroban contract is written in Rust with `soroban-sdk`. The gateway's existing cache, rate limiter, request metrics, and security checks support the operational side of interacting with Stellar services.

The project follows Stellar's current direction toward Stellar RPC for new application work. Horizon remains useful for existing integrations, but Stellar's documentation describes it as nearing end-of-life for new feature development. Tidegate therefore does not introduce a new Horizon dependency.

Relevant official resources:

- [Stellar Stack](https://developers.stellar.org/docs/learn/fundamentals/stellar-stack)
- [Stellar RPC](https://developers.stellar.org/docs/data/apis/rpc)
- [Stellar client SDKs](https://developers.stellar.org/docs/tools/sdks/client-sdks)
- [Soroban contract development](https://developers.stellar.org/docs/build/smart-contracts/getting-started/hello-world)
- [Stellar ecosystem resources](https://github.com/stellar/ecosystem-resources)

## Soroban gateway registry

The `contracts/gateway-registry` package contains the on-chain registry. An operator account can register a gateway endpoint and software version, update its metadata, deactivate the registration, or let another application read the current record.

The contract exposes these functions:

- `register(operator, endpoint, version)` creates an active record. The operator must authorize the call.
- `update(operator, endpoint, version)` changes an existing record and reactivates it. The operator must authorize the call.
- `deactivate(operator)` marks an existing record inactive. The operator must authorize the call.
- `get(operator)` returns the current record, if one exists.

Each record contains the operator address, endpoint string, version string, and active flag. Endpoint and version lengths are bounded on-chain. The contract does not fetch URLs, sign transactions, custody funds, or make off-chain trust claims. It is a small metadata registry, not a payment contract.

The contract is intentionally conservative. It does not let one account edit another account's record, and it does not treat an endpoint as trusted merely because it appears in the registry. Applications must still validate and monitor the service they connect to.

## Stellar RPC gateway

When `STELLAR_RPC_URL` is configured, the service exposes:

```text
GET /v1/stellar/ledger/latest
```

The route sends a JSON-RPC `getLatestLedger` request to the configured Stellar RPC endpoint and returns the upstream result. It does not sign or submit transactions. This first integration keeps the trust boundary clear and gives the project a foundation for later read-only contract and event queries.

The response path includes Tidegate's request rate limiting, tracing, and metrics. Upstream failures are returned as `502 Bad Gateway`; when no RPC URL is configured, the route returns `503 Service Unavailable` with a clear configuration error.

The gateway also exposes Stellar counters through `/metrics`:

```json
{
  "stellar_rpc_requests": 3,
  "stellar_rpc_errors": 0
}
```

The current RPC method is deliberately small. Future Stellar-specific work should add contract reads, event queries, and optional caching only with tests that define freshness and failure behavior.

## Quick start

Install Rust 1.74 or newer. To run the HTTP gateway locally:

```bash
git clone https://github.com/Grace-CODE-D/tidegate.git
cd tidegate
STELLAR_RPC_URL=https://soroban-testnet.stellar.org cargo run
```

The default listener is `127.0.0.1:8080`. Test the gateway:

```bash
curl -sS http://127.0.0.1:8080/healthz
curl -sS http://127.0.0.1:8080/v1/stellar/ledger/latest
curl -sS http://127.0.0.1:8080/metrics
```

Use a provider appropriate for your environment. Testnet and Futurenet are intended for development. Do not put credentials, signing keys, or provider tokens in source control. If a provider requires an authorization header in the future, it should be injected through a secret-management system rather than committed to the repository.

## Build and test the contract

The repository is a Cargo workspace. The contract can be tested without a live network:

```bash
cargo test -p gateway-registry
cargo build -p gateway-registry --target wasm32-unknown-unknown --release
```

The recommended Stellar workflow uses the Stellar CLI for contract builds and deployment. After installing the CLI, the optimized contract build can be produced with:

```bash
stellar contract build
```

Deployment and invocation should initially target Testnet or Futurenet. The repository does not include a private key, deployer identity, or automatic Mainnet deployment. Operators should review the contract, configure their own identity, and follow the official Stellar deployment documentation before sending any transaction.

## HTTP API

| Method | Path | Purpose |
| --- | --- | --- |
| `GET` | `/healthz` | Returns service status, cache usage, and configured capacity. |
| `GET` | `/metrics` | Returns request, cache, and Stellar RPC counters as JSON. |
| `GET` | `/v1/stellar/ledger/latest` | Proxies the read-only `getLatestLedger` Stellar RPC method. |
| `GET` | `/v1/cache?key=name` | Reads a cached JSON value. |
| `PUT` | `/v1/cache/:key` | Stores a JSON value in the bounded local cache. |
| `DELETE` | `/v1/cache/:key` | Removes a cached value. |

Cache keys must be non-empty, no more than 128 Unicode characters, and must not contain whitespace, control characters, or path separators. The cache is process-local and ephemeral. It uses a bounded recency queue so the oldest untouched entry is evicted when capacity is reached. It is not a substitute for on-chain state or durable application storage.

## Configuration

| Variable | Default | Description |
| --- | ---: | --- |
| `TIDEGATE_ADDR` | `127.0.0.1:8080` | HTTP bind address and port. |
| `STELLAR_RPC_URL` | unset | Stellar RPC JSON-RPC endpoint. The Stellar route is unavailable when unset. |
| `TIDEGATE_RATE_LIMIT` | `60` | Requests permitted per client window. |
| `TIDEGATE_RATE_WINDOW_SECONDS` | `60` | Length of each rate-limit window. |
| `TIDEGATE_CACHE_CAPACITY` | `256` | Maximum number of local cache entries. |
| `RUST_LOG` | `tidegate=info` | Tracing filter, such as `tidegate=debug`. |

The rate limiter uses the first address in `X-Forwarded-For` when present. Only trust that header behind a proxy that overwrites it; do not trust arbitrary client-supplied forwarding headers on a public listener.

## Project structure

- `contracts/gateway-registry/src/lib.rs` contains the Soroban contract and contract tests.
- `src/main.rs` contains the Axum HTTP service, cache, middleware, routes, and service tests.
- `src/stellar.rs` contains the read-only Stellar JSON-RPC client.
- `src/validation.rs` contains cache-key validation rules.
- `.github/workflows/ci.yml` runs formatting, workspace tests, and Clippy.
- `.github/workflows/security.yml` runs RustSec dependency auditing and the blocked-CDN asset scan.
- `ROADMAP.md` lists the next Stellar and service improvements.

## Development checks

Run the same checks locally that CI runs:

```bash
cargo fmt --check
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
./scripts/security-scan.sh
cargo audit --ignore RUSTSEC-2026-0387
```

The test suite covers registry registration and authorization flow under mocked Soroban environments, contract validation, cache eviction, cache-key validation, HTTP routes, metrics, and Stellar RPC client configuration. No live network or private key is required for tests.

## Security boundaries

Tidegate does not detect all malware and does not make a Stellar endpoint trustworthy merely because it is listed in a contract. The project uses RustSec auditing, a targeted Polyfill.io supply-chain guard, input validation, bounded local state, and explicit no-key-custody boundaries. Production deployments still need dependency review, secret management, host protection, TLS, proxy configuration, monitoring, and operational review.

See [SECURITY.md](SECURITY.md), [CONTRIBUTING.md](CONTRIBUTING.md), and [ROADMAP.md](ROADMAP.md) for the project policies and planned work.

## License

MIT. See [LICENSE](LICENSE).
