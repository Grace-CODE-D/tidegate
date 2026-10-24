# Tidegate Wave Program Work Plan

Tidegate is a small Stellar-focused Rust project with two parts: a Soroban gateway-registry contract and an HTTP gateway that connects to Stellar RPC. The service provides a bounded in-memory JSON cache, per-client rate limiting, health reporting, operational metrics, dependency auditing, input validation, and a targeted supply-chain asset scan. The project is intentionally compact, but it is structured so that contributors can improve it through focused, reviewable changes rather than through broad rewrites.

The work I would post for a Wave Program would be organized around the real needs of the project: making the current service safer, easier to test, easier to operate, and more useful as a foundation for small services. I would not post artificial renaming tasks, cosmetic changes designed only to create activity, or issues that ask contributors to copy another project. Each issue would correspond to a genuine engineering, documentation, testing, security, or maintenance need, with a defined scope and acceptance criteria.

The repository is a Rust workspace using Axum for HTTP routing, Tokio for asynchronous execution, Serde and Serde JSON for request and response data, Tower HTTP tracing, `reqwest` for the read-only Stellar JSON-RPC client, and `soroban-sdk` for the contract. The current API includes `/healthz`, `/metrics`, `/v1/stellar/ledger/latest`, and cache routes under `/v1/cache`. The current registry contract supports authenticated gateway registration, updates, deactivation, and lookup. The cache is deliberately ephemeral and bounded. This makes Tidegate appropriate for practical Stellar infrastructure experiments while creating a clear set of improvements for contributors to implement.

## Bug fixes and reliability improvements

The first category of Wave issues would be bug fixes. These issues are useful because they improve behavior that already exists, and they can usually be written with precise reproduction steps and acceptance tests.

One issue would cover malformed JSON request handling. The current write endpoint accepts a JSON body containing a `value` field. A contributor could verify what happens when the request body is empty, when the JSON is syntactically invalid, when the `value` field is missing, or when the content type does not describe JSON. The expected behavior should be documented and tested. The service should return a stable client-error response with a structured error body instead of an opaque framework-generated response. The scope would include handler behavior, tests, and API documentation, but would not include a general redesign of the error system.

Another issue would cover invalid cache keys. Contributors could define and implement rules for empty keys, excessively long keys, keys containing unsafe path characters, and keys that create ambiguous URL behavior. The task would require a small validation function, clear status codes, tests for accepted and rejected values, and an explanation in the README. This would protect the service from accidental misuse while keeping the API easy to understand.

A separate reliability issue would investigate the interaction between cache capacity and update behavior. Tidegate currently uses a bounded recency queue: adding a new key at capacity removes the oldest entry, while updating an existing key refreshes its position. A contributor could review this behavior, add property-style cases, and make sure deletion, replacement, and eviction cannot leave stale keys in the ordering queue. The acceptance criteria would include no duplicate queue entries, no cache length above capacity, and consistent results after repeated updates and deletes.

Rate limiting would provide another group of bug-fix issues. Contributors could test what happens when the configured request limit is zero, when the window duration is very short, when the same client sends requests through different forwarding-header formats, and when multiple tasks access the same limit bucket concurrently. The project should define whether zero is rejected, normalized, or treated as a safe default. It should also document that `X-Forwarded-For` must only be trusted behind a proxy that overwrites the header. The implementation should avoid panics and should return a predictable `429 Too Many Requests` response with a useful retry hint.

An operational bug-fix issue could address cleanup of old rate-limit buckets. The current design keeps buckets in memory for the process lifetime. In a service with many client addresses, that map could grow even when traffic stops. A contributor could implement bounded cleanup or periodic expiration, with a test proving that inactive buckets are removed without affecting active clients. The scope would be limited to rate-limit state management; it would not attempt to build a distributed limiter.

Another issue would improve graceful shutdown. The contributor would add a shutdown signal path for common operating-system termination signals, make sure the server stops accepting new requests, and document the expected behavior. The task would include a small test or a manual verification procedure, but it would not require a container orchestration platform. This would make the service more suitable for deployment while keeping the code understandable.

## New features

The second category would be small, useful features. Each feature would be independently valuable and designed to fit within a sprint cycle.

The most important planned feature is pluggable cache storage. Tidegate currently couples its handlers to an in-memory cache. A contributor could introduce a small storage abstraction that defines operations such as get, insert, and remove. The existing bounded in-memory store would remain the default implementation. The issue would require tests against the abstraction, documentation of synchronization and error behavior, and no new database dependency. A later issue could add a durable backend, but the first issue should focus on clean separation rather than expanding the infrastructure too quickly.

A follow-up feature could add an optional SQLite-backed cache for local development. This would be scoped carefully: one supported backend, explicit configuration, migration-free key/value storage, and tests that run against a temporary database. The issue would state that durability is optional and that the in-memory mode remains the default. It would also require documentation about performance, locking, and data lifetime. Contributors would not be asked to design a general database framework.

Another feature would add cache expiration. Each stored entry could optionally include a time-to-live, after which reads return a miss and the entry becomes eligible for removal. The issue would need to define whether expiration is checked lazily during reads, eagerly by a cleanup task, or both. It would include tests using an injectable clock or a deterministic time abstraction rather than flaky sleeps. This would be a meaningful enhancement to the cache while remaining isolated from unrelated gateway functionality.

Metrics could be expanded beyond the current JSON counters. A contributor could add a Prometheus-compatible text representation under a separate endpoint or content negotiation mode. The issue would define metric names, labels, and cardinality limits. In particular, it should prohibit using arbitrary cache keys or client addresses as labels because that could create unbounded metric cardinality. The work would include examples, a test for the output format, and an operational note explaining that the current metrics are process-local.

A request correlation feature would be another useful issue. Tidegate could accept a valid incoming request identifier or generate one, return it in a response header, and include it in tracing fields. The issue would define length limits, allowed characters, behavior for malformed identifiers, and test coverage. This would make debugging easier without creating a complicated distributed tracing system.

A configuration validation feature could replace silent fallback behavior with startup diagnostics. At present, invalid numeric environment values use defaults. Contributors could design a clear policy for values that are malformed, zero, negative, or too large. One possible outcome would be to fail startup for invalid security-sensitive values while keeping defaults for omitted values. The issue would require a configuration struct, unit tests, and clear log messages. It would not change unrelated request handling.

Another feature would add a read-only cache listing endpoint for local development. Because listing keys can expose sensitive information, the issue would need to specify that it is disabled by default or protected by an explicit development-only setting. This would be a good example of a feature that requires security analysis, not just implementation. The contributor would document why it should not be enabled on an untrusted public listener.

## Testing and quality work

Testing issues would be a major part of the Wave portfolio. A small service becomes easier to extend when behavior is captured in deterministic tests.

One issue would expand HTTP integration coverage. The existing tests cover a basic write, read, and metrics flow. The next scope would include missing keys, deletion, repeated deletion, malformed JSON, invalid paths, oversized inputs, rate-limit rejection, and cache eviction through the public router. The tests would use in-process requests and would not depend on an external server or network service. Acceptance criteria would require stable status codes and response bodies for each case.

A concurrency testing issue could exercise simultaneous writes, reads, and deletes against the same cache. The goal would not be to prove formal linearizability, but to catch panics, deadlocks, data races, and violations of the configured capacity. Contributors could use Tokio tasks and a bounded workload. The issue would require a clear explanation of what the test proves and what it does not prove. If a race detector or specialized tool is introduced, the workflow should document how maintainers can reproduce it locally.

A fuzzing issue would target JSON request parsing, cache-key validation, and query extraction. The contributor could use a suitable Rust fuzzing framework or start with property-based tests if full fuzzing is too heavy for the repository. The issue would require a small target, a reproducible command, and a policy for storing regression cases. It would not ask contributors to fuzz the entire operating system or claim that fuzzing finds all vulnerabilities.

A performance benchmark issue would measure cache reads, writes, eviction, and rate-limit checks under a defined workload. The benchmark should report enough context to be useful but should not set brittle pass/fail thresholds tied to a particular runner. The goal would be to detect major regressions and understand allocation behavior. The contributor would document the hardware and command used for sample results.

A test-fixture issue could introduce reusable helpers for constructing application state, sending requests, decoding JSON, and asserting error responses. This would reduce duplication as coverage grows. The acceptance criteria would require that the helpers remain readable and do not hide important assertions. This type of work is small but valuable because it lowers the cost of future feature contributions.

A continuous-integration issue could add a minimum supported Rust version check, dependency caching, and a separate job for documentation builds. The contributor would need to choose a supported version based on actual dependency requirements, update the README, and ensure the workflow remains reliable for pull requests from forks. The task would be about reproducible project maintenance, not about adding unnecessary CI complexity.

## Documentation and developer experience

Documentation issues would help contributors understand and safely use the service. These would include both user-facing documentation and maintainer-facing technical notes.

One issue would create an API reference with complete request and response examples. It would document headers, status codes, error bodies, key constraints, and rate-limit behavior. The contributor would verify every example against the running service or tests. The goal would be to make the API understandable without reading the source.

Another issue would add a deployment guide for a reverse-proxy setup. It would explain binding to a non-public interface, forwarding headers, TLS termination, request-size limits, health checks, and the danger of trusting arbitrary `X-Forwarded-For` values. The guide would clearly separate development instructions from production recommendations. It would not claim that Tidegate itself provides TLS or a complete security boundary.

An architecture document would explain the state model, lock boundaries, middleware ordering, cache eviction semantics, metrics limitations, and extension points. This would be especially helpful for contributors working on pluggable storage or concurrency. The document should include design decisions and rejected alternatives where those choices affect future work.

A troubleshooting issue could document common startup failures, port conflicts, invalid configuration, rate-limit surprises, empty caches after restart, and failed CI checks. The contributor would reproduce each scenario and provide a concise resolution. This kind of work improves contributor experience and reduces repeated maintainer questions.

A documentation-quality issue could review all commands for correctness, remove stale examples, add expected output where useful, and ensure that security limitations are not hidden. The task would be small and suitable for a contributor who wants to improve technical writing while still making a measurable contribution.

## Security and supply-chain work

Security work would be posted cautiously and would focus on concrete controls rather than promises to detect all malware. The repository already runs RustSec dependency auditing and a targeted guard for known Polyfill.io CDN indicators. Future issues could improve those controls.

One dependency-maintenance issue would review the Cargo dependency graph, update stale packages, and document any advisory decisions. The contributor would run `cargo audit`, review changelogs where needed, and avoid blindly applying upgrades that change behavior. If an advisory does not affect Tidegate, the issue would require a written rationale and a maintainer-approved exception rather than a silent ignore.

A supply-chain issue could add lockfile and dependency policy documentation. It might define when new dependencies are justified, how licenses are reviewed, how transitive dependencies are audited, and how maintainers respond to a newly published advisory. The goal would be to make future dependency additions deliberate and reviewable.

The Polyfill guard could be improved to detect risky remote JavaScript patterns in future web assets, but the scope would be carefully defined to avoid false positives. For example, a contributor could add fixtures showing blocked domains, safe local assets, and approved integrity-pinned assets. They could improve the scanner's error messages and ensure that the scanner itself does not trigger false positives when the security documentation mentions the incident. The issue would explicitly state that this is a targeted supply-chain check, not a universal malware scanner.

A secure-input issue could add request-body size limits and key-length limits. This would reduce memory-exhaustion risk and make the service safer when exposed to untrusted clients. The contributor would need to choose defaults, make them configurable where appropriate, test rejection behavior, and document how the limits interact with the cache capacity. This is a concrete security improvement with clear boundaries.

A secrets and repository-hygiene issue could add secret-scanning configuration, review the GitHub Actions permissions, and ensure workflows use least privilege. The contributor would inspect whether actions are pinned appropriately, document the trade-offs, and avoid introducing credentials into tests. This work would be valuable because supply-chain security includes workflow security, not only application dependencies.

A threat-model documentation issue could describe the assets Tidegate protects, its trust boundaries, likely misuse cases, and out-of-scope threats. It would cover untrusted clients, reverse proxies, dependency updates, cache contents, logs, and deployment hosts. The outcome would not be a claim of certification; it would be a practical reference for maintainers and contributors.

## Issue quality and contributor expectations

Every issue posted for a Wave sprint would include a clear title, a short problem statement, a defined scope, acceptance criteria, relevant tests, and any known constraints. Issues would identify whether the work is a bug fix, feature, documentation task, test improvement, maintenance task, or security improvement. If design discussion is needed, the issue would say so before implementation begins.

I would avoid combining unrelated requests in one issue. For example, a contributor should not be asked to add a new storage backend, redesign error handling, rewrite the README, and change the deployment model in one task. Breaking work into focused issues makes it easier to estimate complexity, review pull requests, and award credit for actual outcomes.

Contributors would be expected to read the repository guidance, discuss substantial changes before coding, create a focused branch, run formatting and tests, and link their pull request to the assigned issue using a closing keyword. Pull requests should explain what changed, why it changed, how it was tested, and what limitations remain. Maintainers would respond to questions, review promptly when possible, and give actionable feedback.

Issues would also be kept honest about project maturity. Tidegate is a small MVP, not a finished production platform. A good contribution may improve one narrow behavior without solving every related problem. Descriptions would avoid inflated claims such as “protects against all malware,” “production-ready database,” or “fully distributed rate limiting” unless the implementation genuinely supports those claims.

## Sprint suitability and acceptance

The best Wave issues for Tidegate are those that can be completed within a sprint, have a measurable result, and leave the repository better for the next contributor. Examples include adding deterministic HTTP tests, implementing cache expiration with a testable clock, improving configuration validation, documenting reverse-proxy trust, adding a pluggable storage interface, or updating vulnerable dependencies with a clear review record.

Before marking an issue complete, the maintainer would verify the acceptance criteria, review the code and tests, run the relevant CI checks, and confirm that the documentation matches the behavior. A merged pull request would not automatically mean the work was successful if it failed to address the stated problem. Conversely, a modest but well-tested fix would be valuable even if it added only a small amount of code.

The project would use the Wave Program to support genuine open-source collaboration. Contributors would be selected for issues that match their skills and availability, not encouraged to submit generic applications or artificial changes. Communication would remain on the GitHub issue and pull request as much as possible, and contributors would be encouraged to ask questions early rather than disappearing when blocked.

This work plan gives Tidegate a practical set of contribution paths across bug fixing, features, testing, documentation, maintenance, and security. It also keeps the scope transparent: the project can become more capable through reviewed incremental work, but no single Wave cycle or contribution should be presented as proof that the service is immune to vulnerabilities or ready for every production environment.

## Issue lifecycle and maintainer process

Issues would move through a simple lifecycle. First, the maintainer would reproduce or define the problem and confirm that it belongs in Tidegate rather than in an external dependency. Next, the issue would be written with scope, acceptance criteria, testing expectations, and a suggested complexity level. During a Wave sprint, contributors would apply with a short explanation of their approach and relevant experience. Work would begin only after assignment so that two contributors do not unknowingly implement the same change.

During implementation, the contributor would keep the issue updated with design questions and progress. If the original approach turns out to be too large, the maintainer and contributor could split the work or narrow the acceptance criteria transparently. Pull requests should remain focused and should not include unrelated generated files or cosmetic rewrites. Review comments would be addressed in follow-up commits, with tests updated when behavior changes.

After review, the maintainer would verify the result on the current branch, check the linked issue and pull request, and merge only when the stated behavior is present. The issue would then be closed with a short note describing what shipped and any follow-up issue that remains. This process creates a useful public record for future contributors and gives Wave reviewers evidence that the repository is active, maintained, and organized around real work rather than placeholder tasks.
