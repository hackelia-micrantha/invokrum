# Development guide

## Prerequisites

The repository pins Rust through `rust-toolchain.toml`. A standard Rustup installation should select the expected toolchain automatically.

Recommended local tools:

- Rustup and Cargo;
- Git;
- `cargo fmt`;
- `cargo clippy`;
- a Markdown editor with link checking.

## Workspace

```text
crates/
├── invokrum-core/   policy-neutral domain and engine
└── invokrum-cli/    operator-facing command-line adapter
```

The workspace intentionally starts with two crates. New crates should be added only when they create a meaningful dependency or compatibility boundary.

## Local checks

As implementation grows, the standard local validation contract will be:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo doc --workspace --no-deps
```

Do not add a documented command to required CI until it works from a clean checkout.

## Design expectations

- Keep Anthesis-specific policy outside `invokrum-core`.
- Prefer typed domain states over repeated string validation.
- Avoid nondeterministic map iteration in externally observable output.
- Keep parser-library errors behind stable project error categories.
- Treat path resolution and canonicalization as security-sensitive.
- Separate human-readable CLI output from stable JSON contracts.
- Do not persist or log secret variable values by default.
- Avoid unsafe Rust unless an accepted architecture decision establishes a compelling need and review boundary.

## Testing strategy

Changes should use the smallest relevant combination of:

- unit tests for domain invariants;
- table-driven validation tests;
- golden fixtures for canonical output;
- cross-platform path fixtures;
- property tests for normalization and determinism;
- fuzz targets for parsers and path handling;
- integration tests for CLI exit codes and JSON output;
- compatibility fixtures derived from real consumers such as Anthesis.

Golden files are contracts, not snapshots to update blindly. Review the semantic reason for every change.

## Dependencies

Before adding a dependency, consider:

- whether the standard library is sufficient;
- maintenance and release activity;
- transitive dependency size;
- license compatibility with Apache-2.0;
- unsafe code and native build requirements;
- effect on reproducible static binaries;
- whether it becomes part of a public serialized contract.

## Architecture decisions

Create an ADR under `docs/architecture/` for changes that affect:

- mechanism-versus-policy boundaries;
- canonicalization;
- schema or lockfile compatibility;
- trust boundaries;
- plugin execution;
- network behavior;
- persistent formats;
- public API stability.

## Commit and pull-request scope

Prefer focused changes with explicit validation. A PR should state:

- the problem and intended behavior;
- compatibility impact;
- security impact;
- tests performed;
- documentation affected;
- follow-up work intentionally deferred.