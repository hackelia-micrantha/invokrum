# Configuration model

> [!NOTE]
> This document describes the planned v0.1 configuration contract. The schema is not yet implemented or compatibility-stable.

## Overview

An Invokrum configuration consists of:

- an overlay pack declaration;
- ordered overlay classes;
- overlay definitions backed by local files;
- named profiles selecting overlays;
- declarative validation rules;
- optional variables supplied at composition time.

## Planned pack shape

```yaml
apiVersion: invokrum.dev/v1
kind: OverlayPack

metadata:
  name: example
  version: 0.1.0

classes:
  - name: core
    order: 10
    cardinality:
      min: 1
      max: 1

  - name: security
    order: 30
    cardinality:
      min: 0

  - name: mode
    order: 90
    cardinality:
      min: 1
      max: 1

overlays:
  core/invariant:
    class: core
    path: overlays/core/invariant.md

  security/default:
    class: security
    path: overlays/security/default.md

  mode/review:
    class: mode
    path: overlays/modes/review.md

profiles:
  secure-review:
    overlays:
      - core/invariant
      - security/default
      - mode/review
```

## Ordering

Class order is explicit. Overlay order within a class must also be deterministic, either through declaration order with a canonical representation or an explicit overlay order field. Filesystem traversal order must never influence composition.

## Cardinality

Each class may declare minimum and maximum selections. Typical examples:

- exactly one core overlay: `min: 1`, `max: 1`;
- exactly one mode: `min: 1`, `max: 1`;
- optional environment: `min: 0`, `max: 1`;
- additive quality overlays: `min: 0`, no bounded maximum.

## Compatibility rules

Rules should identify stable overlay or capability identifiers rather than infer semantics from Markdown text. Planned rule categories include:

- required overlay or class;
- incompatible overlay pairs or sets;
- conditional requirement;
- prohibited combination;
- class cardinality.

Unknown rule kinds must fail closed.

## Paths

Overlay paths are pack-relative. Composition should:

1. establish a canonical pack root;
2. resolve each declared path against that root;
3. reject traversal and root escapes;
4. apply an explicit symlink policy;
5. reject non-regular or unreadable files where applicable.

Runtime HTTP or Git references are out of scope for v0.1. Hosts may acquire and verify packs before invoking Invokrum.

## Variables

Variables are expected to be explicit inputs, not ambient environment reads. A variable declaration may include:

- name and type;
- required/default status;
- whether its value is sensitive;
- rendering constraints.

Sensitive values must be redacted from diagnostics, manifests, and lockfiles unless an explicit future contract permits otherwise.

## Profiles

Profiles select overlays but must not silently redefine class order or engine behavior. Profile inheritance is intentionally deferred until its deterministic merge semantics and complexity are justified.

## Versioning

The API version identifies the schema and normalization rules. Unsupported major versions fail. Compatibility behavior for additive minor changes will be defined with the first published schema in issue #3.