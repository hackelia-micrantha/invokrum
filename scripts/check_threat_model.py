#!/usr/bin/env python3
"""Fail when the documented threat model loses required structure or links."""

from __future__ import annotations

from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[1]
THREAT_MODEL = ROOT / "docs" / "security" / "threat-model.md"

REQUIRED_HEADINGS = {
    "## Security objectives",
    "## Assets",
    "## Actors",
    "## Trust boundaries",
    "## Assumptions",
    "## Threat and control status matrix",
    "## Security invariants",
    "## Abuse cases and required mitigations",
    "## Responsibility matrix",
    "## Security claim discipline",
    "## Residual risk",
    "## Vulnerability reporting",
}

REQUIRED_LINKS = {
    ROOT / "README.md": "docs/security/threat-model.md",
    ROOT / "SECURITY.md": "docs/security/threat-model.md",
    ROOT / "docs" / "README.md": "security/threat-model.md",
    ROOT / "docs" / "architecture" / "README.md": "../security/threat-model.md",
}

ALLOWED_STATUSES = {
    "Implemented",
    "Partial",
    "Planned",
    "Delegated",
    "Out of scope",
}
EXPECTED_THREATS = {f"T{number:02d}" for number in range(1, 15)}
ROW_PATTERN = re.compile(
    r"^\| (T\d{2}) \| .*? \| (Implemented|Partial|Planned|Delegated|Out of scope) \|",
    re.MULTILINE,
)


def main() -> int:
    errors: list[str] = []

    if not THREAT_MODEL.is_file():
        print("threat-model check failed: missing docs/security/threat-model.md", file=sys.stderr)
        return 1

    text = THREAT_MODEL.read_text(encoding="utf-8")

    for heading in sorted(REQUIRED_HEADINGS):
        if heading not in text:
            errors.append(f"threat model is missing required heading: {heading}")

    rows = ROW_PATTERN.findall(text)
    ids = [threat_id for threat_id, _ in rows]
    statuses = {status for _, status in rows}

    duplicates = sorted({threat_id for threat_id in ids if ids.count(threat_id) > 1})
    if duplicates:
        errors.append(f"duplicate threat IDs: {', '.join(duplicates)}")

    missing = sorted(EXPECTED_THREATS - set(ids))
    unexpected = sorted(set(ids) - EXPECTED_THREATS)
    if missing:
        errors.append(f"missing threat IDs: {', '.join(missing)}")
    if unexpected:
        errors.append(f"unexpected threat IDs: {', '.join(unexpected)}")

    invalid_statuses = sorted(statuses - ALLOWED_STATUSES)
    if invalid_statuses:
        errors.append(f"invalid threat statuses: {', '.join(invalid_statuses)}")

    for path, required_link in REQUIRED_LINKS.items():
        if not path.is_file():
            errors.append(f"missing documentation file: {path.relative_to(ROOT)}")
            continue
        linked_text = path.read_text(encoding="utf-8")
        if required_link not in linked_text:
            errors.append(
                f"{path.relative_to(ROOT)} must link to {required_link}"
            )

    if errors:
        print("threat-model check failed:", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1

    print(
        "threat-model check passed "
        f"({len(ids)} threats, {len(REQUIRED_LINKS)} required links)"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
