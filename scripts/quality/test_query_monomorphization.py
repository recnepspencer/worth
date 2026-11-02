"""Hand-counted COFF reports check the archive instrument and its refusals."""
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from query_monomorphization import cargo_archive, family, measure


OBJECT = """File: fixture.rlib(code.o)
Format: COFF-x86-64
Sections [
  Section {
    Number: 1
    Name: .text (2E 74 65 78 74)
    RawDataSize: 12
    IMAGE_SCN_CNT_CODE (0x20)
  }
  Section {
    Number: 2
    Name: .data (2E 64 61 74 61)
    RawDataSize: 16
  }
]
Symbols [
  Symbol {
    Name: alpha8
    Value: 0
    Section: .text (1)
    ComplexType: Function (0x2)
    StorageClass: External (0x2)
  }
  Symbol {
    Name: alpha16
    Value: 0
    Section: .text (1)
    ComplexType: Function (0x2)
    StorageClass: External (0x2)
  }
  Symbol {
    Name: beta
    Value: 8
    Section: .text (1)
    ComplexType: Function (0x2)
    StorageClass: External (0x2)
  }
  Symbol {
    Name: data
    Value: 0
    Section: .data (2)
    ComplexType: Null (0x0)
    StorageClass: External (0x2)
  }
]
"""


class ArchiveMeasurementTests(unittest.TestCase):
    def measure_reports(self, *, object_report=OBJECT, size=12, omit_data=False,
                        mixed_alias=False):
        def report(command):
            tool = Path(command[0]).stem
            if tool == "llvm-nm":
                self.assertIn("--no-sort", command)
                values = (["sample::alpha::<u8>", "sample::alpha::<u16>",
                           "sample::beta", "sample::VALUE"] if "--demangle" in command
                          else ["alpha8", "alpha16", "beta", "data"])
                if mixed_alias and "--demangle" in command:
                    values[1] = "sample::renamed::<u16>"
                return iter(values[:-1] if omit_data else values)
            if tool == "llvm-readobj":
                return iter(object_report.splitlines())
            if tool == "llvm-size":
                return iter([f".text {size} 0", ".data 16 0"])
            self.fail(f"Unexpected tool {tool}")

        with tempfile.TemporaryDirectory() as directory:
            archive = Path(directory) / "fixture.rlib"
            archive.write_bytes(b"archive fixture")
            with patch("query_monomorphization.lines", side_effect=report):
                return measure(archive, Path(directory))

    def test_hand_counted_aliases_share_physical_bytes_but_keep_exported_names(self):
        result = self.measure_reports()
        self.assertEqual(result["exported_symbols"], 4)
        self.assertEqual(result["independent_nm_symbols"], 4)
        self.assertEqual(result["code_bytes"], 12)
        self.assertEqual(result["independent_size_code_bytes"], 12)
        self.assertEqual(result["unattributed_code_bytes"], 0)
        self.assertEqual(result["shared_alias_code_bytes"], 0)
        self.assertEqual(result["owners"], {"sample": 4})
        self.assertEqual(result["function_families"], [
            {"name": "sample::alpha::<>", "bytes": 8, "copies": 1},
            {"name": "sample::beta", "bytes": 4, "copies": 1},
        ])

    def test_mixed_family_aliases_have_explicit_shared_ownership(self):
        result = self.measure_reports(mixed_alias=True)
        self.assertEqual(result["code_bytes"], 12)
        self.assertEqual(result["unattributed_code_bytes"], 0)
        self.assertEqual(result["shared_alias_code_bytes"], 8)
        self.assertEqual(result["shared_alias_groups"], [{
            "names": ["sample::alpha::<>", "sample::renamed::<>"], "bytes": 8, "copies": 1,
        }])
        self.assertEqual(result["function_families"], [
            {"name": "sample::beta", "bytes": 4, "copies": 1},
        ])

    def test_corrupt_independent_size_total_is_refused(self):
        with self.assertRaisesRegex(RuntimeError, "Independent LLVM reports disagree"):
            self.measure_reports(size=13)

    def test_corrupt_independent_symbol_roster_is_refused(self):
        with self.assertRaisesRegex(RuntimeError, "missing from the independent nm report"):
            self.measure_reports(omit_data=True)

    def test_function_outside_its_section_is_refused(self):
        with self.assertRaisesRegex(RuntimeError, "exceeds its COFF section"):
            self.measure_reports(object_report=OBJECT.replace("Value: 8", "Value: 13"))

    def test_negative_function_coordinate_is_refused(self):
        with self.assertRaisesRegex(RuntimeError, "exceeds its COFF section"):
            self.measure_reports(object_report=OBJECT.replace("Value: 8", "Value: -1"))

    def test_generic_family_keeps_trait_owner_and_function_arrows(self):
        self.assertEqual(
            family("<sample::Thing<fn() -> u8> as core::clone::Clone>::clone"),
            "sample::Thing<>::clone",
        )

    def test_nested_qualified_owner_preserves_its_projection_and_method(self):
        self.assertEqual(
            family("<<sample::T as sample::Owner>::Associated as core::clone::Clone>::clone"),
            "sample::T::Associated::clone",
        )

    def test_inherent_qualified_owner_remains_a_namespace(self):
        self.assertEqual(
            family("<sample::Registry>::intern::<sample::Application>"),
            "sample::Registry::intern::<>",
        )

    def test_cargo_selects_the_named_successful_archive_from_utf16_output(self):
        with tempfile.TemporaryDirectory() as directory:
            messages = Path(directory) / "build.jsonl"
            messages.write_text("\n".join([
                "A non-JSON diagnostic line.",
                json.dumps({"reason": "compiler-artifact", "target": {"name": "fixture"},
                            "filenames": ["fresh.rlib", "fresh.rmeta"]}),
                json.dumps({"reason": "build-finished", "success": True}),
            ]), encoding="utf-16")
            self.assertEqual(cargo_archive(messages, "fixture"), Path("fresh.rlib"))

    def test_failed_build_cannot_select_a_stale_archive(self):
        with tempfile.TemporaryDirectory() as directory:
            messages = Path(directory) / "build.jsonl"
            messages.write_text("\n".join([
                json.dumps({"reason": "compiler-artifact", "target": {"name": "fixture"},
                            "filenames": ["stale.rlib"]}),
                json.dumps({"reason": "build-finished", "success": False}),
            ]), encoding="utf-8")
            with self.assertRaisesRegex(RuntimeError, "successful build"):
                cargo_archive(messages, "fixture")


if __name__ == "__main__":
    unittest.main()
