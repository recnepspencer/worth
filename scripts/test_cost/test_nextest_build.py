import json
from pathlib import Path
import subprocess
import tempfile
import unittest
import re
import tomllib

from nextest_build import LANES, build, list_tests, summary
from workspace import members, repository_root


class PrebuiltNextestTests(unittest.TestCase):
    def test_committed_store_binary_selectors_name_real_court_targets(self):
        root = repository_root(Path(__file__).parent)
        manifest = root / 'workspaces/worth-store/Cargo.toml'
        real = set()
        for package in members(manifest):
            data = tomllib.loads(package['manifest'].read_text(encoding='utf-8'))
            real.update(package['name'] + '::' + target['name'] for target in data.get('test', []))
        config = tomllib.loads((manifest.parent / '.config/nextest.toml').read_text())
        for profile in config['profile'].values():
            for override in profile.get('overrides', []):
                for binary in re.findall(r'binary_id\(=([^)]*)\)', override['filter']):
                    self.assertIn(binary, real)
        self.assertNotIn('compiler-boundary', config.get('test-groups', {}))

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
