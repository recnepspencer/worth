from __future__ import annotations

from pathlib import Path
from unittest import TestCase, main

import run_worth_ui_compile_probes as probes


SOURCE = probes.RUNTIME / "probe.rs"
TEXT = """use crate::Thing;

// expect: compiles
#[cfg(worth_ui_compile_probe = "reads")]
fn reads(thing: Thing) {}

// expect: E0616
#[cfg(worth_ui_compile_probe = "writes")]
fn writes(thing: Thing) {
    thing.private = 1;
}
"""


def error(code: str | None, line: int, source: Path = SOURCE) -> dict[str, object]:
    return {
        "level": "error",
        "message": "refused",
        "code": {"code": code} if code else None,
        "spans": [
            {
                "is_primary": True,
                "file_name": str(source.relative_to(probes.WORKSPACE)),
                "line_start": line,
            }
        ],
    }


class CompileProbeTests(TestCase):
    def test_cases_span_to_the_next_case(self) -> None:
        found = probes.probes_in(SOURCE, TEXT)

        self.assertEqual(
            [(probe.name, probe.expected, probe.first_line, probe.last_line) for probe in found],
            [("reads", None, 3, 6), ("writes", "E0616", 7, 12)],
        )

    def test_crlf_sources_are_read(self) -> None:
        found = probes.probes_in(SOURCE, TEXT.replace("\n", "\r\n"))

        self.assertEqual([probe.name for probe in found], ["reads", "writes"])

    def test_a_refusal_needs_its_code_inside_its_case(self) -> None:
        writes = probes.probes_in(SOURCE, TEXT)[1]
        aborting = {"level": "error", "message": "aborting due to 1 previous error"}

        self.assertEqual(probes.refusal_failures(writes, 1, [error("E0616", 10), aborting]), [])
        self.assertTrue(probes.refusal_failures(writes, 0, []))
        self.assertTrue(probes.refusal_failures(writes, 1, [error("E0308", 10)]))
        self.assertTrue(probes.refusal_failures(writes, 1, [error("E0616", 5)]))
        self.assertTrue(
            probes.refusal_failures(
                writes, 1, [error("E0616", 10), error("E0616", 10, probes.RUNTIME / "lib.rs")]
            )
        )

    def test_compiling_cases_admit_no_warning(self) -> None:
        warning = {"level": "warning", "message": "unused import"}

        self.assertEqual(probes.compiling_failures(0, []), [])
        self.assertEqual(probes.compiling_failures(0, [warning]), ["unused import"])
        self.assertTrue(probes.compiling_failures(101, []))

    def test_every_probe_source_pairs_its_refusals_with_a_compiling_case(self) -> None:
        loaded = probes.load_probes()

        self.assertTrue(any(probe.expected is None for probe in loaded))
        self.assertTrue(any(probe.expected for probe in loaded))


if __name__ == "__main__":
    main()
