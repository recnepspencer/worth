"""Loader placement is a sourced calculation, including its independent slide."""
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
from .loader import LoaderBounds, read_bounds
from .macho import Image, Segment, InvalidImage, read
from .placement import (CACHE_BASE, MAX_SLIDE, PAGE, validate_historical,
                        LoaderBoundsUnavailable, UnsafeLayout)


class LoaderPlacementTests(unittest.TestCase):
    def test_low_top_includes_both_slides_loader_extent_and_page_rounding(self):
        bounds = LoaderBounds(2**22, 0x16c000)
        end = CACHE_BASE - MAX_SLIDE - bounds.slide - bounds.extent
        image = lambda top: Image(2, 0x200000, (Segment('__TEXT', 2**32, top-2**32, 5),), False)
        with patch('scripts.macos_link.placement.host_loader', return_value=bounds):
            validate_historical(image(end))
            with self.assertRaisesRegex(UnsafeLayout, 'dyld placement'):
                validate_historical(image(end + 1))
            for gap in [82 * 1024**2, 84 * 1024**2]:
                with self.assertRaisesRegex(UnsafeLayout, 'dyld placement'):
                    validate_historical(image(CACHE_BASE - gap))

    def test_unavailable_loader_evidence_is_a_typed_refusal(self):
        with tempfile.TemporaryDirectory() as directory:
            with self.assertRaises(LoaderBoundsUnavailable):
                read_bounds(Path(directory) / 'missing-dyld')
        with patch('scripts.macos_link.placement.host_loader', side_effect=LoaderBoundsUnavailable('unavailable')):
            with self.assertRaises(LoaderBoundsUnavailable):
                validate_historical(Image(2, 0x200000, (Segment('__TEXT', 2**32, PAGE, 5),), False))

    def test_universal_loader_selects_arm64_load_commands(self):
        import struct
        segment = struct.pack('<II16sQQQQiiII', 0x19, 72, b'__TEXT',
                              0, 0x4000, 0, 104, 5, 5, 0, 0)
        arm = struct.pack('<8I', 0xfeedfacf, 0x100000c, 0, 7, 1, 72, 0, 0)+segment
        fat = struct.pack('>II', 0xcafebabe, 3)
        fat += struct.pack('>5I', 0x1000007, 3, 0x100, 32, 0)
        fat += struct.pack('>5I', 0x100000c, 0, 0x200, len(arm), 0)
        fat += struct.pack('>5I', 0x100000c, 2, 0x300, len(arm), 0)
        variant = bytearray(arm)
        struct.pack_into('<Q', variant, 64, 0x8000)
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory)/'dyld'
            path.write_bytes(fat+b'\0'*(0x200-len(fat))+arm+b'\0'*(0x100-len(arm))+variant)
            self.assertEqual(read_bounds(path), LoaderBounds(2**22, 0x8000))
            with self.assertRaises(InvalidImage):
                read(path)
