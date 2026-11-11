"""Address geometry is the policy; file byte counts are deliberately irrelevant."""
from pathlib import Path
import struct
import tempfile
import unittest
from unittest.mock import patch

from .macho import Image, InvalidImage, Segment, read
from .placement import (CACHE_BASE, HIGH_BASE, MAX_SLIDE, USER_END, UnsafeLayout,
                        relocated_arguments, validate_historical, validate)
from .driver import expanded, executable_link, link


def image(*segments, tls=False):
    return Image(2, 0x200000, tuple(segments), tls)


def segment(name, start, size):
    return Segment(name, start, size, 1)


class GeometryTests(unittest.TestCase):
    def setUp(self):
        from .loader import LoaderBounds
        bounds = patch('scripts.macos_link.placement.host_loader',
                       return_value=LoaderBounds(4 * 1024**2, 0x16c000))
        bounds.start()
        self.addCleanup(bounds.stop)

    def test_safe_low_code_and_high_metadata_have_no_image_size_ceiling(self):
        for extent in (1024, 2**31, 2**37):
            validate_historical(image(segment('__TEXT', 2**32, 1024),
                           segment('__LINKEDIT', HIGH_BASE, extent)))

    def test_slide_range_is_reserved_even_before_the_unslid_cache(self):
        with self.assertRaisesRegex(UnsafeLayout, 'ASLR mappings'):
            validate_historical(image(segment('__TEXT', 2**32, CACHE_BASE - 2**32 - 1)))
        validate_historical(image(segment('__TEXT', 2**32, CACHE_BASE - 2**32 - MAX_SLIDE - 4 * 1024**2 - 0x16c000)))
        with self.assertRaises(UnsafeLayout):
            validate_historical(image(segment('__TEXT', 64 * 1024**3, 1024)))

    def test_non_tls_translation_preserves_relative_addresses(self):
        placement = relocated_arguments(image(segment('__TEXT', 2**32, 0x4000),
                                             segment('__DATA', 2**32 + 0x4000, 2**31)))
        self.assertIn(f'-Wl,-segaddr,__TEXT,{HIGH_BASE:#x}', placement)
        self.assertIn(f'-Wl,-segaddr,__DATA,{HIGH_BASE + 0x4000:#x}', placement)
        with self.assertRaisesRegex(UnsafeLayout, 'TLS'):
            relocated_arguments(image(segment('__TEXT', 2**32, 2**31), tls=True))

    def test_only_the_real_unmapped_pagezero_is_exempt(self):
        validate_historical(image(Segment('__PAGEZERO', 0, 2**32, 0),
                       segment('__TEXT', 2**32, 1024)))
        with self.assertRaises(UnsafeLayout):
            validate_historical(image(Segment('__GUARD', CACHE_BASE, 1024, 0)))
        with self.assertRaises(UnsafeLayout):
            validate_historical(image(Segment('__PAGEZERO', CACHE_BASE, 1024, 0)))

    def test_high_placement_still_requires_the_os_user_address_range(self):
        with self.assertRaises(UnsafeLayout):
            validate_historical(image(segment('__LINKEDIT', USER_END, 1024)))

    def test_main_slide_still_applies_to_low_segments_when_loader_is_high(self):
        with self.assertRaises(UnsafeLayout):
            validate(image(segment('__TEXT', 2**32, CACHE_BASE - 2**32 - 40 * 1024**2),
                           segment('__LINKEDIT', HIGH_BASE, 0x4000)))

    def test_driver_cannot_publish_an_otherwise_safe_low_top(self):
        low = image(segment('__TEXT', 2**32, 0x4000))
        validate_historical(low)
        with self.assertRaisesRegex(UnsafeLayout, 'top mapped segment'):
            validate(low)

    def test_missing_pie_is_a_refusal(self):
        with self.assertRaisesRegex(UnsafeLayout, 'PIE'):
            validate_historical(Image(2, 0, (segment('__TEXT', 2**32, 1024),), False))

    def test_malformed_format_is_not_admitted(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'image'
            for contents in (b'ELF', struct.pack('<8I', 0xfeedfacf, 0x100000c,
                                               0, 2, 1, 100, 0x200000, 0)):
                path.write_bytes(contents)
                with self.assertRaises(InvalidImage):
                    read(path)

    def test_tls_sections_are_read_from_real_load_commands(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'image'
            section = bytearray(80)
            struct.pack_into('<I', section, 64, 0x13)
            command = struct.pack('<II16sQQQQiiII', 0x19, 152, b'__DATA',
                                  2**32, 0x4000, 0, 0, 3, 3, 1, 0) + section
            path.write_bytes(struct.pack('<8I', 0xfeedfacf, 0x100000c, 0, 2,
                                         1, len(command), 0x200000, 0) + command)
            self.assertTrue(read(path).tls)


class DriverTests(unittest.TestCase):
    def test_response_files_and_non_executable_links_keep_original_arguments(self):
        with tempfile.TemporaryDirectory() as directory:
            response = Path(directory) / 'args'
            response.write_text('-dynamiclib "object with spaces.o" -o library.dylib')
            arguments = ['@' + str(response)]
            self.assertEqual(expanded(arguments)[1], 'object with spaces.o')
            self.assertFalse(executable_link(expanded(arguments)))
            with patch('scripts.macos_link.driver.subprocess.call', return_value=0) as call:
                self.assertEqual(link(arguments), 0)
            call.assert_called_once_with(['cc', *arguments])
        for flag in ('-shared', '-bundle', '-r', '-Wl,-r'):
            self.assertFalse(executable_link([flag]))

    def test_failed_link_is_not_retried_or_published(self):
        with tempfile.TemporaryDirectory() as directory:
            destination = Path(directory) / 'artifact'
            with patch('scripts.macos_link.driver.subprocess.call', return_value=3) as call:
                self.assertEqual(link(['object.o', '-o', str(destination)]), 3)
            self.assertEqual(call.call_count, 1)
            self.assertFalse(destination.exists())
            self.assertFalse(destination.with_name('artifact.layout-pending').exists())

    def test_rustflags_are_preserved_and_only_arm64_darwin_selects_the_adapter(self):
        import tomllib
        config = tomllib.loads(Path('.cargo/config.toml').read_text())
        flags = config['build']['rustflags']
        self.assertIn('overflow-checks=on', flags)
        self.assertIn('worth_ui_windowing="wayland"', flags)
        self.assertIn('worth_ui_adapter="hardware"', flags)
        targets = config['target']
        self.assertEqual(targets['aarch64-apple-darwin'], {'linker': 'scripts/macos_linker'})
        self.assertEqual(targets['x86_64-pc-windows-msvc'], {'linker': 'scripts/windows_linker.cmd'})
        self.assertNotIn('x86_64-unknown-linux-gnu', targets)
