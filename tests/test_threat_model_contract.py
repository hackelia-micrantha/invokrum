from __future__ import annotations

import unittest

from scripts.check_threat_model import parse_threat_rows, validate_threat_rows


class ThreatModelContractTests(unittest.TestCase):
    def test_rejects_invalid_status_even_when_expected_row_exists(self) -> None:
        text = """
| T01 | valid row | Implemented | control | evidence |
| T01 | forged duplicate | Complete | control | evidence |
"""
        rows, parse_errors = parse_threat_rows(text)
        errors = parse_errors + validate_threat_rows(rows, {"T01"})

        self.assertTrue(any("invalid status" in error for error in errors))
        self.assertTrue(any("duplicate threat IDs" in error for error in errors))

    def test_rejects_malformed_threat_row(self) -> None:
        rows, errors = parse_threat_rows("| T01 | missing cells | Planned |\n")

        self.assertEqual(rows, [])
        self.assertTrue(any("must contain exactly five columns" in error for error in errors))

    def test_accepts_one_complete_row_per_expected_id(self) -> None:
        text = """
| T01 | first | Implemented | control | tests |
| T02 | second | Partial | control | issue #4 |
"""
        rows, parse_errors = parse_threat_rows(text)
        errors = parse_errors + validate_threat_rows(rows, {"T01", "T02"})

        self.assertEqual(errors, [])


if __name__ == "__main__":
    unittest.main()
