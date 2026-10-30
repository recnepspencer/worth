import json
from pathlib import Path
import subprocess
import tempfile
import unittest

from nextest_build import LANES, build, list_tests, summary


class PrebuiltNextestTests(unittest.TestCase):
    def test_pretty_inventory_is_separated_from_compiler_jsonl(self):
        text = '{"reason":"compiler-artifact"}\n' + json.dumps({'rust-binaries': {'court': {}}}, indent=2)
        self.assertEqual(summary(text, 'rust-binaries'), {'rust-binaries': {'court': {}}})

    def test_build_is_once_and_listing_uses_only_prebuilt_metadata(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            commands = []

            def runner(command, cwd, log):
                commands.append(command)
                key = 'rust-binaries' if 'binaries-only' in command else 'rust-suites'
                return subprocess.CompletedProcess(command, 0, json.dumps({key: {}}), '')

            reuse = build(root / 'Cargo.toml', root / 'out', runner)
            list_tests(reuse, root / 'out/inventory.json', runner)
            self.assertEqual(len(commands), 3)
            self.assertIn('--binaries-metadata', commands[-1])
            self.assertIn('--ignore-default-filter', commands[-1])
            self.assertNotIn('--manifest-path', commands[-1])
            self.assertNotIn('--all-targets', commands[-1])
            self.assertNotIn('--include-ignored', ' '.join(' '.join(c) for c in commands))

    def test_category_lanes_use_paths_and_ordinary_excludes_them(self):
        self.assertIn('test(/^compile_fail::/)', LANES['compile-fail'])
        self.assertIn('not ', LANES['ordinary'])


if __name__ == '__main__':
    unittest.main()
