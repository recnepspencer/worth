"""Unwind parsing, attribution and refusal of the frame budget lane."""

import struct
import sys
import unittest
import unittest.mock
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import frame_budget  # noqa: E402


def unwind_info(codes: list[int], chain: tuple[int, int, int] | None = None) -> bytes:
    """An UNWIND_INFO record at offset 0 with the given 16-bit code slots."""
    flags = frame_budget.UNW_FLAG_CHAININFO if chain else 0
    header = bytes([1 | (flags << 3), 0, len(codes), 0])
    body = b"".join(struct.pack("<H", code) for code in codes)
    if len(codes) % 2:
        body += b"\0\0"
    if chain:
        body += struct.pack("<III", *chain)
    return header + body


def slot(operation: int, info: int) -> int:
    return (operation | (info << 4)) << 8


class UnwindTests(unittest.TestCase):
    def frame(self, image: bytes, rva: int = 0) -> int:
        return frame_budget.unwind_frame_bytes(image, lambda value: value, rva)

    def test_a_frameless_leaf_allocates_nothing(self):
        self.assertEqual(self.frame(unwind_info([])), 0)

    def test_pushes_and_a_small_allocation_count_their_bytes(self):
        codes = [slot(2, 5), slot(0, 3), slot(0, 6)]
        self.assertEqual(self.frame(unwind_info(codes)), 5 * 8 + 8 + 16)

    def test_a_large_allocation_reads_its_32_bit_operand(self):
        size = 600 * 1024
        codes = [slot(1, 1), size & 0xFFFF, size >> 16]
        self.assertEqual(self.frame(unwind_info(codes)), size)

    def test_a_scaled_large_allocation_reads_its_16_bit_operand(self):
        self.assertEqual(self.frame(unwind_info([slot(1, 0), 4096])), 4096 * 8)

    def test_saved_registers_take_slots_without_stack(self):
        codes = [slot(4, 3), 0x10, slot(9, 6), 0, 1, slot(2, 1)]
        self.assertEqual(self.frame(unwind_info(codes)), 16)

    def test_a_chained_fragment_adds_its_primary_allocation(self):
        primary = unwind_info([slot(1, 0), 1000])
        fragment = unwind_info([slot(0, 3)], chain=(0x100, 0x200, 0))
        image = primary + fragment
        rva = len(primary)
        self.assertEqual(self.frame(image, rva), 8000 + 8)
        start = frame_budget.primary_start(image, lambda value: value, rva, 0x300)
        self.assertEqual(start, 0x100)

    def test_a_cyclic_chain_refuses(self):
        image = unwind_info([], chain=(0, 0, 0))
        with self.assertRaises(frame_budget.UnwindError):
            self.frame(image)

    def test_an_unknown_operation_refuses(self):
        with self.assertRaises(frame_budget.UnwindError):
            self.frame(unwind_info([slot(11, 0)]))


class AttributionTests(unittest.TestCase):
    def test_generic_arguments_and_impl_markers_leave_the_defining_path(self):
        path = frame_budget.owner_path(
            "worth_store::checkpoint::impl$3::capture<enum2$<core::option::Option<u8> >,ref$<str$> >"
        )
        self.assertEqual(path, "worth_store::checkpoint::capture")

    def test_function_arrows_do_not_close_generic_arguments(self):
        path = frame_budget.owner_path("worth_query::run<fn() -> u8>::closure$0")
        self.assertEqual(path, "worth_query::run::closure$0")

    def test_a_foreign_generic_over_an_owned_type_is_not_owned(self):
        path = frame_budget.owner_path("core::ptr::drop_in_place<worth_store::Home>")
        self.assertFalse(frame_budget.owned(path, ["worth_"]))
        self.assertTrue(frame_budget.owned("worth_store::Home::open", ["worth_"]))

    def test_rows_keep_the_largest_frame_across_binaries(self):
        rows = frame_budget.deduplicate(
            [
                frame_budget.Frame("worth_a::f", 40000, "one"),
                frame_budget.Frame("worth_a::f", 70000, "two"),
                frame_budget.Frame("worth_b::g", 50000, "one"),
            ]
        )
        self.assertEqual([(row["path"], row["bytes"]) for row in rows], [("worth_a::f", 70000), ("worth_b::g", 50000)])
        self.assertEqual(rows[0]["binaries"], ["one", "two"])
        self.assertEqual(frame_budget.bands(rows)["64-128KiB"], 1)


def pdb(guid: bytes, age: int, block: int = 512) -> bytes:
    """A minimal MSF 7.00 file: directory, stream 0 and the information stream."""
    info = struct.pack("<III", 20000404, 0, age) + guid
    directory = struct.pack("<I2I", 2, 0, len(info)) + struct.pack("<I", 4)
    blocks = [b"", b"", b"", struct.pack("<I", 5), info, directory]
    header = frame_budget.MSF_MAGIC + struct.pack("<6I", block, 1, len(blocks), len(directory), 0, 3)
    blocks[0] = header
    return b"".join(chunk.ljust(block, b"\0") for chunk in blocks)


def pe_with_codeview(guid: bytes, age: int) -> frame_budget.PeImage:
    """A PE32+ image whose only section holds one CodeView debug entry."""
    data = bytearray(0x400)
    struct.pack_into("<I", data, 0x3C, 0x80)
    data[0x80:0x84] = b"PE\0\0"
    struct.pack_into("<H", data, 0x86, 1)  # one section
    struct.pack_into("<H", data, 0x94, 240)  # optional header size
    optional = 0x98
    struct.pack_into("<H", data, optional, 0x20B)
    struct.pack_into("<II", data, optional + 112 + 6 * 8, 0x1000, 28)  # debug directory
    section = optional + 240
    struct.pack_into("<IIII", data, section + 8, 0x200, 0x1000, 0x200, 0x200)
    struct.pack_into("<IIII", data, 0x200 + 12, 2, 24, 0, 0x300)
    data[0x300:0x304] = b"RSDS"
    data[0x304:0x314] = guid
    struct.pack_into("<I", data, 0x314, age)
    return frame_budget.PeImage.parse(bytes(data))


class IdentityTests(unittest.TestCase):
    guid = bytes(range(16))

    def test_a_pdb_reports_its_information_stream_identity(self):
        self.assertEqual(frame_budget.pdb_identity(pdb(self.guid, 7)), (self.guid, 7))

    def test_an_executable_reports_its_codeview_identity(self):
        self.assertEqual(pe_with_codeview(self.guid, 7).codeview(), (self.guid, 7))

    def test_an_executable_with_another_builds_pdb_refuses(self):
        directory = Path(self.id().replace(".", "_"))
        image = pe_with_codeview(self.guid, 7)
        with unittest.mock.patch.object(Path, "is_file", return_value=True), unittest.mock.patch.object(
            Path, "read_bytes", return_value=pdb(self.guid, 8)
        ):
            with self.assertRaisesRegex(frame_budget.UnwindError, "does not belong"):
                frame_budget.require_matching_pdb(directory / "court.exe", image)
            with unittest.mock.patch.object(Path, "read_bytes", return_value=pdb(self.guid, 7)):
                frame_budget.require_matching_pdb(directory / "court.exe", image)

    def test_only_the_wanted_executables_of_this_build_are_measured(self):
        messages = "\n".join(
            [
                '{"reason":"compiler-artifact","target":{"name":"court"},"executable":"C:/t/court-1.exe"}',
                '{"reason":"compiler-artifact","target":{"name":"worth_store"},"executable":null}',
                '{"reason":"compiler-artifact","target":{"name":"other"},"executable":"C:/t/other.exe"}',
                "   Compiling worth_store v0.1.0",
                '{"reason":"build-finished","success":true}',
            ]
        )
        self.assertEqual(frame_budget.built_executables(messages, ["court"]), {"court": Path("C:/t/court-1.exe")})


if __name__ == "__main__":
    unittest.main()
