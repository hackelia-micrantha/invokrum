# Overlay pack schema v1

Invokrum accepts YAML and JSON documents using the schema family `invokrum.dev/v1`.

The format adapter is implemented by `invokrum-schema`. It depends inward on the parsing-neutral `invokrum-core` domain model; the core crate does not depend on YAML, JSON, or Serde.

## Top-level fields

- `schema`: required and must equal `invokrum.dev/v1`.
- `id`: required validated pack identifier.
- `classes`: required overlay-class declarations. Precedence is determined by `order`, never file or map iteration order.
- `overlays`: optional pack-relative overlay sources and incompatibility declarations.
- `profiles`: optional named selections grouped by class.
- `variables`: optional declared variable names with `public` or `secret` sensitivity.

Unknown fields are rejected at every object boundary. An unsupported schema family is identified before strict v1 field decoding, so an unambiguous future-version document receives an unsupported-version error rather than a misleading unknown-field error.

## Structural preflight

Before schema-family negotiation, the adapter recursively walks the complete serialized value and rejects duplicate keys in every JSON object or YAML mapping. This includes map-like fields such as `profiles[].selections`, not only named DTO fields.

The ordering is deliberate:

1. the complete document must be syntactically valid and structurally unambiguous;
2. the schema family is negotiated;
3. strict v1 DTO decoding and domain validation run.

A future-version document containing duplicate keys therefore fails with a duplicate-mapping error rather than bypassing ambiguity checks through version preflight. JSON Schema remains a secondary contract because duplicate object keys may already have been collapsed before a JSON Schema validator receives the value.

Parser-facing error text is bounded before it becomes a public `SchemaError`, and unsupported schema names are truncated to a bounded representation.

## Accepted YAML subset

The v1 YAML surface is intentionally narrower than the complete YAML language. It supports one document containing ordinary string-keyed mappings, sequences, and scalar values. One leading `---` document-start marker is accepted.

The following features fail closed before DTO mapping:

- multiple YAML documents;
- YAML directives such as `%YAML` and `%TAG`;
- explicit document-end markers (`...`);
- anchors (`&name`) and aliases (`*name`);
- merge keys (`<<`);
- explicit tags (`!tag`);
- literal and folded block scalars (`|` and `>`);
- explicit complex mapping keys (`?`).

Reserved YAML indicators inside quoted scalar text are not interpreted by the subset scanner. Values must still satisfy the v1 field and domain constraints after decoding.

This subset prevents parser expansion and parser-version differences from changing pack meaning. It also keeps recursive alias expansion out of the v1 denial-of-service surface.

## Classes and cardinality

Each class declares:

- `id`: class identifier;
- `order`: unsigned 32-bit precedence value;
- `minimum`: unsigned 32-bit minimum selection count;
- `maximum`: optional unsigned 32-bit maximum selection count.

Omitting `maximum`, or setting it to `null`, means the class is unbounded above. The domain model rejects a maximum lower than the minimum and rejects duplicate class order values.

## Paths

Overlay `source` values use a portable forward-slash grammar rather than host-native path parsing. Values must be relative and must not contain:

- a leading or trailing `/`;
- backslashes or platform prefixes;
- `:` characters;
- empty path segments;
- `.` or `..` segments.

This lexical validation is platform-independent. Canonical pack-root resolution and filesystem link policy remain part of the composition/filesystem work in issue #4.

## Strict collections

- Duplicate keys are rejected at every JSON object and YAML mapping boundary.
- `incompatible_with` values must be unique; duplicates are rejected rather than silently deduplicated.
- Profile selection arrays must not contain duplicate overlay identifiers.
- Duplicate class, overlay, profile, and variable identifiers are rejected by the domain aggregate.
- Profile selection keys must be valid class identifiers.

## Deterministic normalization

Equivalent documents normalize independently of incidental declaration order:

- classes are ordered by explicit `order`;
- overlays are ordered by identifier;
- profiles are ordered by identifier;
- variables are ordered by identifier;
- selection maps and incompatibility sets use deterministic key/value ordering;
- explicit profile selection order within a class is preserved because it may become composition-significant.

`to_normalized_json` emits the validated normalized representation rather than re-serializing the original input document.

## Compatibility

The v1 reader fails closed on any other schema family. Additive fields are not accepted until a reader version explicitly supports them. Breaking semantic, structural, YAML-subset, or normalization changes require a new schema family. Deprecation must be documented before support is removed.

## Validation layers

1. JSON/YAML syntax, recursive duplicate-key detection, single-document enforcement, and accepted-YAML-subset checks.
2. Schema-family compatibility.
3. Strict v1 DTO decoding with unknown-field rejection.
4. Domain validation for identifiers, portable paths, unique declarations, references, class membership, and cardinality.
5. Deterministic normalization for downstream composition and attestation.

## Resource limits

Rejecting aliases and block-scalar expansion removes two unbounded parser features, but it is not a complete denial-of-service policy. Maximum serialized document size, nesting depth, collection counts, overlay file size, and rendered output growth remain explicit work in issues #4 and #20. Hosts should bound input bytes before calling the adapter until those contracts are implemented.

The machine-readable schema is [`schemas/invokrum-pack-v1.schema.json`](../schemas/invokrum-pack-v1.schema.json). Equivalent YAML and JSON fixtures are under `tests/fixtures/schema/`.
