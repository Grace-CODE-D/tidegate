# Tidegate

Tidegate is a compact Rust edge gateway for small services that need a dependable HTTP boundary without immediately adopting a database or a large platform. It combines a bounded JSON cache, a simple per-client rate limiter, health reporting, request metrics, and a security-focused CI pipeline in one small service that is easy to run locally and easy to extend.

Tidegate is an independent clean-room implementation. It is not affiliated with Lumen-Scribe, Lumenqraph, Drips, or any other project. It follows common gateway design patterns without copying another repository's source code, documentation, branding, or history.

## What it does

A request enters through the Axum router and passes through the rate-limit middleware. Accepted requests reach one of the health, metrics, or cache handlers. Cache state is held in a bounded process-local store, while counters are exposed through a small JSON metrics endpoint.

```text
client
  |
  v
HTTP router -> rate-limit middleware -> health / metrics / cache handlers
                                      |                |
                                      v                v
                              operational counters   bounded JSON store
```

The service is intentionally focused. It does not pretend to be a durable database, a malware detector, or a complete production API gateway. Those boundaries are documented so that future changes can add capability without hiding operational risk.

## Endpoints

| Method | Path | Purpose |
| --- | --- | --- |
| `GET` | `/healthz` | Returns service status, current cache entries, and configured capacity. |
| `GET` | `/metrics` | Returns request and cache operation counters as JSON. |
| `GET` | `/v1/cache?key=name` | Reads a cached JSON value. Returns `404` if the key is absent. |
| `PUT` | `/v1/cache/:key` | Stores a JSON value and returns `201 Created`. |
| `DELETE` | `/v1/cache/:key` | Removes a value and returns `204` or `404`. |

The cache uses a bounded recency queue. When it reaches capacity, the oldest untouched entry is removed. Updating an existing key refreshes its position. Cache values disappear when the process stops.

## Quick start

Install Rust 1.74 or newer, then run:

```bash
git clone https://github.com/Grace-CODE-D/tidegate.git
cd tidegate
cargo run
```

The default listener is `127.0.0.1:8080`. In another terminal:

```bash
curl -sS http://127.0.0.1:8080/healthz
# {"status":"ok","cached_entries":0,"cache_capacity":256}

curl -sS -X PUT http://127.0.0.1:8080/v1/cache/greeting \
  -H 'content-type: application/json' \
  -d '{"value":{"message":"hello","source":"demo"}}'

curl -sS 'http://127.0.0.1:8080/v1/cache?key=greeting'
# {"value":{"message":"hello","source":"demo"}}

curl -sS http://127.0.0.1:8080/metrics
# {"total_requests":4,"rejected_requests":0,"cache_reads":1,"cache_writes":1,"cache_deletes":0}
```

## Configuration

Tidegate reads configuration from environment variables. Invalid numeric values fall back to safe defaults.

| Variable | Default | Description |
| --- | ---: | --- |
| `TIDEGATE_ADDR` | `127.0.0.1:8080` | Bind address and port. |
| `TIDEGATE_RATE_LIMIT` | `60` | Requests permitted per client window. |
| `TIDEGATE_RATE_WINDOW_SECONDS` | `60` | Length of each rate-limit window. |
| `TIDEGATE_CACHE_CAPACITY` | `256` | Maximum number of cached entries. Values below `1` use the default. |
| `RUST_LOG` | `tidegate=info` | `tracing` filter, such as `tidegate=debug`. |

For a local stress test with a small cache:

```bash
TIDEGATE_ADDR=127.0.0.1:9000 \
TIDEGATE_CACHE_CAPACITY=10 \
TIDEGATE_RATE_LIMIT=120 \
RUST_LOG=tidegate=debug \
cargo run
```

The rate limiter uses the first address in `X-Forwarded-For` when present and otherwise groups requests into a local bucket. Only enable that header behind a proxy that overwrites it; do not trust arbitrary client-supplied forwarding headers on a public listener.

## Project structure

The current MVP keeps the implementation deliberately small:

- `src/main.rs` contains the Axum router, middleware, handlers, bounded cache, metrics, and tests.
- `.github/workflows/ci.yml` runs formatting, tests, and Clippy.
- `.github/workflows/security.yml` runs RustSec dependency auditing and the blocked-CDN asset scan.
- `scripts/security-scan.sh` rejects tracked runtime/configuration files that reference `polyfill.io` or `cdn.polyfill.io`.
- `ROADMAP.md` describes the next pieces of work without claiming that they are already implemented.

## Development

Use the same checks locally that run in CI:

```bash
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
./scripts/security-scan.sh
cargo audit
```

The test suite covers cache round trips, bounded eviction, refresh behavior, configuration defaults, and an end-to-end router flow for writing, reading, and measuring a value. No external service is required for tests.

## Security posture and limits

The repository has targeted supply-chain controls rather than a claim of universal malware detection. RustSec audits the locked Rust dependency graph. The asset guard prevents the known Polyfill.io CDN indicators from being introduced into tracked runtime or configuration files. Pull requests should still be reviewed, dependencies should be updated, and production deployments should use host-level monitoring, least privilege, network controls, and secret management.

See [SECURITY.md](SECURITY.md) for reporting guidance and [CONTRIBUTING.md](CONTRIBUTING.md) for the review workflow.

## Roadmap

The next planned improvements are a pluggable storage trait, broader HTTP integration coverage, and clearer deployment guidance. Each change should be proposed in an issue, include tests, explain trade-offs, and link its pull request with a closing keyword.

## License

MIT. See [LICENSE](LICENSE).
