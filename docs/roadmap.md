# Roadmap

Invokrum is being built from architecture and compatibility contracts outward. Milestones are intentionally capability-oriented rather than date-based.

## v0.1 — deterministic local composition

- [x] Accept the mechanism-versus-policy architecture boundary.
- [x] Implement typed pack, overlay, class, profile, and rule models.
- [x] Publish the first versioned YAML/JSON schema.
- [x] Validate references, cardinality, compatibility, and pack-relative paths.
- [x] Resolve deterministic overlay order.
- [x] Render canonical context bytes.
- [x] Emit stable inspection JSON and diagnostics.
- [x] Add hashes, resolved manifests, lockfiles, verification, and structural diffing.
- [x] Provide the initial CLI.
- [x] Add a minimal example pack.
- [x] Establish CI and reproducible pre-release artifacts.
- [ ] Publish and independently verify the first v0.1 prerelease.

Anthesis conformance is not a v0.1 release gate. Invokrum must remain independently useful and must not make Anthesis policy part of its public model.

## v0.2 — integration and reference-consumer validation

Potential scope after v0.1 contracts are released:

- review the stable library API;
- evolve the implemented subprocess JSON request/response contract from real consumer feedback;
- represent selected Anthesis behavior as an external reference-consumer conformance pack;
- add an Anthesis adapter without Anthesis-specific branches in the engine;
- add a read-only MCP adapter;
- add a GitHub Action for validation and drift checks;
- design a signed pack-install workflow separated from offline composition.

The Anthesis conformance suite is intended to test the extraction boundary, not to establish permanent product coupling or compatibility guarantees.

## Later exploration

These are not commitments:

- WASM builds for editors or browser tooling;
- language bindings;
- pack registries and discovery metadata;
- policy-authoring diagnostics;
- richer conditional rule expressions;
- profile composition or inheritance;
- signature verification and transparency-log integration.

## Explicit sequencing constraints

- Remote pack loading should not precede a documented trust and signature model.
- Plugin code execution should not precede a capability and sandbox model.
- Profile inheritance should not precede deterministic merge semantics.
- Public JSON and lockfile formats should not be declared stable before golden and cross-platform tests exist.
- Consumer-specific conformance work must not introduce consumer-specific policy into the core engine.

The issue tracker is the authoritative source for active work and dependencies.
