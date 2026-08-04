#!/usr/bin/env python3
"""Fail when workspace crates cross documented clean-architecture boundaries."""

from __future__ import annotations

from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[1]
CORE = ROOT / "crates" / "invokrum-core"
SCHEMA = ROOT / "crates" / "invokrum-schema"
FILESYSTEM = ROOT / "crates" / "invokrum-fs"
INTEGRITY = ROOT / "crates" / "invokrum-integrity"
HOST = ROOT / "crates" / "invokrum-host"

FORBIDDEN_CORE_DEPENDENCIES = {
    "clap",
    "invokrum-fs",
    "invokrum-host",
    "invokrum-integrity",
    "invokrum-schema",
    "reqwest",
    "serde",
    "serde_json",
    "serde_yaml",
    "serde_yaml_ng",
    "tokio",
    "tracing-subscriber",
}

FORBIDDEN_CORE_SOURCE_TOKENS = {
    "serde::",
    "serde_json::",
    "serde_yaml::",
    "serde_yaml_ng::",
    "std::env::",
    "std::fs::",
    "std::net::",
    "std::process::",
}

FORBIDDEN_HOST_DEPENDENCIES = {
    "invokrum-fs",
    "invokrum-schema",
    "reqwest",
    "serde",
    "serde_json",
    "tokio",
}

FORBIDDEN_HOST_SOURCE_TOKENS = {
    "serde::",
    "serde_json::",
    "std::env::",
    "std::fs::",
    "std::net::",
    "std::process::",
}


def require_inward_adapter(
    errors: list[str],
    crate: Path,
    name: str,
) -> None:
    manifest_path = crate / "Cargo.toml"
    if not manifest_path.is_file():
        errors.append(f"{name} adapter crate is missing")
        return

    manifest = manifest_path.read_text(encoding="utf-8")
    if "invokrum-core" not in manifest:
        errors.append(f"{name} adapter must depend inward on invokrum-core")
    if name == "filesystem" and any(
        dependency in manifest for dependency in ["invokrum-integrity", "invokrum-schema"]
    ):
        errors.append("filesystem adapter must depend only on core workspace policy")
    if name == "schema" and any(
        dependency in manifest for dependency in ["invokrum-fs", "invokrum-integrity"]
    ):
        errors.append("schema adapter must not depend on filesystem or integrity adapters")
    if name == "integrity" and any(
        dependency in manifest for dependency in ["invokrum-fs", "invokrum-schema"]
    ):
        errors.append("integrity adapter must consume validated core values directly")


def require_host_facade(errors: list[str]) -> None:
    manifest_path = HOST / "Cargo.toml"
    if not manifest_path.is_file():
        errors.append("host facade crate is missing")
        return
    manifest = manifest_path.read_text(encoding="utf-8")
    for dependency in sorted(FORBIDDEN_HOST_DEPENDENCIES):
        if f"{dependency} =" in manifest or f'"{dependency}"' in manifest:
            errors.append(f"host facade contains forbidden transport dependency: {dependency}")
    for required in ["invokrum-core", "invokrum-integrity"]:
        if required not in manifest:
            errors.append(f"host facade must depend on {required}")
    for source in sorted((HOST / "src").rglob("*.rs")):
        text = source.read_text(encoding="utf-8")
        for token in sorted(FORBIDDEN_HOST_SOURCE_TOKENS):
            if token in text:
                relative = source.relative_to(ROOT)
                errors.append(f"{relative} directly accesses transport boundary through {token}")


def main() -> int:
    errors: list[str] = []

    core_manifest = (CORE / "Cargo.toml").read_text(encoding="utf-8")
    for dependency in sorted(FORBIDDEN_CORE_DEPENDENCIES):
        if f"{dependency} =" in core_manifest or f'"{dependency}"' in core_manifest:
            errors.append(f"core manifest contains forbidden outer-layer dependency: {dependency}")

    require_inward_adapter(errors, SCHEMA, "schema")
    require_inward_adapter(errors, FILESYSTEM, "filesystem")
    require_inward_adapter(errors, INTEGRITY, "integrity")
    require_host_facade(errors)

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
