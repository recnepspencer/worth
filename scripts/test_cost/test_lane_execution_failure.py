"""Completed failures transport actual observations; selection controls full scope."""
import json
from pathlib import Path
import tempfile
import unittest

from run_lane import LANDING, LaneExecutionFailure, full_workspace_required, retain_completed


class CompletedExecutionTests(unittest.TestCase):
    def test_budget_failure_retains_the_exact_completed_documentation_and_native_result(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            output = root/'run';output.mkdir()
            snippet = {'binary': 'owner::doc', 'test': 'src/lib.rs (line 3)', 'ignored': False}
            result = {'manifest': 'Cargo.toml', 'source_counts': {}, 'source_sites': [], 'lanes': {},
                      'coverage': {'status': 'pass'}, 'llvm_lines': {},
                      'documentation': {'inventory': [snippet], 'selections': [snippet], 'status': 'pass'},
                      'executed_coverage': {'executed': 2, 'passed': 2}}
            with self.assertRaises(LaneExecutionFailure) as raised:
                retain_completed(root, 'root', output, result, [{'kind': 'duplicate-graph'}], [])
            self.assertIs(raised.exception.result, result)
            self.assertIn('build budget exceeded', str(raised.exception))
            self.assertEqual(raised.exception.result['documentation']['selections'], [snippet])
            self.assertEqual(json.loads((output/'census.json').read_text()), result)
            self.assertEqual(json.loads((root/'target/test-cost/root/census.json').read_text()), result)

    def test_a_full_callback_with_docs_enforces_the_complete_workspace(self):
        self.assertTrue(full_workspace_required([], [('suite', 'all()')], None, None, True, {'status': 'pass'}))
        self.assertFalse(full_workspace_required([], [('suite', 'partial')], None, None, True, {'status': 'fail'}))
        self.assertFalse(full_workspace_required([], [('suite', 'all()')], None, None, False, {'status': 'pass'}))

    def test_default_full_plan_still_fails_hard_if_structural_selection_is_incomplete(self):
        self.assertTrue(full_workspace_required(LANDING, None, None, None, True, {'status': 'fail'}))
        self.assertFalse(full_workspace_required(LANDING, None, None, 'hash:1/2', False, {'status': 'fail'}))

    def test_test_failure_is_a_value_error_with_the_completed_result(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary);output = root/'run';output.mkdir()
            result = {'manifest': 'Cargo.toml', 'source_counts': {}, 'source_sites': [], 'lanes': {},
                      'coverage': {'status': 'fail'}, 'llvm_lines': {}, 'documentation': None}
            with self.assertRaises(ValueError) as raised:
                retain_completed(root, 'root', output, result, [], ['process'])
            self.assertIsInstance(raised.exception, LaneExecutionFailure)
            self.assertIs(raised.exception.result, result)
            self.assertIn('process', str(raised.exception))


if __name__ == '__main__':
    unittest.main()
