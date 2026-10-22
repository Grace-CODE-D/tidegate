# Security Policy

## Supported versions

Only the latest `main` branch is currently supported.

## Reporting a vulnerability

Please do not open a public issue for a suspected security vulnerability. Use GitHub's private vulnerability reporting for this repository when available. If that channel is unavailable, open a minimal issue asking the maintainer for a private contact method and do not include exploit details.

Tidegate is an MVP. Do not expose it directly to untrusted traffic without reviewing proxy trust, rate-limit behavior, cache sizing, and operational logging.

## Supply-chain checks

GitHub Actions runs `cargo-audit` through RustSec on pushes, pull requests, and a weekly schedule. It also runs `scripts/security-scan.sh`, which rejects tracked web and configuration files containing references to `polyfill.io` or `cdn.polyfill.io`. Those domains were associated with a widely reported malicious JavaScript supply-chain incident; projects should remove such references rather than load remote code at runtime.

These checks are targeted safeguards, not a universal malware detector. Keep dependencies updated, review pull requests, avoid executing untrusted build scripts, and use host-level protections and monitoring for production deployments.
