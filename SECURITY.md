# Security policy

## Project status

Invokrum is pre-release software. No production security guarantee is currently made. Security claims must map to implemented and tested controls.

The accepted [threat model and trust boundaries](docs/security/threat-model.md) classify controls as implemented, partial, planned, delegated, or out of scope. The classification is authoritative for current security claims.

## Reporting a vulnerability

Do not disclose suspected vulnerabilities in a public issue, discussion, pull request, or generated fixture.

Use GitHub's private vulnerability reporting feature for this repository when available. Include:

- affected commit or version;
- reproduction steps or proof of concept;
- expected and observed behavior;
- impact and realistic attack path;
- relevant platform and filesystem details;
- whether secrets or untrusted packs are involved;
- suggested remediation, if known.

Do not include live credentials, private keys, access tokens, or third-party confidential data.

## Response expectations

Maintainers will attempt to:

1. acknowledge a complete report;
2. reproduce and classify the issue;
3. coordinate remediation and disclosure;
4. publish an advisory when affected released versions exist.

Response timelines are best-effort while the project is pre-release.

## Security-sensitive areas

Reports are especially useful for:

- pack-root traversal or symlink escape;
- canonicalization inconsistencies;
- verification bypass or lockfile confusion;
- nondeterministic resolution affecting policy order;
- secret leakage through diagnostics or persisted manifests;
- denial of service through malformed or pathological inputs;
- host-adapter behavior that incorrectly preserves an Invokrum attestation after modifying output;
- unsafe parsing or deserialization behavior.

## Trust boundaries

Invokrum is intended to validate declared composition structure and integrity. It does not independently establish that:

- overlay content is semantically safe;
- a prompt is free from injection;
- an invoking user is authorized;
- a host runtime is sandboxed;
- tools or models behave safely;
- a downloaded pack is trustworthy.

Pack acquisition, signature policy, authorization, execution isolation, and audit retention remain host responsibilities unless a future component explicitly defines otherwise. The complete ownership model is documented in the [threat model](docs/security/threat-model.md).

## Supported versions

Until the first release, only the current default branch is considered for security fixes. A supported-version table will be added when versioned releases exist.
