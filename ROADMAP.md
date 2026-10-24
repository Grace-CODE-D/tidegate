# Tidegate roadmap

Tidegate is now organized around Stellar: a Soroban gateway registry contract plus a read-only Stellar RPC gateway. The roadmap focuses on useful, reviewable improvements to that foundation.

## Near-term Stellar work

- Add a read-only contract-record query that reads gateway registrations from a configured registry contract.
- Add a typed Stellar RPC response model for latest-ledger data instead of returning an unstructured JSON value.
- Add mocked JSON-RPC integration tests for successful responses, upstream errors, malformed payloads, and timeouts.
- Add explicit Testnet and Futurenet configuration examples without embedding identities or secrets.
- Add a Stellar CLI deployment guide for the registry contract and a documented post-deployment verification flow.

## Contract improvements

- Add contract events for registration, update, and deactivation so indexers can observe registry changes.
- Review storage durability and rent-related behavior for the target protocol version.
- Add more authorization and boundary tests, including unauthorized updates and maximum endpoint/version lengths.
- Consider a separate governance or moderation layer only after a concrete use case is documented. The registry should not silently become a trust oracle.

## Gateway improvements

- Add request-size limits and clearer JSON-RPC timeout configuration.
- Add optional, freshness-aware caching for read-only Stellar RPC responses.
- Add safe proxy-header configuration rather than always trusting `X-Forwarded-For`.
- Add Prometheus-compatible metrics without unbounded labels.
- Add a pluggable storage interface for non-chain application data.

All work should remain scoped to a real issue, include tests, document operational limits, and avoid private-key custody or automatic Mainnet deployment.
