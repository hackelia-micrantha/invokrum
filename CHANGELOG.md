# Changelog

All notable user-visible changes to Invokrum will be documented in this file.

The format is based on Keep a Changelog principles, and versioned releases will follow Semantic Versioning once a stable public contract exists.

## Unreleased

### Added

- Initial project purpose, architecture, use-case, configuration, usage, development, security, support, contribution, and roadmap documentation.
- Rust workspace scaffold with `invokrum-core` and `invokrum-cli`.
- ADR-0001 defining the mechanism-versus-policy boundary.

## Versioning notes

Before `1.0.0`, minor releases may include breaking changes to experimental interfaces. Breaking changes must still be explicit in this changelog and in release notes.

Compatibility-sensitive surfaces include:

- overlay-pack schemas;
- canonicalization and rendering rules;
- resolved manifest and lockfile formats;
- machine-readable CLI output;
- exit codes;
- public Rust APIs;
- adapter request and response envelopes.

Documentation-only corrections that do not alter a public contract may be grouped under the next release.