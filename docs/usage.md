# Usage

Invokrum v0.1 provides an offline CLI for validating packs, resolving profiles, composing exact context bytes, inspecting manifests, generating canonical locks, verifying repository state, comparing locks, and serving a bounded read-only JSON subprocess contract.

## Platform boundary

Pack and overlay reads use the fail-closed local source adapter, which currently supports Linux only. Safe persistent output also supports Linux only. Non-Linux hosts fail closed rather than approximating link and identity guarantees.

The host must provide a stable mount namespace, protect pack and output parent directories from privileged concurrent replacement, and avoid same-device bind aliases below the selected pack root.

## Workflow

1. Acquire or author a local overlay pack through a trusted process.
2. Validate the pack and selected profile.
3. Inspect the resolved order and byte counts.
4. Compose exact context bytes.
5. Generate a canonical lock.
6. Verify current repository state or compare lock revisions.
7. Bind the exact output digest and bytes to the host invocation.

Invokrum performs no network acquisition and does not invoke an agent or model.

## Validate

```bash
invokrum validate \
  --pack ./pack.yaml \
  --profile secure-review
```

`--profile` is optional. Without it, every declaration and profile in the pack is validated. With it, the CLI also verifies that the selected profile exists and has a valid identifier.

Stable JSON output:

```bash
invokrum validate --pack ./pack.yaml --profile secure-review --format json
```

Validation does not read overlay files or render context.

## Compose

Write exact normalized context bytes to stdout:

```bash
invokrum compose --pack ./pack.yaml --profile secure-review > context.md
```

Write through the safe output adapter:

```bash
invokrum compose \
  --pack ./pack.yaml \
  --profile secure-review \
  --output ./context.md
```

An existing output is never replaced implicitly. Explicit replacement requires `--force`:

```bash
invokrum compose \
  --pack ./pack.yaml \
  --profile secure-review \
  --output ./context.md \
  --force
```

Raw context is written without an added newline or wrapper. A host must not claim the Invokrum output digest after modifying those bytes.

## Inspect

Human view:

```bash
invokrum inspect --pack ./pack.yaml --profile secure-review
```

Stable JSON envelope:

```bash
invokrum inspect \
  --pack ./pack.yaml \
  --profile secure-review \
  --format json
```

Inspection reports pack, schema, profile, ordered class/overlay/source identities, per-source lengths, total source bytes, and normalized output bytes. It does not expose variable declarations or values.

## Lock

Emit canonical `invokrum.lock/v1` bytes to stdout:

```bash
invokrum lock --pack ./pack.yaml --profile secure-review > invokrum.lock
```

Write atomically with private permissions:

```bash
invokrum lock \
  --pack ./pack.yaml \
  --profile secure-review \
  --output ./invokrum.lock
```

The lock captures canonical structural and content identities without persisting variable values or variable names. Its unkeyed digests detect corruption and drift; they do not authenticate the publisher or storage channel.

## Verify

Verification requires the lock plus the current pack and profile so current source and output bytes can be recomposed:

```bash
invokrum verify \
  --lock ./invokrum.lock \
  --pack ./pack.yaml \
  --profile secure-review
```

JSON output:

```bash
invokrum verify \
  --lock ./invokrum.lock \
  --pack ./pack.yaml \
  --profile secure-review \
  --format json
```

A valid unchanged repository exits `0`. Detected pack, profile, overlay-set, overlay-content, or rendered-output drift exits `5` and writes the deterministic report to stdout. Malformed, unsupported, or internally inconsistent lock evidence is an integrity error rather than drift.

## Diff

```bash
invokrum diff ./baseline.lock ./candidate.lock
```

```bash
invokrum diff ./baseline.lock ./candidate.lock --format json
```

Identical locks exit `0`. Structural or content differences exit `5` and identify the changed categories. Both inputs must be strict canonical v1 lockfiles.

## Read-only host RPC

`invokrum rpc` reads one `invokrum.host/v1` JSON request from stdin and writes one JSON response to stdout. It supports `capabilities`, `resolve`, and `verify` only. It performs no network access, persistent writes, model invocation, or tool execution.

```bash
printf '%s\n' '{
  "protocol":"invokrum.host/v1",
  "request_id":"resolve-1",
  "operation":"resolve",
  "pack":"./pack.yaml",
  "profile":"secure-review"
}' | invokrum rpc
```

Exact context and canonical lock bytes are returned as canonical padded RFC 4648 base64. Failures after the `rpc` command is accepted also use stdout as one versioned JSON error envelope; stderr remains reserved for command-line parsing or stdout failures.

The complete lifecycle, schemas, capability model, Anthesis design, MCP scope, and CI/editor guidance are documented in [Host adapters and subprocess integration](host-adapters.md).

## Output contracts

- Raw context and canonical lock bytes use stdout only.
- Human and JSON command results use stdout.
- Ordinary CLI errors use stderr only.
- RPC success and operation errors use one JSON response on stdout.
- JSON output has one trailing LF; raw context and lock bytes are not modified.
- Human diagnostics visibly encode control characters.
- No command emits ANSI escape sequences.
- `--no-color` is accepted for automation compatibility but currently has no behavioral effect.
- Diagnostics never include variable values.

## Exit codes

| Code | Meaning |
| ---: | --- |
| `0` | Success, verified state, or identical locks |
| `2` | Invalid command or arguments |
| `3` | Pack, lock, path, local-source, or RPC request/input failure |
| `4` | Schema, domain, profile, compatibility, or composition failure |
| `5` | Verification drift or lock difference for ordinary CLI commands |
| `6` | Stdout or persistent-output failure |
| `7` | Lock integrity, canonicalization, digest, or internal encoding failure |

RPC `verify` reports drift inside a successful protocol response rather than returning exit `5`; the host decides whether drift blocks execution.

## Safe persistent output

On Linux, `--output`:

- rejects parent traversal and symlink parent components;
- requires an existing real parent directory;
- rejects symlink and non-regular targets;
- uses same-directory temporary files with mode `0600`;
- syncs file contents before commit;
- uses an atomic no-clobber hard-link commit when `--force` is absent;
- uses atomic rename replacement only after explicit `--force`;
- checks parent identity and final target type, permissions, and link count;
- cleans temporary files after failure.

The adapter cannot defeat a privileged actor that remaps the output namespace between checks. Protecting the parent directory and namespace remains a host precondition.

## Shell completion

Static completion definitions are maintained in [`completions/`](../completions/):

```bash
# Bash
source completions/invokrum.bash

# Zsh
source completions/_invokrum

# Fish
source completions/invokrum.fish
```

Completions cover commands and options only. They do not inspect packs, discover profiles, access the network, or execute Invokrum during shell initialization.
