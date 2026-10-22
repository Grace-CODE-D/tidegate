#!/usr/bin/env bash
set -euo pipefail

# Tidegate is a Rust service today, but this guard protects future web assets and
# documentation from reintroducing the compromised Polyfill.io CDN.
blocked_pattern='polyfill[.]io|cdn[.]polyfill[.]io'

if matches=$(git grep -n -I -i -E "$blocked_pattern" -- \
    '*.html' '*.htm' '*.js' '*.mjs' '*.css' '*.json' '*.yaml' '*.yml' '*.toml' \
    ':!scripts/security-scan.sh' 2>/dev/null); then
    printf '%s\n' "Blocked third-party asset reference detected:" >&2
    printf '%s\n' "$matches" >&2
    printf '%s\n' 'Remove the reference and self-host or pin a reviewed asset with integrity metadata.' >&2
    exit 1
fi

printf '%s\n' 'Security asset scan passed: no blocked Polyfill.io references found.'
