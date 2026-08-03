# Roadmap

Invokrum is being built from architecture and compatibility contracts outward. Milestones are intentionally capability-oriented rather than date-based.

## v0.1 — deterministic local composition

- [ ] Accept the mechanism-versus-policy architecture boundary.
- [ ] Implement typed pack, overlay, class, profile, and rule models.
- [ ] Publish the first versioned YAML/JSON schema.
- [ ] Validate references, cardinality, compatibility, and pack-relative paths.
- [ ] Resolve deterministic overlay order.
- [ ] Render canonical context bytes.
- [ ] Emit stable inspection JSON and diagnostics.
- [ ] Add hashes, resolved manifests, lockfiles, verification, and structural diffing.
- [ ] Provide the initial CLI.
- [ ] Add a minimal example pack.
- [ ] Prove selected behavior against Anthesis compatibility fixtures.
- [ ] Establish CI and reproducible pre-release artifacts.

## v0.2 — integration contracts

Potential scope after v0.1 contracts are stable:

- stable library API review;
- subprocess JSON request/response contract;
- Anthesis adapter;
- read-only MCP adapter;
- GitHub Action for validation and drift checks;
- signed pack-install workflow separated from offline composition.

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

The issue tracker is the authoritative source for active work and dependencies.