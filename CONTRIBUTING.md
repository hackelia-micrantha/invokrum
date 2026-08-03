# Contributing to Invokrum

Invokrum welcomes focused issues, design review, documentation improvements, adversarial fixtures, and implementation contributions.

The project is early. Architecture, deterministic behavior, compatibility, and trust boundaries take precedence over feature breadth.

## Before contributing

1. Read the [purpose](docs/purpose.md), [architecture](docs/architecture/README.md), and [development guide](docs/development.md).
2. Search existing issues and pull requests.
3. Use an issue for behavior that changes schemas, persistent formats, security boundaries, or public APIs.
4. Keep consumer policy outside the generic engine unless it is clearly an example or compatibility fixture.

Small typo and documentation corrections do not require prior discussion.

## Development workflow

```bash
git clone https://github.com/hackelia-micrantha/invokrum.git
cd invokrum
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

Some commands will become meaningful as implementation lands. Required checks must remain executable from a clean checkout.

## Pull requests

A pull request should include:

- a concise problem statement;
- the chosen approach and notable alternatives;
- compatibility impact;
- security or trust-boundary impact;
- tests and validation performed;
- documentation changes;
- deferred follow-up work.

Keep PRs reviewable. Avoid combining schema redesign, core implementation, CLI presentation, and unrelated repository cleanup unless they are inseparable.

## Definition of done

For externally observable behavior:

- implementation is tested;
- deterministic output has stable fixtures where relevant;
- errors are actionable and do not expose sensitive values;
- public JSON, exit-code, schema, or lockfile changes are documented;
- security-sensitive path or canonicalization behavior has negative tests;
- user-facing documentation reflects actual implementation status.

## Compatibility-sensitive changes

Changes to these surfaces require explicit review and release notes:

- pack schema;
- lockfile and manifest formats;
- canonicalization rules;
- CLI JSON output and exit codes;
- public Rust API;
- adapter envelopes;
- path and symlink policy.

## Architecture decisions

Use an ADR for durable decisions affecting architecture, security, compatibility, or project boundaries. ADRs should state context, decision, consequences, alternatives, and status.

## Security reports

Do not open public issues for suspected vulnerabilities. Follow [SECURITY.md](SECURITY.md).

## Conduct

Participation is governed by the [Code of Conduct](CODE_OF_CONDUCT.md).

By contributing, you agree that your contributions are licensed under Apache-2.0.