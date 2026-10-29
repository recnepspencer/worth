"""Behaviour proofs over real, offline Cargo fixture graphs."""
from collections import Counter
import copy
import json
import re
import stat
from pathlib import Path
import shutil
import subprocess
import tempfile
import tomllib
import unittest

from cargo_inventory import metadata
from court_plan import build
from court_packing import area_suffix
from frontier_adapter import surface
from module_tree import read_tree, test_names
from plan_effects import apply
from rust_source import crate_uses, modules
from rust_source import pairs, tokens
from frontier_adapter import frontiers
from filesystem_path import filesystem_path

FIXTURE = Path(__file__).parent / 'fixtures/frontiers'


class CourtCodemodTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temporary = tempfile.TemporaryDirectory()
        cls.root = Path(cls.temporary.name)
        shutil.copytree(FIXTURE, cls.root, dirs_exist_ok=True)
        subprocess.run(['git', 'init', '-q', str(cls.root)], check=True)
        subprocess.run(['git', '-C', str(cls.root), 'add', '.'], check=True)
        cls.data = metadata(cls.root / 'Cargo.toml')
        cls.plan = build(cls.root / 'Cargo.toml', cls.root, copy.deepcopy(cls.data))

    @classmethod
    def tearDownClass(cls):
        cls.temporary.cleanup()

    def test_frontiers_group_equal_surfaces_without_widening_narrow_courts(self):
        courts = {tuple(c['frontier']): c for c in self.plan['courts']}
        self.assertEqual(set(courts), {('worth-base',), ('worth-host',), ('worth-extra', 'worth-host')})
        self.assertEqual(courts[('worth-base',)]['name'], 'worth-court-base')
        self.assertGreater(len(courts[('worth-host',)]['units']), 1)

    def test_source_evidence_removes_unused_dev_frontier(self):
        suite = next(t for t in self.plan['targets'] if t['name'] == 'suite')
        self.assertEqual(suite['available_frontier'], ['worth-extra', 'worth-host'])
        self.assertEqual(suite['refined_frontier'], ['worth-host'])
        self.assertNotIn('worth_extra', suite['named_crates'])

    def test_categories_hoist_overrides_and_leave_ambiguity_in_work_list(self):
        units = {Path(u['source']).stem: u for c in self.plan['courts'] for u in c['units']}
        expected = {'plain': 'ordinary', 'child': 'process', 'model': 'oracle', 'slow': 'cost',
                    'count': 'cost', 'isolation': 'process::serial', 'boundary': 'compile_fail'}
        self.assertEqual({name: units[name]['category'] for name in expected}, expected)
        work = [w for rows in self.plan['fixer_work_by_court'].values() for w in rows]
        self.assertTrue(any(w['kind'] == 'classification' and w['source'].endswith('ambiguous.rs') for w in work))

    def test_shared_module_compiles_at_one_testkit_owner(self):
        moves = [m for m in self.plan['effects']['moves'] if m['source'].endswith('support/mod.rs')]
        self.assertEqual(len(moves), 1)
        self.assertIn('worth-testkit-host/src/', moves[0]['destination'])
        kit = next(k for k in self.plan['testkits'] if k['name'] == 'worth-testkit-host')
        self.assertEqual(len(kit['readers']), 2)

    def test_visibility_and_cross_category_paths_are_mechanical(self):
        writes = self.plan['effects']['writes']
        declarations = next(w['content'] for w in writes if '/support/declarations.rs' in w['path'])
        self.assertIn('pub struct Value', declarations)
        self.assertIn('pub amount: u32', declarations)
        self.assertIn('pub fn shared_value', declarations)
        model = next(w['content'] for w in writes if '/model/mod.rs' in w['path'])
        self.assertIn('crate::plain::ordinary_context::cross_category_value()', model)
        self.assertNotIn('crate::fast::', model)
        self.assertTrue(any('#[cfg(not(any()))]' in w['content'] for w in writes))
        forbidden = {'testkit-visibility', 'testkit-crate-context', 'cross-category-support'}
        self.assertFalse(forbidden & set(self.plan['totals']['work_items_by_kind']))

    def test_executable_paths_have_a_named_runtime_owner_and_build_prerequisite(self):
        self.assertEqual(self.plan['executable_prerequisites'], [{'package': 'worth-host', 'binary': 'host-inspect'}])
        helper = next(w['content'] for w in self.plan['effects']['writes'] if w['path'].endswith('worth-testkit-executables/src/lib.rs'))
        self.assertIn("pub fn host_inspect() -> &'static str", helper)
        self.assertIn('std::env::consts::EXE_SUFFIX', helper)
        self.assertIn('profile.join', helper)
        self.assertTrue(any(r['kind'] == 'executable-path' for r in self.plan['path_rewrites']))

    def test_asset_rewrites_cover_include_and_manifest_join(self):
        rewrites = [r for r in self.plan['path_rewrites'] if r['source'].endswith('fast/plain.rs')]
        self.assertIn('include_str', {r['kind'] for r in rewrites})
        self.assertIn('manifest-path', {r['kind'] for r in rewrites})
        for rewrite in rewrites:
            self.assertNotEqual(rewrite['before'], rewrite['after'])

    def test_ceiling_splits_use_registered_module_areas(self):
        plan = build(self.root / 'Cargo.toml', self.root, copy.deepcopy(self.data), ceiling=55)
        self.assertGreater(len(plan['courts']), len(self.plan['courts']))
        self.assertTrue(plan['splits'])
        self.assertTrue(all(s['area'] and s['owners'] for s in plan['splits']))
        self.assertTrue(all('bucket' not in s['area'] for s in plan['splits']))
        for a in plan['courts']:
            for b in plan['courts']:
                if a['name'] < b['name'] and a['frontier'] == b['frontier']:
                    self.assertGreater(a['packing_lines'] + b['packing_lines'], 55)

    def test_plan_is_byte_deterministic(self):
        other = build(self.root / 'Cargo.toml', self.root, copy.deepcopy(self.data))
        self.assertEqual(json.dumps(self.plan, sort_keys=True), json.dumps(other, sort_keys=True))

    def test_apply_preserves_tests_and_is_a_noop_on_replay(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            shutil.copytree(FIXTURE, root, dirs_exist_ok=True)
            subprocess.run(['git', 'init', '-q', str(root)], check=True)
            subprocess.run(['git', '-C', str(root), 'add', '.'], check=True)
            before = self.named_tests(root, ['crates/worth-base/tests/base.rs',
                                           'crates/worth-host/tests/suite.rs', 'crates/worth-host/tests/wide.rs'])
            before_bodies = self.named_bodies(root, ['crates/worth-base/tests/base.rs',
                                                   'crates/worth-host/tests/suite.rs', 'crates/worth-host/tests/wide.rs'])
            first = apply(self.plan, root)
            self.assertGreater(first['moved'], 0)
            self.assertEqual(apply(self.plan, root), {'moved': 0, 'written': 0})
            after = self.named_tests(root, [f'crates/{c["name"]}/tests/{c["name"].replace("-", "_")}.rs'
                                           for c in self.plan['courts']])
            self.assertEqual(before, after)
            after_targets = [f'crates/{c["name"]}/tests/{c["name"].replace("-", "_")}.rs' for c in self.plan['courts']]
            expected = Counter()
            for body, count in before_bodies.items():
                replacements = {r['before']: r['after'] for r in self.plan['path_rewrites'] if r['before']}
                pattern = '|'.join(re.escape(v) for v in sorted(replacements, key=lambda v: (-len(v), v)))
                body = re.sub(pattern, lambda m: replacements[m[0]], body)
                expected[body] += count
            self.assertEqual(expected, self.named_bodies(root, after_targets))
            self.assertFalse((root / 'crates/worth-host/tests').exists())
            for court in self.plan['courts']:
                manifest = tomllib.loads((root / 'crates' / court['name'] / 'Cargo.toml').read_text())
                self.assertFalse(manifest['package']['autotests'])
                self.assertEqual(len(manifest['test']), 1)
            data = metadata(root / 'Cargo.toml')
            selections = {p['name']: {p['id']} for p in data['packages'] if '-court-' in p['name']}
            actual = frontiers(data, selections)
            self.assertEqual(actual, {c['name']: c['frontier'] for c in self.plan['courts']})

    def test_apply_refuses_stale_source_before_moving_anything(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            shutil.copytree(FIXTURE, root, dirs_exist_ok=True)
            source = root / self.plan['effects']['moves'][-1]['source']
            source.write_text(source.read_text() + '\n// changed\n')
            with self.assertRaisesRegex(ValueError, 'move precondition'):
                apply(self.plan, root)
            self.assertTrue(all((root / m['source']).exists() for m in self.plan['effects']['moves']))

    def test_apply_supports_deep_destination_paths(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder) / ('nested_destination_' * 7)
            shutil.copytree(FIXTURE, filesystem_path(root))
            subprocess.run(['git', 'init', '-q', str(root)], check=True)
            subprocess.run(['git', '-C', str(root), 'add', '.'], check=True)
            apply(self.plan, root)
            self.assertEqual(apply(self.plan, root), {'moved': 0, 'written': 0})
            self.assertTrue(all(filesystem_path(root / m['destination']).is_file() for m in self.plan['effects']['moves']))
            self.assertTrue(root.resolve().is_relative_to(Path(folder).resolve()))
            def writable(function, path, error):
                Path(path).chmod(stat.S_IWRITE)
                function(path)
            shutil.rmtree(filesystem_path(root), onexc=writable)

    def test_source_scan_excludes_comments_strings_and_shadow_names(self):
        source = '// use unused::Thing;\nlet s = "unused::Thing"; use host as h; extern crate extra; h::run();'
        self.assertEqual(crate_uses(source, {'unused', 'host', 'extra'}), {'host', 'extra'})
        self.assertEqual(len(modules('fn f() { mod hidden {} } #[path = "x.rs"] pub(crate) mod real;')), 1)

    def test_naming_matches_the_frontier_authoritys_examples(self):
        self.assertEqual(surface('sample', []), 'source')
        self.assertEqual(area_suffix(('foundational', 'foundational_values'), 'foundational'), 'values')
        self.assertEqual(area_suffix(('store', 'physical_blob_journeys'), 'physical'), 'store-blob-journeys')
        self.assertEqual(surface('worth-query', ['worth-query', 'worth-query-host']), 'query-host')
        self.assertEqual(surface('worth-query-bank', ['bank-external-rail', 'bank-server']), 'external-rail-server')

    @staticmethod
    def named_tests(root, targets):
        result = Counter()
        for target in targets:
            for node in read_tree(root / target).walk():
                result.update(test_names(node.own_text()))
        return result

    @staticmethod
    def named_bodies(root, targets):
        result = Counter()
        for target in targets:
            for node in read_tree(root / target).walk():
                text = node.own_text()
                names, stream = set(test_names(text)), tokens(text)
                matching = pairs(stream)
                for i, token in enumerate(stream[:-1]):
                    if token.value == 'fn' and stream[i + 1].value in names:
                        opening = next(j for j in range(i + 2, len(stream)) if stream[j].value == '{')
                        result[text[stream[opening].start:stream[matching[opening]].end]] += 1
        return result


if __name__ == '__main__':
    unittest.main()
