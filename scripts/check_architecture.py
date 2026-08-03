#!/usr/bin/env python3
"""Fail when workspace crates cross documented clean-architecture boundaries."""

from __future__ import annotations

from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[1]
CORE = ROOT / "crates" / "invokrum-core"
SCHEMA = ROOT / "crates" / "invokrum-schema"

FORBIDDEN_CORE_DEPENDENCIES = {
    "clap",
    "reqwest",
    "serde",
    "serde_json",
    "serde_yaml",
    "tokio",
    "tracing-subscriber",
}

FORBIDDEN_CORE_SOURCE_TOKENS = {
    "serde::",
    "serde_json::",
    "serde_yaml::",
    "std::env::",
    "std::fs::",
    "std::net::",
    "std::process::",
}


def main() -> int:
    errors: list[str] = []

    core_manifest = (CORE / "Cargo.toml").read_text(encoding="utf-8")
    for dependency in sorted(FORBIDDEN_CORE_DEPENDENCIES):
        if f"{dependency} =" in core_manifest or f'"{dependency}"' in core_manifest:
            errors.append(f"core manifest contains forbidden outer-layer dependency: {dependency}")

    if "invokrum-schema" in core_manifest:
        errors.append("core must not depend on the outward schema adapter")

    schema_manifest_path = SCHEMA / "Cargo.toml"
    if not schema_manifest_path.is_file():
        errors.append("schema adapter crate is missing")
    else:
        schema_manifest = schema_manifest_path.read_text(encoding="utf-8")
        if "invokrum-core" not in schema_manifest:
            errors.append("schema adapter must depend inward on invokrum-core")

    for source in sorted((CORE / "src").rglob("*.rs")):
        text = source.read_text(encoding="utf-8")
        for token in sorted(FORBIDDEN_CORE_SOURCE_TOKENS):
            if token in text:
                relative = source.relative_to(ROOT)
                errors.append(f"{relative} directly accesses outer boundary through {token}")

    if errors:
        print("architecture boundary check failed:", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1

    print("architecture boundary check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
