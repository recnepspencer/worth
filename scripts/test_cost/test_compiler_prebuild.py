from pathlib import Path
import tempfile
import unittest

from compiler_prebuild import normalized_source_key, unit_inputs


class CompilerPrebuildInputTests(unittest.TestCase):
    def test_unit_world_preserves_real_private_and_wrong_reason_compiler_inputs(self):
        with tempfile.TemporaryDirectory() as temporary:
            root, environment = unit_inputs(temporary)
            self.assertEqual(len(environment['sources']), 3)
            self.assertEqual(environment['environment_class'], 'production')
            self.assertIn('sealed_dependency::private()', (root / 'fixtures/private_a.rs').read_text())
            self.assertIn('absent_dependency::private', (root / 'fixtures/unresolved.rs').read_text())
            self.assertIn('OPT_LEVEL', (root / 'dependency/build.rs').read_text())

    def test_source_key_is_a_concrete_canonical_path(self):
        path = normalized_source_key(Path(__file__))
        self.assertNotIn('\\', path)
        self.assertTrue(path.endswith('/test_compiler_prebuild.py'))


if __name__ == '__main__':
    unittest.main()
