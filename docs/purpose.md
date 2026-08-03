# Purpose and scope

## Purpose

Invokrum provides a deterministic, policy-neutral mechanism for composing layered prompt context before an AI agent, model, or tool is invoked.

The project exists because prompt context increasingly carries operational meaning: authority, security requirements, execution constraints, quality gates, cost controls, and output contracts. Treating those inputs as informal string concatenation makes review, reproducibility, and incident analysis difficult.

Invokrum aims to make prompt composition:

- deterministic;
- explicitly ordered;
- schema-validated;
- fail-closed on ambiguity;
- explainable through resolved manifests;
- reproducible through content hashes and lockfiles;
- portable across hosts through stable adapter contracts.

## Mechanism, not policy

Invokrum does not define which governance rules a system should use. Consumers define their own:

- overlay classes and authority order;
- profiles;
- required and incompatible combinations;
- prompt content;
- approval and execution semantics.

Invokrum validates and resolves those declarations without embedding consumer-specific policy in the core.

## Intended users

- agent-platform and developer-experience teams;
- security and governance engineers;
- CI/CD maintainers validating prompt configuration;
- tool authors building agent runtimes, plugins, or MCP servers;
- researchers who need reproducible context assembly;
- projects such as Anthesis that require an independently testable composition engine.

## v0.1 goals

1. Define a portable overlay-pack model.
2. Resolve profiles deterministically.
3. Validate cardinality, compatibility, and paths.
4. Render composed context offline.
5. Emit stable machine-readable inspection output.
6. Produce content hashes, lockfiles, and verification results.
7. Provide a small Rust CLI and library boundary.
8. Prove the engine against selected Anthesis fixtures.

## Non-goals for v0.1

- hosting or executing language models;
- deciding whether prompt content is semantically safe or correct;
- remote mutable pack resolution during composition;
- secrets management;
- general-purpose templating;
- policy authoring UI;
- executing arbitrary plugin code;
- replacing host-level authorization, sandboxing, or audit systems.

## Success criteria

Invokrum is successful when two independent hosts can consume the same pinned pack and profile, obtain byte-identical normalized output, and explain or verify every input that contributed to it.