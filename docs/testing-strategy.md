# Testing strategy

Invokrum uses distinct unit, integration, and end-to-end test layers. The layers answer different questions and are not interchangeable.

## Unit tests

Unit tests exercise domain and application behavior with ordinary values, in-memory fakes, or deterministic test doubles.

Required targets include:

- validated identifiers and value objects;
- class ordering and cardinality;
- compatibility and conflict rules;
- deterministic normalization;
- canonical error categories;
- redaction decisions;
- application orchestration with fake ports.

Unit tests must not require network access, user configuration, wall-clock timing, or shared mutable state.

Preferred placement:

```text
crates/<crate>/src/<module>.rs        inline focused unit tests
crates/<crate>/tests/unit/            larger black-box unit suites when needed
```

## Integration tests

Integration tests verify real boundaries working together while remaining bounded and deterministic.

Examples:

- YAML/JSON parser to normalized domain model;
- filesystem adapter plus canonical path enforcement;
- renderer plus canonical serialization;
- hashing plus manifest/lockfile generation;
- CLI argument parsing plus application dispatch;
- adapter conformance against shared contract suites.

Preferred placement:

```text
crates/invokrum-core/tests/integration/
crates/invokrum-cli/tests/integration/
tests/fixtures/
```

Use temporary directories and repository-owned fixtures. Tests must not read or mutate the caller's home directory.

## End-to-end tests

E2E tests invoke the built `invokrum` executable as an external process. They validate the public product contract, not internal functions.

Each supported workflow should eventually cover:

- complete command invocation;
- stdout and stderr;
- exit code;
- generated files;
- rendered prompt bytes;
- resolved manifests and lockfiles;
- deterministic repeated execution;
- meaningful invalid-input behavior.

Preferred placement:

```text
tests/e2e/
tests/fixtures/e2e/
```

E2E tests should use the binary path supplied by Cargo test tooling and should not assume a globally installed executable.

## Adversarial and negative tests

Security-sensitive behavior requires negative coverage for:

- path traversal;
- symlink and pack-root escape;
- duplicate or ambiguous identifiers;
- malformed and oversized inputs;
- unsupported schema or lockfile versions;
- incompatible overlays;
- canonicalization mismatches;
- secret leakage in errors, manifests, and logs;
- interrupted or partial writes where persistence is implemented.

Fuzzing and property tests complement these suites but do not replace deterministic regression tests.

## Test doubles

- Prefer small fakes that implement application-owned ports.
- Use mocks only when interaction ordering or exact calls are part of the contract.
- Avoid mocking domain objects.
- Shared contract tests should verify that every concrete adapter and fake preserves the same semantics.

## Coverage expectations

The project does not use a raw percentage as the sole quality gate. Coverage review focuses on behavior and risk:

- every domain rule has positive and negative unit tests;
- every infrastructure adapter has integration tests;
- every stable CLI workflow has at least one E2E success and failure path;
- every fixed defect gains a regression test at the lowest useful layer;
- security boundaries include adversarial tests.

## CI separation

CI should expose test layers as distinct jobs or clearly separated steps:

1. formatting and linting;
2. unit tests;
3. integration tests;
4. end-to-end tests;
5. security/property/fuzz checks where appropriate.

A required layer must not be silently skipped because no matching tests were discovered. Test commands and directory conventions must remain documented and executable from a clean checkout.

## Pull-request expectations

Each PR that changes behavior must state:

- which test layers changed;
- why a layer is not applicable, if omitted;
- fixtures added or updated;
- deterministic and security-sensitive cases covered;
- residual untested risk.