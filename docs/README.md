# Invokrum documentation

Invokrum has an implemented domain model, bounded strict v1 YAML/JSON schema adapter, deterministic composition use case, fail-closed Linux filesystem adapter, canonical lockfile and verification adapter, read-only host façade and JSON subprocess contract, operator CLI, Linux safe-output adapter, layered test baseline, dependency and secret gates, and a gated cross-platform prerelease workflow.

Documentation distinguishes between **accepted design**, **planned interfaces**, and **implemented behavior** so examples do not imply unsupported functionality.

## Start here

- [Purpose and scope](purpose.md)
- [Use cases](use-cases.md)
- [Architecture](architecture/README.md)
- [Deterministic composition and filesystem contract](composition-and-filesystem.md)
- [Integrity, canonical manifests, and lockfiles](integrity-and-lockfiles.md)
- [Host adapters and subprocess integration](host-adapters.md)
- [Threat model and trust boundaries](security/threat-model.md)
- [Fuzzing strategy](security/fuzzing.md)
- [V1 schema contract](schema-v1.md)
- [Configuration model](configuration.md)
- [Usage model](usage.md)
- [Governed code-review example](../examples/governed-code-review/README.md)
- [Release and artifact verification](release.md)
- [Development guide](development.md)
- [Roadmap](roadmap.md)

## Project policies

- [Contributing](../CONTRIBUTING.md)
- [Security policy](../SECURITY.md)
- [Support](../SUPPORT.md)
- [Code of conduct](../CODE_OF_CONDUCT.md)
- [License](../LICENSE)

## Status vocabulary

Documentation uses these terms deliberately:

- **Accepted** — recorded in an accepted architecture decision or repository policy.
- **Planned** — intended for a future milestone but not yet available.
- **Implemented** — present in the repository and covered by executable validation.
- **Experimental** — implemented but not yet compatibility-stable.

Security controls additionally use **Partial**, **Delegated**, and **Out of scope** as defined by the [threat model](security/threat-model.md).

When documentation and implementation diverge, implementation and executable contracts are authoritative; the discrepancy should be reported as documentation drift.
