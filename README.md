<p align="center">
  <img src="assets/invokrum-mark.svg" width="168" alt="Invokrum: layered green bracts surrounding a blue deterministic core">
</p>

<h1 align="center">Invokrum</h1>

<p align="center">
  <strong>Deterministic prompt-overlay composition and attestation for governed AI contexts.</strong>
</p>

<p align="center">
  <a href="LICENSE"><img alt="License: Apache-2.0" src="https://img.shields.io/badge/license-Apache--2.0-2f7d65.svg"></a>
  <img alt="Project status: schema baseline" src="https://img.shields.io/badge/status-schema%20baseline-355f7d.svg">
  <img alt="Implementation language: Rust" src="https://img.shields.io/badge/language-Rust-b7410e.svg">
</p>

> [!IMPORTANT]
> Invokrum has an implemented domain model, strict v1 YAML/JSON schema adapter, and layered test baseline. Composition, rendering, lockfiles, attestations, and the proposed CLI commands are not yet complete.

## What is Invokrum?

Invokrum is an open-source engine for composing layered prompt context with explicit ordering, compatibility rules, validation, provenance, and reproducible output.

It treats prompt context less like an ad hoc string and more like a build input:

- overlays belong to declared classes;
- profiles select a bounded set of overlays;
- ordering and cardinality are deterministic;
- incompatible combinations fail closed;
- source and rendered content can be hashed and locked;
- a resolved manifest explains exactly what entered an agent context.

The goal is not another prompt-template manager. Invokrum is intended to provide a small, auditable mechanism for systems where prompt composition affects authority, security, cost, quality, or execution behavior.

## Documentation

- [Purpose and scope](docs/purpose.md)
- [Use cases](docs/use-cases.md)
- [Architecture](docs/architecture/README.md)
- [V1 schema contract](docs/schema-v1.md)
- [Configuration](docs/configuration.md)
- [Usage](docs/usage.md)
- [Development](docs/development.md)
- [Roadmap](docs/roadmap.md)
- [Contributing](CONTRIBUTING.md)
- [Security](SECURITY.md)
- [Support](SUPPORT.md)
- [Governance](GOVERNANCE.md)

See the [documentation index](docs/README.md) for status conventions and the complete reference set.

## Why the name?

**Invokrum** is a technical twist on *involucrum* and *invoke*.

In botany, an involucrum is a surrounding structure of bracts: modified leaves arranged around and protecting a flower or flower cluster. That maps naturally to a prompt-overlay system, where ordered layers surround a core context and add constraints without weakening what came before.

The spelling also points to invocation. Invokrum resolves and attests context before an agent, model, or tool is invoked.

## Design principles

1. **Deterministic by construction** — identical normalized inputs produce identical composition and diagnostics.
2. **Mechanism, not policy** — the engine provides ordering and validation; consumers define their classes and governance rules.
3. **Fail closed** — ambiguity, missing requirements, unsupported schema versions, and unresolved conflicts are errors.
4. **Offline composition** — runtime composition does not implicitly fetch mutable remote content.
5. **Attestable inputs and outputs** — packs, overlays, profiles, variables, and rendered context can be content-addressed.
6. **Explainable resolution** — machine-readable manifests expose what was selected, in which order, and why.
7. **Host independence** — Anthesis, CI systems, editors, MCP servers, and agent runtimes integrate through adapters rather than core-specific branches.

The mechanism-versus-policy boundary is documented in [ADR-0001](docs/architecture/ADR-0001-mechanism-policy-boundary.md).

## Architecture

```text
┌──────────────────────────────────────────────────────────┐
│ Host integrations                                        │
│ Anthesis · CI · MCP · editors · agent runtimes           │
└────────────────────────────┬─────────────────────────────┘
                             │ stable API / JSON contract
┌────────────────────────────▼─────────────────────────────┐
│ invokrum-cli                                              │
│ delivery, diagnostics, exit codes, composition root      │
└────────────────────────────┬─────────────────────────────┘
                             │ format adapter
┌────────────────────────────▼─────────────────────────────┐
│ invokrum-schema                                           │
│ strict YAML/JSON DTOs · version negotiation · encoding   │
└────────────────────────────┬─────────────────────────────┘
                             │ validated domain values
┌────────────────────────────▼─────────────────────────────┐
│ invokrum-core                                             │
│ model · invariants · deterministic normalization         │
└────────────────────────────┬─────────────────────────────┘
                             │
┌────────────────────────────▼─────────────────────────────┐
│ Consumer-owned overlay packs                              │
│ classes · profiles · overlays · compatibility policy     │
└──────────────────────────────────────────────────────────┘
```

The workspace uses durable boundaries rather than placing serialization inside the domain:

- `invokrum-core` owns parsing-neutral domain types and engine behavior;
- `invokrum-schema` translates strict YAML/JSON documents into the domain model;
- `invokrum-cli` owns delivery concerns and will wire concrete adapters.

## V1 pack format

The implemented schema family is `invokrum.dev/v1`:

```yaml
schema: invokrum.dev/v1
id: example

classes:
  - id: core
    order: 10
    minimum: 1
    maximum: 1
  - id: mode
    order: 90
    minimum: 1
    maximum: 1

overlays:
  - id: core-invariant
    class: core
    source: overlays/core/invariant.md
  - id: review
    class: mode
    source: overlays/modes/review.md

profiles:
  - id: secure-review
    selections:
      core:
        - core-invariant
      mode:
        - review
```

Unknown fields and unsupported schema families fail closed. Paths use a portable forward-slash grammar. See [docs/schema-v1.md](docs/schema-v1.md) and the [machine-readable JSON Schema](schemas/invokrum-pack-v1.schema.json).

## Proposed CLI workflow

The intended CLI surface remains under implementation:

```bash
invokrum validate --pack ./pack.yaml --profile secure-review
invokrum compose  --pack ./pack.yaml --profile secure-review
invokrum inspect  --pack ./pack.yaml --profile secure-review --format json
invokrum lock     --pack ./pack.yaml --profile secure-review
invokrum verify   --lock ./invokrum.lock
invokrum diff     ./baseline.lock ./candidate.lock
```

## Relationship to Anthesis

Invokrum originates from the prompt-overlay composition model developed inside [Anthesis](https://github.com/hackelia-micrantha/anthesis).

| Invokrum owns | Anthesis owns |
| --- | --- |
| Generic schemas and domain types | Anthesis overlay taxonomy |
| Deterministic ordering and resolution | Core invariant and STOP semantics |
| Cardinality and compatibility validation | Governance and approval policy |
| Rendering, hashing, and lockfiles | Session, evidence, and audit binding |
| Adapter contracts | Anthesis runtime integration |

Anthesis is expected to become an early real-world consumer and compatibility test, not a special case embedded in the engine.

## Roadmap

The active backlog is tracked in [GitHub Issues](https://github.com/hackelia-micrantha/invokrum/issues). The milestone sequence is documented in [docs/roadmap.md](docs/roadmap.md).

## Security posture

Prompt overlays are configuration and potentially untrusted content. Invokrum assumes hostile or malformed inputs and plans controls for traversal, symlink escape, malicious packs, secret persistence, canonicalization confusion, denial of service, and unsafe host adapters.

No security guarantee should be inferred before the relevant controls are implemented and tested. See [SECURITY.md](SECURITY.md) and threat-model issue [#9](https://github.com/hackelia-micrantha/invokrum/issues/9).

## Contributing

Start with [CONTRIBUTING.md](CONTRIBUTING.md) and the [development guide](docs/development.md).

## License

Invokrum is licensed under the [Apache License 2.0](LICENSE).
