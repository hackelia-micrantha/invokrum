# Use cases

Invokrum is useful when prompt context is assembled from multiple sources and the composition itself must be reviewed, reproduced, or governed.

## Governed agent sessions

A platform can define a required core overlay, one execution mode, and optional security or quality overlays. Invokrum resolves the profile before the host starts the session and returns a digest that the host binds to execution evidence.

**Invokrum provides:** ordering, validation, rendering, and a resolved manifest.

**The host provides:** authorization, approvals, tool access, execution, and audit storage.

## CI validation for prompt packs

A repository can validate that:

- referenced overlays exist;
- required classes are present;
- exclusive classes contain exactly one selection;
- incompatible overlays are rejected;
- rendered output and lockfiles have not drifted.

This supports reviewable prompt changes without granting CI permission to execute an agent.

## Reproducible evaluations

An evaluation harness can pin the exact prompt pack, profile, variables, and rendered digest used for a benchmark. Results can then distinguish model changes from context-composition changes.

## Secure code-review context

A code-review profile may combine:

- a non-relaxable review invariant;
- a read-only mode;
- repository-specific security constraints;
- output-format requirements;
- evidence and quality gates.

An implementation profile that conflicts with the read-only overlay should fail before invocation.

## Environment-specific context

A consumer may define mutually exclusive local, CI, and production environment overlays while retaining common governance and security layers. Invokrum validates the selection but does not decide which environment is authorized.

## Host and plugin adapters

Anthesis, an MCP server, a GitHub Action, or an editor extension can consume stable library or JSON output rather than reimplementing resolution rules. Adapters must preserve validation and provenance results rather than parsing human-oriented CLI text.

## Cases that do not require Invokrum

Invokrum may be unnecessary when:

- a prompt is a single static file with no composition rules;
- reproducibility and provenance are not required;
- a host already provides an equivalent deterministic composition contract;
- the desired behavior is unrestricted text templating rather than governed layering.

## Misuse boundaries

Invokrum validation does not prove that prompt content is trustworthy, free from prompt injection, legally compliant, or authorized for execution. It validates declared structure and integrity; semantic review and runtime controls remain separate responsibilities.