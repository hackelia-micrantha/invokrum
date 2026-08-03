<p align="center">
  <img src="assets/invokrum-mark.svg" width="168" alt="Invokrum: layered green bracts surrounding a blue deterministic core">
</p>

<h1 align="center">Invokrum</h1>

<p align="center">
  <strong>Deterministic prompt-overlay composition and attestation for governed AI contexts.</strong>
</p>

<p align="center">
  <a href="LICENSE"><img alt="License: Apache-2.0" src="https://img.shields.io/badge/license-Apache--2.0-2f7d65.svg"></a>
  <img alt="Project status: architecture" src="https://img.shields.io/badge/status-architecture-355f7d.svg">
  <img alt="Implementation language: Rust" src="https://img.shields.io/badge/language-Rust-b7410e.svg">
</p>

> [!IMPORTANT]
> Invokrum is at the architecture and extraction stage. The interfaces and commands below describe the intended v0.1 direction; they are not yet a released implementation.

## What is Invokrum?

Invokrum is an open-source engine for composing layered prompt context with explicit ordering, compatibility rules, validation, provenance, and reproducible output.

It treats prompt context less like an ad hoc string and more like a build input:

- overlays belong to declared classes;
- profiles select a bounded set of overlays;
- ordering and cardinality are deterministic;
- incompatible combinations fail closed;
- source and rendered content can be hashed and locked;
- a resolved manifest explains exactly what entered an agent context.

The goal is not to create another prompt-template manager. Invokrum is intended to provide a small, auditable mechanism for systems where prompt composition affects authority, security, cost, quality, or execution behavior.

## Documentation

- [Purpose and scope](docs/purpose.md)
- [Use cases](docs/use-cases.md)
- [Architecture](docs/architecture/README.md)
- [Configuration](docs/configuration.md)
- [Usage](docs/usage.md)
- [Development](docs/development.md)
- [Roadmap](docs/roadmap.md)
- [Contributing](CONTRIBUTING.md)
- [Security](SECURITY.md)
- [Support](SUPPORT.md)
- [Governance](GOVERNANCE.md)

See the [documentation index](docs/README.md) for status conventions and the complete set of project references.

## Why the name?

**Invokrum** is a technical twist on *involucrum* and *invoke*.

In botany, an **involucrum** is a surrounding structure of bracts: modified leaves arranged around and protecting a flower or flower cluster. That maps naturally to a prompt-overlay system, where ordered layers surround a core context and add constraints without weakening what came before.

The spelling also points to **invocation**. Invokrum is meant to resolve and attest context before an agent, model, or tool is invoked.

The project mark reflects that origin: layered botanical forms surround a deterministic core inside a restrained systems-oriented boundary.

## Design principles

1. **Deterministic by construction** — identical normalized inputs produce identical composition and diagnostics.
2. **Mechanism, not policy** — the engine provides ordering and validation; consumers define their own classes and governance rules.
3. **Fail closed** — ambiguity, missing requirements, unsupported schema versions, and unresolved conflicts are errors.
4. **Offline composition** — runtime composition does not implicitly fetch mutable remote content.
5. **Attestable inputs and outputs** — packs, overlays, profiles, variables, and rendered context can be content-addressed.
6. **Explainable resolution** — machine-readable manifests expose what was selected, in which order, and why.
7. **Host independence** — Anthesis, CI systems, editors, MCP servers, and agent runtimes integrate through adapters rather than core-specific branches.

The accepted mechanism-versus-policy boundary, invariants, compatibility model, error taxonomy, and v0.1 non-goals are documented in [ADR-0001](docs/architecture/ADR-0001-mechanism-policy-boundary.md).

## Architecture

```text
┌──────────────────────────────────────────────────────────┐
│ Host integrations                                        │
│ Anthesis · CI · MCP · editors · agent runtimes           │
└────────────────────────────┬─────────────────────────────┘
                             │ stable API / JSON contract
┌────────────────────────────▼─────────────────────────────┐
│ invokrum-cli                                              │
│ validate · compose · inspect · lock · verify · diff      │
└────────────────────────────┬─────────────────────────────┘
                             │ typed Rust API
┌────────────────────────────▼─────────────────────────────┐
│ invokrum-core                                             │
│ model · resolution · rules · rendering · hashing         │
└────────────────────────────┬─────────────────────────────┘
                             │
┌────────────────────────────▼─────────────────────────────┐
│ Consumer-owned overlay packs                              │
│ classes · profiles · overlays · compatibility policy     │
└──────────────────────────────────────────────────────────┘
```

The initial Rust workspace intentionally starts with two crates:

- `invokrum-core` owns the generic typed model and deterministic engine behavior;
- `invokrum-cli` exposes operator-facing and subprocess-safe contracts.

Additional crates should be introduced only when they represent a durable compatibility or trust boundary.

## Proposed workflow

A future pack may describe ordered classes and profiles in YAML or JSON:

```yaml
apiVersion: invokrum.dev/v1
kind: OverlayPack

metadata:
  name: example
  version: 0.1.0

classes:
  - name: core
    order: 10
    cardinality: { min: 1, max: 1 }
  - name: security
    order: 30
    cardinality: { min: 0 }
  - name: mode
    order: 90
    cardinality: { min: 1, max: 1 }

profiles:
  secure-review:
    overlays:
      - core/invariant
      - security/default
      - mode/review
```

The intended CLI surface is:

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

The extraction boundary is deliberate:

| Invokrum owns | Anthesis owns |
| --- | --- |
| Generic schemas and domain types | Anthesis overlay taxonomy |
| Deterministic ordering and resolution | Core invariant and STOP semantics |
| Cardinality and compatibility validation | Governance and approval policy |
| Rendering, hashing, and lockfiles | Session, evidence, and audit binding |
| Adapter contracts | Anthesis runtime integration |

Anthesis is expected to become an early real-world consumer and compatibility test, not a special case embedded in the engine.

## Roadmap

The active project backlog is tracked in [GitHub Issues](https://github.com/hackelia-micrantha/invokrum/issues). The milestone sequence is documented in [docs/roadmap.md](docs/roadmap.md).

## Security posture

Prompt overlays are configuration **and potentially untrusted content**. Invokrum's design therefore assumes hostile or malformed inputs and plans to address:

- path traversal and symlink escape;
- ambiguous filesystem resolution;
- malicious or compromised packs;
- secret interpolation and accidental persistence;
- canonicalization and lockfile confusion;
- denial of service through pathological input;
- adapters that bypass or reinterpret validated output.

No security guarantee should be inferred before the relevant controls are implemented and tested. See [SECURITY.md](SECURITY.md) and threat-model issue [#9](https://github.com/hackelia-micrantha/invokrum/issues/9).

## Contributing

Design discussion and focused contributions are welcome. Start with [CONTRIBUTING.md](CONTRIBUTING.md) and the [development guide](docs/development.md).

## License

Invokrum is licensed under the [Apache License 2.0](LICENSE).
