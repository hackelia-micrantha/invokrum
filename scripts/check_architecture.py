#!/usr/bin/env python3
"""Fail when the core crate crosses documented clean-architecture boundaries."""

from __future__ import annotations

from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[1]
CORE = ROOT / "crates" / "invokrum-core"

FORBIDDEN_DEPENDENCIES = {
    "clap",
    "reqwest",
    "tokio",
    "tracing-subscriber",
}

FORBIDDEN_SOURCE_TOKENS = {
    "std::env::",
    "std::fs::",
    "std::net::",
    "std::process::",
}


def main() -> int:
    errors: list[str] = []

    manifest = (CORE / "Cargo.toml").read_text(encoding="utf-8")
    for dependency in sorted(FORBIDDEN_DEPENDENCIES):
        if f"{dependency} =" in manifest or f'"{dependency}"' in manifest:
            errors.append(f"core manifest contains forbidden outer-layer dependency: {dependency}")

    for source in sorted((CORE / "src").rglob("*.rs")):
        text = source.read_text(encoding="utf-8")
        for token in sorted(FORBIDDEN_SOURCE_TOKENS):
            if token in text:
                relative = source.relative_to(ROOT)
                errors.append(f"{relative} directly accesses host boundary through {token}")

    if errors:
        print("architecture boundary check failed:", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1

    print("architecture boundary check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
