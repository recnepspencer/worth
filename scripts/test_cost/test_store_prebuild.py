import json
from pathlib import Path
import tempfile
import unittest

from store_prebuild import ExecutablePrebuildFailure, retain


class StoreExecutablePrebuildTests(unittest.TestCase):
    def test_only_independently_compiled_binary_can_be_retained(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            binary = root / 'observer.exe'
            binary.write_bytes(b'ordinary independently built executable')
            request = dict(package='owner', binary='observer', features=[])
            event = dict(reason='compiler-artifact', package_id='path+file:///owner#owner@1',
                         target={'name': 'observer', 'kind': ['bin']},
                         profile={'test': False}, features=[], executable=str(binary))
            env = retain([event], root / 'retained', [request])
            manifest = json.loads(Path(env['WORTH_PREBUILT_EXECUTABLES']).read_text())
            retained = Path(manifest['executables'][0]['path'])
            self.assertEqual(retained.read_bytes(), binary.read_bytes())
            self.assertNotEqual(retained, binary)
            with self.assertRaises(ExecutablePrebuildFailure):
                retain([dict(event, profile={'test': True})], root / 'retained', [request])

    def test_missing_binary_cannot_be_replaced_by_the_test_process(self):
        with tempfile.TemporaryDirectory() as temporary:
            with self.assertRaises(ExecutablePrebuildFailure):
                retain([], temporary, [dict(package='owner', binary='absent', features=[])])


if __name__ == '__main__':
    unittest.main()
