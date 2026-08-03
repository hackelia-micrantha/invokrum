# Usage

> [!IMPORTANT]
> The CLI is currently a scaffold. Commands in this document define the intended v0.1 interface and should not be treated as released behavior until implemented and tested.

## Planned workflow

1. Acquire or author a local overlay pack.
2. Validate the pack and selected profile.
3. Inspect the resolved overlay order.
4. Compose the final context.
5. Optionally generate and verify a lockfile.
6. Pass the exact rendered bytes and digest to the host runtime.

## Validate

```bash
invokrum validate \
  --pack ./pack.yaml \
  --profile secure-review
```

Validation should perform schema, reference, path, cardinality, and compatibility checks without rendering or invoking an agent.

## Inspect

```bash
invokrum inspect \
  --pack ./pack.yaml \
  --profile secure-review \
  --format json
```

Inspection should return the selected overlays, canonical order, rule evaluation, source digests, warnings, and normalized profile identity.

Machine integrations should consume documented JSON output rather than parse human-readable tables.

## Compose

```bash
invokrum compose \
  --pack ./pack.yaml \
  --profile secure-review \
  --output ./context.md
```

When no output path is supplied, composed context may be written to stdout so it can be piped to a host:

```bash
invokrum compose --pack ./pack.yaml --profile secure-review | agent-runtime
```

Hosts must not claim the Invokrum digest if they transform the bytes after composition.

## Lock

```bash
invokrum lock \
  --pack ./pack.yaml \
  --profile secure-review \
  --output ./invokrum.lock
```

The lockfile is expected to capture canonical identities and digests without persisting sensitive variable values.

## Verify

```bash
invokrum verify --lock ./invokrum.lock
```

Verification should distinguish at least:

- pack metadata drift;
- profile-selection drift;
- overlay-content drift;
- variable-input drift when safely representable;
- rendered-output drift;
- unsupported lockfile versions.

## Diff

```bash
invokrum diff ./baseline.lock ./candidate.lock
```

Diff output should explain changes structurally rather than only report a final digest mismatch.

## Exit behavior

The final exit-code contract is not yet defined. The intended principle is:

- zero for success;
- stable non-zero categories for invalid input, validation failure, verification mismatch, and internal error;
- no secret-bearing data in stderr;
- deterministic diagnostic ordering.

## Offline operation

Core validation and composition are intended to run without network access. Downloading, authenticating, or verifying remote packs belongs to the host or a separate installation workflow.