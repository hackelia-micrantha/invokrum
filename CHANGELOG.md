# Changelog

All notable user-visible changes to Invokrum will be documented in this file.

The format is based on Keep a Changelog principles, and versioned releases follow Semantic Versioning for the currently experimental pre-1.0 contracts.

## 0.1.0 — prerelease candidate

### Added

- Typed Rust domain model for packs, ordered classes, overlays, profiles, variables, compatibility rules, and resolved manifests.
- Strict bounded `invokrum.dev/v1` YAML and JSON schema adapter with duplicate-key rejection, conservative YAML feature policy, deterministic normalization, and resource limits.
- Deterministic fail-closed composition with explicit cardinality and incompatibility validation.
- Linux local-filesystem source adapter with root identity pinning, traversal and link rejection, stable opened-byte reads, mutation detection, and bounded input handling.
- Canonical `invokrum.lock/v1` evidence, SHA-256 content addressing, strict lock decoding, verification, and deterministic drift categories.
- Operator CLI commands: `validate`, `compose`, `inspect`, `lock`, `verify`, and `diff`.
- Versioned `invokrum.cli/v1` machine output, stable exit categories, strict stdout/stderr separation, terminal-safe diagnostics, shell completions, and Linux safe persistent output.
- Governed code-review example pack with byte-exact context, manifest, and lock golden artifacts.
- Transport-neutral `invokrum-host` facade and read-only `invokrum rpc` subprocess protocol using `invokrum.host/v1` request and response envelopes.
- Layered unit, integration, end-to-end, golden, architecture, adversarial, and compatibility-contract test gates.
- Cross-platform CLI builds for Linux x64, Intel macOS, and Windows x64.
- Deterministic release packaging, SHA-256 checksums, SPDX SBOMs, GitHub provenance attestations, and SBOM attestations.
- Threat model, trust-boundary documentation, architecture decisions, governance, contribution guidance, and security reporting process.

### Security

- Untrusted schema documents are bounded by serialized size, nesting depth, and declaration counts before domain aggregate construction.
- Ambiguous JSON/YAML mappings and unsupported YAML expansion features fail closed.
- Overlay source reads reject traversal, symlinks, hard links, root replacement, non-regular files, and relevant identity changes on Linux.
- Lockfiles exclude variable values and are accepted only in exact canonical form.
- CLI and RPC machine envelopes are versioned, bounded, and reject duplicate or unknown fields.
- CI uses immutable action revisions, least-privilege permissions, dependency and license policy, and full-history secret scanning.

### Known limitations

- Secure filesystem-backed composition and persistent output currently support Linux only. macOS and Windows binaries support portable parsing, validation, inspection, lock decoding, diffing, RPC capability discovery, and other operations that do not require the Linux source adapter.
- Integrity and GitHub build provenance do not authenticate third-party overlay-pack publishers.
- Structural validation does not establish semantic safety, authorization, approval, or sandboxing.
- Remote pack fetching, plugin execution, publisher signatures, MCP runtime integration, and Anthesis conformance fixtures are not part of v0.1.
- Public Rust, CLI machine, host RPC, schema, manifest, and lock contracts remain pre-1.0 and may evolve with explicit changelog entries.

## Versioning notes

Before `1.0.0`, minor releases may include breaking changes to experimental interfaces. Breaking changes must still be explicit in this changelog and in release notes.

Compatibility-sensitive surfaces include:

- overlay-pack schemas;
- canonicalization and rendering rules;
- resolved manifest and lockfile formats;
- machine-readable CLI output;
- exit codes;
- public Rust APIs;
- adapter request and response envelopes.

Documentation-only corrections that do not alter a public contract may be grouped under the next release.
