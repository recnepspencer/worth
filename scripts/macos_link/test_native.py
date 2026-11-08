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

    def test_historical_1964_and_1966_images_refuse_but_driver_outputs_are_high(self):
        from .diagnostic import diagnose
        for mib in [1964, 1966]:
            with self.subTest(mib=mib), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                source, original, fixed = root/'body.c', root/'old', root/'fixed'
                source.write_text(f'static volatile char body[{mib}UL*1024*1024];\n'
                                  'int main(void){body[0]=7;return body[0]!=7;}\n')
                subprocess.run(['/usr/bin/cc', str(source), '-o', str(original)],
                               check=True, capture_output=True)
                result = subprocess.run([str(ROOT/'scripts/check_macos_layout.py'), str(original)],
                                        capture_output=True, text=True)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn('dyld placement', result.stderr)
                self.assertIn('dyld placement', diagnose(original))
                subprocess.run([str(ROOT/'scripts/macos_linker'), str(source), '-o', str(fixed)],
                               check=True, capture_output=True)
                validate(read(fixed))
                self.assertGreaterEqual(max(s.end for s in read(fixed).mapped), HIGH_BASE)
                subprocess.run([str(fixed)], check=True, capture_output=True)

    def test_declared_interpreter_safe_path_and_cc_path_override(self):
        import os
        import shlex
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            marker = root/'selected'
            python = root/'python3'
            python.write_text('#!/bin/sh\nexit 93\n')
            cc = root/'cc'
            cc.write_text('#!/bin/sh\necho selected >> '+shlex.quote(str(marker))+
                          '\nexec /usr/bin/cc "$@"\n')
            python.chmod(0o755)
            cc.chmod(0o755)
            source = root/'one.c'
            source.write_text('int main(void){return 0;}\n')
            environment = dict(os.environ, PATH=str(root)+os.pathsep+os.environ['PATH'],
                               PYTHONSAFEPATH='1')
            output = root/'one'
            subprocess.run([str(ROOT/'scripts/macos_linker'), str(source), '-o', str(output)],
                           env=environment, check=True, capture_output=True)
            self.assertIn('selected', marker.read_text())
            subprocess.run([str(ROOT/'scripts/check_macos_layout.py'), str(output)],
                           env=environment, check=True, capture_output=True)

    def test_real_response_file_with_spaces_quotes_and_backslashes_is_forwarded(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root/"an object's\\name.c"
            source.write_text('int main(void){return 0;}\n')
            output = root/'final output'
            quote = lambda value: '"'+str(value).replace('\\', '\\\\').replace('"', '\\"')+'"'
            response = root/'link.rsp'
            response.write_text(' '.join(map(quote, [source, '-o', output])))
            subprocess.run([str(ROOT/'scripts/macos_linker'), '@'+str(response)],
                           check=True, capture_output=True)
            validate(read(output))
            subprocess.run([str(output)], check=True, capture_output=True)
