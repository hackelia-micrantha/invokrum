# Release and artifact verification

Invokrum releases are gated prereleases built from version tags on the default branch. The release workflow reuses the complete CI workflow before creating any draft release.

## Release inputs

A release tag must:

- use `vMAJOR.MINOR.PATCH` or a SemVer prerelease suffix;
- match `[workspace.package].version` in `Cargo.toml` exactly;
- point to a commit reachable from `origin/main`;
- pass formatting, Clippy, architecture, security, schema, integration, E2E, golden, and portable-build gates.

The workflow never executes pull-request code with release-write or attestation permissions. Tag builds run only from repository-owned refs on GitHub-hosted runners.

## Produced targets

Each tagged prerelease builds one CLI archive for:

- `x86_64-unknown-linux-gnu` on Ubuntu 24.04;
- `x86_64-apple-darwin` on an Intel macOS runner;
- `x86_64-pc-windows-msvc` on Windows Server 2025.

The local source and persistent-output adapters remain Linux-only. The macOS and Windows artifacts provide parsing, validation, inspection, lock, diff, and stdout workflows; filesystem-backed composition fails closed until native adapters exist.

## Artifact contents

Each archive contains:

- the `invokrum` or `invokrum.exe` binary;
- `README.md`;
- `LICENSE`;
- `SBOM.spdx.json`.

Each release also publishes:

- a standalone target-specific SPDX 2.3 JSON SBOM;
- a SHA-256 checksum file for the archive;
- a GitHub artifact provenance attestation;
- a GitHub SPDX SBOM attestation.

The release remains a draft if any target build, packaging step, attestation, or upload fails. It is published as a prerelease only after every target succeeds.

## Determinism boundary

`scripts/release.py` produces deterministic archive metadata, file ordering, permissions, timestamps, gzip headers, ZIP entries, SPDX ordering, and checksum formatting from fixed inputs. Unit tests build identical tar and ZIP packages twice and require byte equality.

The Rust toolchain and dependency graph are pinned, and the source timestamp comes from the tagged commit. GitHub-hosted operating-system images and native linkers are managed externally, so Invokrum does not yet claim independently reproducible native binary bytes across different runner-image revisions. The archive guarantee is:

> identical binary bytes, source files, SBOM, target, version, and source timestamp produce identical archive and checksum bytes.

A stronger cross-builder reproducible-binary claim requires hermetic toolchains or independently repeated builders and is not currently made.

## Verify a download

Verify the published checksum from the directory containing both files:

```bash
sha256sum --check invokrum-v0.1.0-x86_64-unknown-linux-gnu.tar.gz.sha256
```

On macOS:

```bash
shasum -a 256 --check invokrum-v0.1.0-x86_64-apple-darwin.tar.gz.sha256
```

Verify GitHub provenance and SBOM attestations against this repository:

```bash
gh attestation verify \
  invokrum-v0.1.0-x86_64-unknown-linux-gnu.tar.gz \
  --repo hackelia-micrantha/invokrum
```

Artifact attestations use short-lived keyless signing credentials issued during the trusted workflow. There is no long-lived project signing key to distribute or rotate. Attestations establish workflow provenance for exact artifact digests; they do not authenticate overlay packs or make prompt content semantically safe.

## Release procedure

1. Update the workspace version and release notes in a reviewed pull request.
2. Ensure the default branch is green.
3. Create and push the matching `v...` tag from the intended default-branch commit.
4. Review the draft release, checksums, SBOMs, and attestation links.
5. The workflow publishes the prerelease after all matrix jobs complete.
6. Verify at least one asset using both its checksum and `gh attestation verify`.

A failed run may leave a draft release. Correct the cause and rerun from the same immutable tag only when the source commit remains appropriate. Do not move a published version tag; create a new version for changed source or artifacts.

## Dependency and scanner exceptions

Security gates fail closed by default.

An exception must be introduced through a reviewed pull request and include:

- the advisory, license, source, or secret-scanner finding identifier;
- a linked public issue or private security advisory;
- affected package or exact scanner fingerprint;
- exploitability or false-positive analysis;
- compensating controls;
- an owner;
- a review or expiry condition.

Rust advisory ignores belong in `deny.toml` with an adjacent issue reference. License exceptions must be package-specific rather than broad new allow-list entries when possible. Secret-scan exceptions must use an exact fingerprint or narrow path/rule combination; broad directory exclusions are not acceptable. Workflow action revisions remain full immutable commit SHAs.

The CODEOWNERS file assigns the repository maintainer to workflows, dependency policy, security documentation, and release tooling. Security reports follow [`SECURITY.md`](../SECURITY.md).
