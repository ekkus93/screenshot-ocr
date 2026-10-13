"""Negative fixtures for the offline Windows OCR desktop acceptance guard."""
from __future__ import annotations

import importlib.util
import unittest
from pathlib import Path

MODULE_PATH = Path(__file__).with_name("check_windows_acceptance_fixture.py")
SPEC = importlib.util.spec_from_file_location("windows_acceptance_fixture", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
guard = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(guard)

FIXTURE = (
    Path(__file__).resolve().parent.parent
    / "docs/fixtures/windows-ocr-acceptance.html"
)


class OfflineAcceptanceFixtureTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.good = FIXTURE.read_text(encoding="utf-8")

    def test_current_public_fixture_passes(self) -> None:
        self.assertEqual(guard.validate(self.good), [])

    def test_rejects_remote_assets_and_active_content(self) -> None:
        for addition in (
            '<script src="https://example.invalid/a.js"></script>',
            '<img src="https://example.invalid/a.png">',
            '<iframe src="file:///private.txt"></iframe>',
            '<form action="/upload"></form>',
            '<meta http-equiv="refresh" content="0;url=https://example.invalid">',
            '<p onmouseover="send()">SAMPLE 123</p>',
        ):
            with self.subTest(addition=addition):
                changed = self.good.replace("</body>", addition + "</body>")
                self.assertTrue(guard.validate(changed))

    def test_rejects_external_css(self) -> None:
        for addition in (
            '@import "https://example.invalid/a.css";',
            "background: url(https://example.invalid/a.png);",
            "width: expression(alert(1));",
        ):
            with self.subTest(addition=addition):
                changed = self.good.replace("</style>", addition + "</style>")
                self.assertIn(
                    "External or executable CSS is forbidden.",
                    guard.validate(changed),
                )

    def test_rejects_missing_or_duplicate_test_cases(self) -> None:
        missing = self.good.replace('data-fixture-id="GRID-07"', "")
        self.assertTrue(guard.validate(missing))
        duplicate = self.good.replace(
            "</body>", '<section data-fixture-id="GRID-07"></section></body>'
        )
        self.assertTrue(guard.validate(duplicate))

    def test_rejects_prettier_collapsing_terminal_whitespace(self) -> None:
        changed = self.good.replace(
            "  indented    spaces    stay visible",
            "indented spaces stay visible",
        )
        self.assertIn(
            "Monospace OCR fixture whitespace changed.",
            guard.validate(changed),
        )

    def test_rejects_removed_preformatted_text(self) -> None:
        changed = self.good.replace("<pre class=\"mono\">", '<div class="mono">')
        changed = changed.replace("</pre>", "</div>")
        self.assertTrue(guard.validate(changed))


if __name__ == "__main__":
    unittest.main()
