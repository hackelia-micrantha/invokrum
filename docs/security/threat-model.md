# Threat model and trust boundaries

**Status:** Accepted security architecture for the v0.1 implementation line  
**Scope:** Local YAML/JSON packs, local overlay files, deterministic composition, manifests, digests, lockfiles, CLI adapters, output sinks, and host integrations  
**Review trigger:** Any change to schema, path resolution, rendering, hashing, persistence, acquisition, plugins, host adapters, output handling, or secret handling

Invokrum treats pack metadata and overlay content as untrusted input. Validation can establish structural consistency, deterministic resolution, and byte-level integrity. It cannot establish that prompt text is truthful, non-malicious, authorized, or safe for a particular model or tool runtime.

## Security objectives

Invokrum should:

1. reject malformed, ambiguous, unsupported, or inconsistent packs;
2. resolve local inputs deterministically without implicit network access;
3. prevent declared paths from escaping the selected pack root;
4. preserve exact input and output identities for manifests, digests, and lockfiles;
5. avoid leaking sensitive variable values through diagnostics or persisted evidence;
6. avoid unsafe output replacement, terminal injection, and misleading host claims;
7. make Invokrum, pack-author, operator, and host responsibilities explicit;
8. fail closed when a required security decision is unknown or unverifiable.

Invokrum does not determine whether overlay prose is semantically safe, authorize users or actions, sandbox runtimes, authenticate publishers, guarantee model behavior, or preserve an attestation after a host modifies rendered bytes.

## Assets

- Pack metadata, schema family, identifiers, classes, overlays, profiles, and variables.
- Overlay files and their exact bytes.
- Selected profile and caller-supplied variable values.
- Sensitive variable values and derived rendered content.
- Canonical ordering and normalized resolution results.
- Rendered context bytes, manifests, digests, lockfiles, diagnostics, and exit status.
- Operator-selected output paths and destination permissions.
- Host claims that an execution used a verified Invokrum result.
- Release artifacts, dependency lockfiles, and published schemas.

## Actors

- **Pack author:** defines metadata, content, profiles, and compatibility declarations.
- **Operator:** chooses a pack, profile, variables, command, and output destination.
- **Host integrator:** acquires packs, wires adapters, authorizes use, and invokes a runtime.
- **Agent or tool runtime:** consumes rendered context and may have external capabilities.
- **Local attacker:** influences files, links, process timing, mount namespace, terminal text, or output destinations.
- **Supply-chain attacker:** compromises a dependency, artifact, registry, or pack source.
- **Malicious publisher:** supplies structurally valid but harmful prompt content.

## Entry points

- YAML or JSON pack document bytes supplied by a file, standard input, API, or host adapter.
- Pack root and overlay source paths selected by the operator or declared by the pack.
- Overlay file bytes and filesystem metadata observed during composition.
- Profile identifier and explicit variable values supplied by an operator or host.
- CLI arguments, environment used by the delivery layer, standard input, and working directory.
- Output paths, stdout, stderr, manifests, lockfiles, and host evidence sinks.
- Host-adapter requests and the exact rendered bytes handed to a model or tool runtime.
- Acquired binaries, schemas, dependencies, pack archives, and future plugin artifacts.

## Trust boundaries

```mermaid
flowchart LR
  Source[Remote source or local authoring] -->|untrusted acquisition| Root[Selected local pack root]
  Root -->|untrusted bytes and paths| FS[Filesystem adapter]
  Root -->|untrusted YAML or JSON| Schema[invokrum-schema]
  Schema -->|validated domain values| Core[invokrum-core]
  FS -->|bounded local bytes| App[Composition use case]
  Core --> App
  Secrets[Explicit variable inputs] -->|sensitive values| App
  App -->|rendered bytes and manifest| Host[Host adapter]
  Host -->|authorization and sandbox boundary| Runtime[Agent or tool runtime]
  App -->|diagnostics and files| Output[CLI and evidence sinks]
```

### Boundary A — acquisition to local pack root

A downloaded or copied pack is untrusted. V0.1 composition performs no acquisition and no implicit network access. The host owns source authentication, transport security, signature policy, pinning, quarantine, and update decisions.

### Boundary B — filesystem to filesystem adapter

Pack paths and filesystem entries are attacker-controlled. The adapter must establish one canonical root, apply explicit symlink and link policies, reject escapes and unacceptable file types, and avoid check-then-use ambiguity. Lexical validation alone is not containment.

The v0.1 `invokrum-fs` adapter supports Linux only. It rejects symbolic links at every declared component, hard-linked files, non-regular files, device-boundary crossings, canonical root escapes, and changed file identity or metadata. It validates the opened `/proc/self/fd` target and returns bytes from that same opened file. Non-Linux platforms fail closed as unsupported. The detailed contract is documented in [`docs/composition-and-filesystem.md`](../composition-and-filesystem.md).

Canonical containment still cannot prove provenance against a privileged actor that can remap mounts or the namespace during composition. The host must provide a stable mount namespace and protect the selected root from privileged concurrent manipulation.

### Boundary C — serialized document to schema adapter

YAML and JSON are attacker-controlled. The schema adapter owns syntax decoding, schema-family negotiation, unknown-field rejection, duplicate-key behavior, an explicitly accepted YAML feature subset, format limits, and mapping into validated domain values. JSON Schema validation alone cannot detect duplicate object keys after a parser has collapsed them.

### Boundary D — schema adapter to core domain

Only validated value objects and aggregates cross into core. Core owns deterministic ordering, references, class membership, cardinality, compatibility invariants, and stable domain failures. It performs no I/O and parses no external formats.

### Boundary E — variables to composition

Variable values are explicit caller inputs and may be sensitive. Domain and application logic must not read ambient environment state. Sensitive values require redaction and are excluded from persistent evidence by default.

### Boundary F — rendered result to host and runtime

Rendered prompt content remains untrusted text. The host owns authorization, runtime selection, sandboxing, capabilities, network policy, approvals, evidence retention, and binding the exact digest to execution bytes. Any byte transformation invalidates the original digest claim.

### Boundary G — application result to CLI and evidence sinks

Output destinations may be attacker-controlled or security-sensitive. The delivery layer owns terminal-safe diagnostics, machine-output encoding, no-follow and overwrite policy, destination permissions, atomic replacement where applicable, cleanup after failure, and preventing partial or stale output from being mistaken for a successful result.

## Assumptions

- The operating system, Rust runtime, and future cryptographic primitives are not already compromised.
- The caller can identify the intended local pack root.
- Linux hosts supply a stable mount namespace for the duration of composition.
- Inputs are explicit rather than read from mutable ambient state by inner layers.
- Hosts do not treat structural validation as semantic approval.
- Binary and schema acquisition trust is defined by the operator or host.
- Controls marked **Planned** or **Partial** are not production guarantees.

## Threat and control status matrix

Status meanings:

- **Implemented:** present and covered by executable validation.
- **Partial:** prerequisites exist, but the threat is not fully mitigated.
- **Planned:** accepted requirement without a complete implementation.
- **Delegated:** explicitly owned by a pack author, operator, or host.
- **Out of scope:** not a security property Invokrum claims.

| ID | Threat or abuse case | Status | Current control or boundary | Owner / follow-up |
| --- | --- | --- | --- | --- |
| T01 | Invalid syntax, unknown v1 fields, or unsupported schema families create ambiguous interpretation. | Implemented | Strict DTOs, unknown-field rejection, schema preflight, JSON Schema, and fixtures. | `invokrum-schema`; regression CI. |
| T02 | Duplicate declarations or selected overlay IDs, dangling references, wrong-class selections, or invalid cardinality bypass structural policy. | Implemented | Validated domain values and aggregate construction reject these states. | `invokrum-core`; unit and integration tests. |
| T03 | Nondeterministic declaration, collection, diagnostic, or filesystem ordering changes output. | Partial | Domain, schema, and composition ordering are deterministic and tested; CLI multi-diagnostic ordering and persistent manifest encoding remain. | Issues #5 and #6. |
| T04 | Traversal, platform separators, symlinks, hard links, namespace aliases, mount points, reparse points, or races expose unintended bytes. | Partial | Portable lexical validation plus the Linux adapter reject symlinks, hard links, device crossings, root escapes, non-regular files, and changed opened-file identity. Non-Linux platforms fail closed. | Stable host namespace precondition; future platform adapters. |
| T05 | A structurally valid overlay injects instructions or changes model or tool behavior. | Delegated | Invokrum preserves provenance and structure but does not classify prompt semantics. | Pack trust, host approvals, sandboxing, and capability policy. |
| T06 | Mutable remote content or substitution changes composition without review. | Partial | Core, schema, composition, and filesystem adapters perform no network access; runtime composition consumes local bytes only. | Host acquisition policy; signed distribution requires explicit future design. |
| T07 | Secret variables leak through diagnostics, manifests, lockfiles, stdout, logs, or crashes. | Planned | Sensitivity is represented; interpolation, redaction, and persistence controls remain. | Issues #5, #6, and #20. |
| T08 | Hash, canonicalization, manifest, or lockfile confusion represents one artifact as another. | Planned | Exact source segments and normalized composition requirements are implemented; hashing and persistent canonical formats are not. | Issue #5. |
| T09 | Pathological nesting, collection size, file size, or output expansion causes denial of service. | Partial | Aliases and block scalars are rejected; selected-overlay count, per-file bytes, and normalized output bytes are bounded. Serialized document bytes, nesting depth, and declaration counts remain. | Issue #20. |
| T10 | A host modifies rendered bytes but claims the original digest or verification. | Delegated | Architecture requires exact-byte binding and invalidates claims after transformation. | Host contract; issue #5 provides digest material. |
| T11 | A host bypasses validation, reinterprets ordering, or invokes a runtime with different inputs. | Delegated | Stable adapter boundaries and composition-root rules are documented. | Host conformance work in issues #7 and #12. |
| T12 | Parser, dependency, build, release, or artifact compromise changes behavior. | Partial | Pinned Rust toolchain, Cargo lockfile, strict CI, and maintained schema dependencies are present. | Issue #8 for audits, action pinning, and release gates. |
| T13 | Errors or source locations expose sensitive content or unstable parser internals. | Partial | Domain errors are typed, parser messages are bounded, source failures use stable categories, and application source diagnostics do not echo paths. | Issue #6 for complete delivery-layer redaction and escaping. |
| T14 | Self-referential, asymmetric, or inconsistent incompatibility rules produce surprising results. | Implemented | References are validated and every selected directional or self-incompatibility declaration is evaluated in deterministic composition order before source reads. | `invokrum-core`; composition regression tests. |
| T15 | Duplicate JSON or YAML mapping keys, aliases, merge keys, tags, or multi-document input create parser-dependent meaning. | Implemented | Recursive structural preflight rejects duplicate keys at every depth; the documented YAML subset rejects directives, aliases, anchors, merge keys, tags, block scalars, explicit complex keys, and multiple documents. | `invokrum-schema`; adversarial regression tests. |
| T16 | Attacker-controlled paths or parser messages inject control characters into terminals, logs, or machine-output boundaries. | Partial | Identifiers are restricted to ASCII, parser messages are bounded, and application source diagnostics do not echo structured paths. | Issue #6 for any human path presentation and machine-output separation. |
| T17 | Output paths cause clobbering, symlink following, unsafe permissions, partial writes, or stale artifacts after failure. | Planned | The CLI and persistent output adapters are not implemented. | Issue #6; output adapter tests required. |

## Security invariants

These requirements are normative. Unimplemented items remain requirements rather than guarantees.

1. Core and application policy perform no implicit network access.
2. Inner layers do not read filesystem, process, environment, clock, randomness, or host state directly.
3. Unsupported schema families, unknown v1 fields, and unknown future rule kinds fail closed.
4. Parser-level duplicate keys and unsupported YAML features fail closed before semantic mapping.
5. Ordering does not depend on hash-map iteration, filesystem enumeration, locale, or platform path semantics.
6. Every composed file remains inside one canonical root under an explicit filesystem and link policy.
7. Validation, hashing, rendering, and manifests describe the same stable bytes.
8. Sensitive values are explicit, redacted from diagnostics, and excluded from persistent evidence by default.
9. Manifests, digests, and lockfiles identify their format and canonicalization version.
10. Verification applies only to the exact represented bytes.
11. Structurally valid prompt content remains untrusted and receives no semantic safety claim.
12. Hosts do not bypass validation or reinterpret canonical ordering while claiming verification.
13. Human output escapes attacker-controlled control characters; machine output remains valid for its declared encoding.
14. Persistent output uses explicit overwrite, link, permission, atomicity, and failure-cleanup policy.
15. Security limits and failures are deterministic and testable.

## Abuse cases and required mitigations

### Pack-root escape or namespace alias

An attacker declares `../../secret`, an absolute or Windows-style path, a symlink chain, a hard link, or a path below a mount that exposes unintended bytes. Lexical rejection occurs before I/O. On Linux, `invokrum-fs` rejects links, device changes, canonical escapes, non-regular files, and opened-file identity changes. Windows junction and reparse behavior is not approximated; the adapter fails closed as unsupported. A privileged hostile mount namespace remains a host-controlled residual risk.

### Ambiguous serialized input

An attacker repeats a profile-selection key, uses a YAML merge key or alias, or supplies multiple YAML documents so parsers or versions could derive different values. The schema boundary recursively rejects duplicate mapping keys and excludes parser-expanding YAML features before schema-family negotiation or semantic mapping.

### Prompt injection in an approved-looking pack

A pack passes structural validation but contains instructions that manipulate a runtime. Invokrum reports source identity and content digest but does not label prose safe. Hosts apply publisher trust, human approval, sandboxing, and capability policy.

### Secret exfiltration through evidence

A sensitive value is interpolated and copied into a diagnostic, manifest, lockfile, or log. Sensitive values require a dedicated representation, redacted diagnostics, safe rendering, and default exclusion from persisted evidence.

### Mutable-input race

A file is validated and then replaced before hashing or rendering. The Linux adapter compares candidate and opened identity, reads through one open descriptor, verifies the opened target is contained, and compares identity and metadata after reading. Downstream composition and hashing must consume the returned bytes rather than reopening the path.

### Canonicalization split

Platforms or versions normalize a pack differently but produce comparable-looking evidence. Canonicalization rules must be versioned, platform-independent, tested across fixtures, and included in artifact identity.

### Terminal or log injection

A path or parser message contains newlines, terminal escapes, or delimiter-like text that forges diagnostics or corrupts downstream parsing. Application source failures expose stable categories and do not echo paths. Any delivery-layer path presentation must escape or visibly encode controls, while JSON and other machine formats must remain syntactically valid and separate from human stderr.

### Unsafe output replacement

An output path points through a symlink, names an existing sensitive file, or receives a partial result before composition fails. The output adapter must use explicit overwrite and link policy, safe permissions, atomic replacement where supported, and cleanup that never presents an incomplete artifact as successful.

### Host attestation laundering

A host appends instructions after verification and records the original digest. Transforming hosts must create a new artifact identity and cannot preserve the original verification claim.

### Resource exhaustion

A pack uses deep nesting, large collections, oversized overlays, or expansion-heavy variables. YAML aliases and block scalars are not accepted. Composition bounds selected overlay count, each source read, and normalized output growth; schema document byte, nesting, and declaration-count limits remain required.

## Responsibility matrix

| Responsibility | Invokrum core or adapters | Pack author or distributor | Host integration |
| --- | --- | --- | --- |
| Schema and structural validity | Enforce | Produce conforming documents | Reject failures |
| Duplicate keys and accepted YAML subset | Schema adapter enforces | Avoid unsupported features | Do not bypass decoding |
| Deterministic ordering and compatibility | Enforce | Declare explicit intent | Do not reinterpret |
| Local path containment | Linux adapter enforces documented policy | Use pack-relative paths | Select protected root and stable namespace; reject unsupported platforms |
| Publisher authenticity | Not currently provided | Sign or publish through chosen process | Authenticate and pin source |
| Prompt semantic safety | Not provided | Review and govern content | Apply approvals and capability policy |
| Secret handling | Enforce declared policy when implemented | Mark variables correctly | Supply values securely and protect outputs |
| Runtime authorization and sandboxing | Not provided | N/A | Enforce |
| Output path and terminal safety | CLI and output adapters enforce | Avoid hostile names where possible | Select destinations and retain securely |
| Exact-byte execution binding | Produce identity material | N/A | Bind digest to invocation bytes |
| Audit retention and access control | Produce bounded evidence | N/A | Persist and protect evidence |

## Security claim discipline

Documentation labels controls as **Implemented**, **Partial**, **Planned**, **Delegated**, or **Out of scope**. A control may be **Implemented** only when executable validation demonstrates the behavior. Adding a threat, changing a boundary, or changing status requires updating this document and linked issue or test evidence.

The threat-model checker validates document structure and status-table integrity. It does not prove that prose accurately describes code; reviewers must verify every **Implemented** claim against executable evidence.

## Residual risk

Residual risk includes malicious prompt semantics, compromised hosts or dependencies, unsafe model or tool behavior, stolen signing keys, operator error, privileged hostile mount namespaces, unsupported non-Linux filesystem semantics, and operating-system or filesystem vulnerabilities. Invokrum reduces ambiguity and improves evidence; it does not replace host security architecture.

## Vulnerability reporting

Report suspected vulnerabilities privately as described in [`SECURITY.md`](../../SECURITY.md). Do not include live secrets or confidential third-party content in reports or fixtures.
