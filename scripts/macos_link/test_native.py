"""Real linker/loader regression for the zero-fill reproduction of #67."""
from pathlib import Path
import platform
import subprocess
import sys
import tempfile
import unittest
from .macho import read
from .placement import HIGH_BASE, UnsafeLayout, validate

ROOT = Path(__file__).resolve().parents[2]


@unittest.skipUnless(sys.platform == 'darwin' and platform.machine() == 'arm64',
                     'Apple arm64 linker and loader court')
class NativePlacementTests(unittest.TestCase):
    def test_in_band_geometry_is_refused_then_relocated_with_aslr_preserved(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / 'zero_fill.c'
            # Zero-fill extends virtual geometry, without a large file/allocation.
            source.write_text('static volatile char body[2000UL*1024*1024];\n'
                              'int main(void) { body[0]=7; return body[0]!=7; }\n')
            original, fixed = root / 'original', root / 'fixed'
            subprocess.run(['/usr/bin/cc', str(source), '-o', str(original)], check=True,
                           capture_output=True)
            with self.assertRaises(UnsafeLayout):
                validate(read(original))
            subprocess.run([str(ROOT / 'scripts/macos_linker'), str(source), '-o', str(fixed)],
                           check=True, capture_output=True)
            image = read(fixed)
            validate(image)
            self.assertTrue(all(s.address >= HIGH_BASE for s in image.mapped))
            self.assertTrue(image.flags & 0x200000, 'PIE/ASLR must remain enabled')
            # Execute exactly once. Repeated ASLR samples belong to qualification.
            subprocess.run([str(fixed)], check=True, capture_output=True)

    def test_unsafe_tls_layout_is_a_named_build_refusal_without_an_artifact(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source, output = root / 'tls.c', root / 'tls'
            source.write_text('_Thread_local volatile int tls;\n'
                              'static volatile char body[2000UL*1024*1024];\n'
                              'int main(void){tls=7;body[0]=1;return tls!=7;}\n')
            result = subprocess.run([str(ROOT / 'scripts/macos_linker'), str(source), '-o', str(output)],
                                    capture_output=True, text=True)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn('MACOS_EXEC_LAYOUT', result.stderr)
            self.assertIn('TLS', result.stderr)
            self.assertFalse(output.exists())
            self.assertFalse(output.with_name('tls.layout-pending').exists())
