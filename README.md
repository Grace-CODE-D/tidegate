# Tidegate

Tidegate is a small Rust HTTP edge gateway for services that need a health endpoint, a bounded in-memory JSON cache, and a simple per-client rate limit.

This is an independent clean-room implementation. It is not affiliated with Lumen-Scribe, Lumenqraph, Drips, or any other project. It uses common gateway patterns rather than copying another repository's source, documentation, branding, or history.

## Features

- `GET /healthz` reports service health, current cache usage, and cache capacity.
- `PUT /v1/cache/:key` stores a JSON value in memory.
- `GET /v1/cache?key=...` reads a JSON value.
- `DELETE /v1/cache/:key` removes a value.
- Requests are rate-limited per `X-Forwarded-For` address, falling back to a local client bucket.
- Configuration is provided through environment variables.

The cache is intentionally ephemeral and process-local. It has a configurable bounded recency queue: the oldest untouched entry is evicted when capacity is reached, while updating an entry refreshes its position. Use a durable store and a trusted proxy configuration before using this as a production service.

## Run

```bash
cargo run
```

The server listens on `127.0.0.1:8080` by default. Configure it with:

- `TIDEGATE_ADDR`, for example `0.0.0.0:8080`
- `TIDEGATE_RATE_LIMIT`, requests allowed per window, default `60`
- `TIDEGATE_RATE_WINDOW_SECONDS`, window length, default `60`
- `TIDEGATE_CACHE_CAPACITY`, maximum cached entries, default `256`
- `RUST_LOG`, for example `tidegate=debug`

Example:

```bash
curl http://127.0.0.1:8080/healthz
curl -X PUT http://127.0.0.1:8080/v1/cache/greeting \
  -H 'content-type: application/json' \
  -d '{"value":{"message":"hello"}}'
curl 'http://127.0.0.1:8080/v1/cache?key=greeting'
```

## Development

```bash
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

## Contributing

The project is an MVP and welcomes focused improvements. See [CONTRIBUTING.md](CONTRIBUTING.md) for the review workflow and [ROADMAP.md](ROADMAP.md) for proposed work. Please open or discuss an issue before starting a substantial change.

Security concerns should follow [SECURITY.md](SECURITY.md). Community participation follows the [Code of Conduct](CODE_OF_CONDUCT.md).

## License

MIT. See [LICENSE](LICENSE).
