# Overlay pack schema v1

Invokrum accepts YAML and JSON documents using the schema family `invokrum.dev/v1`.

## Top-level fields

- `schema`: required and must equal `invokrum.dev/v1`.
- `id`: validated pack identifier.
- `classes`: ordered overlay-class declarations. Precedence is determined by `order`, never file or map iteration order.
- `overlays`: pack-relative overlay sources and optional incompatibility declarations.
- `profiles`: named selections grouped by class.
- `variables`: declared variable names with `public` or `secret` sensitivity.

Unknown fields are rejected at every object boundary. Absolute paths and parent-directory segments are rejected by the domain model. Network references are not part of v1.

## Compatibility

The v1 reader fails closed on any other schema family. Additive fields are not accepted until a reader version explicitly supports them. Breaking semantic or structural changes require a new schema family. Deprecation must be documented before support is removed.

## Validation layers

1. JSON/YAML syntax and strict field decoding.
2. Schema-family compatibility.
3. Domain validation for identifiers, paths, unique declarations, references, class membership, and cardinality.
4. Deterministic normalization for downstream composition and attestation.

The machine-readable schema is [`schemas/invokrum-pack-v1.schema.json`](../schemas/invokrum-pack-v1.schema.json). Equivalent YAML and JSON fixtures are under `tests/fixtures/schema/`.
