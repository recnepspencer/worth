"""Behaviour proofs for the in-source test family inventory and private-access audit."""
from pathlib import Path
import shutil
import tempfile
import unittest

from src_audit import audit, markdown
from src_families import families, test_gated
from src_paths import use_trees

FIXTURE = Path(__file__).parent / 'fixtures/src_families'


class SourceFamilyAuditTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temporary = tempfile.TemporaryDirectory()
        cls.root = Path(cls.temporary.name)
        shutil.copytree(FIXTURE, cls.root / 'fixture')
        crate = cls.root / 'fixture'
        cls.plan = audit(crate / 'src/lib.rs', cls.root, crate / 'baseline.json', crate / 'audit.json')
        cls.rows = {row['module']: row for row in cls.plan['families']}

    @classmethod
    def tearDownClass(cls):
        cls.temporary.cleanup()

    def test_cfg_predicates_gate_only_code_absent_without_test(self):
        self.assertTrue(test_gated('#[cfg(test)]'))
        self.assertTrue(test_gated('#[cfg(all(test, feature = "x"))]'))
        self.assertFalse(test_gated('#[cfg(not(test))]'))
        self.assertFalse(test_gated('#[cfg(any(test, feature = "x"))]'))
        self.assertFalse(test_gated('#[cfg_attr(test, allow(dead_code))]'))

    def test_inventory_finds_outermost_gated_modules_with_identities(self):
        _tree, found = families(self.root / 'fixture/src/lib.rs')
        self.assertEqual(sorted('::'.join(f.names) for f in found),
                         ['inner::hook_tests', 'inner::public_tests', 'inner::tests', 'world'])
        tests = {f.names[-1]: f for f in found}
        self.assertEqual([name for name, _digest in tests['tests'].tests], ['helper_returns_its_input'])
        self.assertEqual(tests['hook_tests'].macro_tests, [{'macro': 'twice', 'invocations': 1}])
        self.assertNotIn('production_only', {f.names[-1] for f in found})

    def test_private_logic_stays_and_public_surface_moves(self):
        self.assertEqual(self.rows['inner::tests']['decision'], 'private-logic')
        self.assertIn('inner::helper', self.rows['inner::tests']['private_needs'])
        self.assertIn('inner::Hidden', self.rows['inner::tests']['private_needs'])
        self.assertEqual(self.rows['inner::public_tests']['decision'], 'public-surface')
        self.assertEqual(self.rows['inner::public_tests']['private_needs'], [])

    def test_runtime_root_with_test_hooks_is_a_blocked_court(self):
        row = self.rows['inner::hook_tests']
        self.assertEqual(row['decision'], 'runtime-root-private')
        self.assertEqual(row['runtime_roots'], ['FixtureRuntime'])
        self.assertIn('inner::hook', row['test_hooks'])
        self.assertIn('.probe_for_test', row['test_hooks'])
        self.assertNotIn('.real', row['test_hooks'])
        self.assertEqual(row['support_from'], ['world'])

    def test_support_without_tests_follows_its_readers(self):
        self.assertEqual(self.rows['world']['decision'], 'support')

    def test_report_is_deterministic_and_checkout_independent(self):
        crate = self.root / 'fixture'
        again = audit(crate / 'src/lib.rs', self.root, crate / 'baseline.json', crate / 'audit.json')
        self.assertEqual(markdown(again), markdown(self.plan))
        self.assertNotIn(str(self.root), markdown(self.plan))
        self.assertEqual(self.plan['library'], 'fixture/src/lib.rs')

    def test_use_trees_flatten_groups_self_globs_and_aliases(self):
        leaves = use_trees('use crate::a::{self, b::C as D, e::*};')
        self.assertEqual(sorted(leaves, key=str), sorted([
            (('crate', 'a'), False, None), (('crate', 'a', 'b', 'C'), False, 'D'),
            (('crate', 'a', 'e'), True, None)], key=str))


if __name__ == '__main__':
    unittest.main()
