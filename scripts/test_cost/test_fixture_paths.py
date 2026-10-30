from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

from fixture_paths import rewrite


class FixturePathRelocationTests(unittest.TestCase):
    def test_standalone_entrypoint_has_no_import_order_dependency(self):
        command = [sys.executable, str(Path(__file__).with_name('fixture_paths.py')), '--help']
        result = subprocess.run(command, capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_path_attribute_keeps_its_original_existing_producer_target(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            original = root / 'owner/tests/ui/case.rs'
            current = root / 'court/tests/sources/owner/ui/case.rs'
            target = root / 'owner/src/leaf.rs'
            target.parent.mkdir(parents=True)
            target.write_text('pub struct Existing;')
            current.parent.mkdir(parents=True)
            text = '#[path="../../src/leaf.rs"] mod leaf;'
            result, observations = rewrite(text, current, original)
            self.assertEqual(observations[0]['status'], 'relocated')
            self.assertIn('owner/src/leaf.rs', result)

    def test_missing_original_is_reported_without_inventing_support(self):
        result, observations = rewrite('#[path="absent.rs"] mod absent;', Path('new/a.rs'), Path('old/a.rs'))
        self.assertEqual(result, '#[path="absent.rs"] mod absent;')
        self.assertEqual(observations[0]['status'], 'unresolved')


if __name__ == '__main__':
    unittest.main()
