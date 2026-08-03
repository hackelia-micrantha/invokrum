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

## Ownership and response

The repository maintainer identified in [`.github/CODEOWNERS`](.github/CODEOWNERS) owns initial triage for:

- dependency advisories and license policy;
- secret-scanner findings;
- parser, path, filesystem, integrity, and output-boundary reports;
- CI and release workflow changes;
- release-artifact provenance and checksum failures.

Maintainers will attempt to:

1. acknowledge a complete report;
2. reproduce and classify the issue;
3. identify affected commits and releases;
4. coordinate remediation and disclosure;
5. publish an advisory when affected released versions exist.

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
- unsafe parsing or deserialization behavior;
- release artifact, checksum, SBOM, or attestation inconsistencies.

## Gate exceptions

Dependency, license, source, and secret-scanning gates fail closed by default. Exceptions must be narrow, reviewed, owned, and linked to an issue or private advisory. Each exception must record the finding identifier, affected package or scanner fingerprint, rationale, compensating controls, and review or expiry condition.

- Rust advisory ignores belong in `deny.toml` with an adjacent issue reference.
- License exceptions should be package-specific rather than broad allow-list expansion.
- Secret-scan exceptions must use exact fingerprints or narrow rule/path matches; broad directory exclusions are not accepted.
- GitHub Actions remain pinned to full immutable commit SHAs.

The complete release and exception procedure is documented in [release and artifact verification](docs/release.md).

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

Until the first published prerelease, only the current default branch is considered for security fixes. After prereleases begin, the current default branch and latest prerelease receive best-effort fixes; older prereleases are unsupported unless a security advisory states otherwise.
