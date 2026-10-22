# Roadmap

Tidegate is an MVP. The following work is intentionally listed as future, reviewable improvements rather than promises of production readiness:

- Add a configurable cache capacity and an explicit eviction policy.
- Add integration tests for HTTP status codes, malformed JSON, and rate-limit responses.
- Introduce a storage trait so the in-memory cache can be replaced without changing handlers.
- Add structured request metrics and a documented operational example.
- Review proxy-header trust and make deployment behind a reverse proxy safer by default.

Each item should be discussed in a GitHub issue before implementation. Changes should remain small enough to review and should include tests and documentation.
