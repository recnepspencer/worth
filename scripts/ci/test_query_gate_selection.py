"""Local defaults and full requests preserve existing categories and evidence."""
from pathlib import Path
import contextlib
import io
import json
import tempfile
import unittest
from unittest.mock import patch

from query_gate_selection import select_suites, workspace_coverage
from query_gate_courts import CATEGORY_SUITES, landing_suites
from workspace import WORKSPACES


class LocalSelectionTests(unittest.TestCase):
    def test_local_defaults_select_only_current_workspace_ordinary_owner_suites(self):
        expected = {'root': ['relational', 'signal', 'execution', 'root-rest'],
                    'query': ['query-execution', 'query-rest'], 'bank': ['bank']}
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            for workspace, wanted in expected.items():
                manifest = root/WORKSPACES[workspace]
                manifest.parent.mkdir(parents=True, exist_ok=True)
                manifest.write_text('[workspace]\nmembers=[]\n', encoding='utf-8')
                nested = manifest.parent/'crates/owner/src'; nested.mkdir(parents=True)
                selected = select_suites(None, False, None, ['full'], root, nested)
                self.assertEqual(selected, wanted)
                self.assertFalse(set(selected) & set(CATEGORY_SUITES))

    def test_nested_unknown_and_store_workspaces_require_an_explicit_selection(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            (root/'Cargo.toml').write_text('[workspace]\nmembers=[]\n', encoding='utf-8')
            for relative in ('workspaces/worth-store', 'workspaces/worth-ui',
                             'workspaces/worth-query/external-consumer'):
                directory = root/relative; directory.mkdir(parents=True)
                (directory/'Cargo.toml').write_text('[workspace]\nmembers=[]\n', encoding='utf-8')
                with self.subTest(relative=relative), self.assertRaisesRegex(ValueError, 'explicit suite'):
                    select_suites(None, False, None, [], root, directory)
                self.assertEqual(select_suites('query-execution', False, None, [], root, directory),
                                 ['query-execution'])

    def test_explicit_risk_and_full_requests_keep_existing_selection_without_ignored_suites(self):
        for workspace in ('root', 'query', 'bank'):
            for category in ('process', 'exhaustive', 'cost'):
                suite = workspace+'-'+category
                self.assertEqual(select_suites(suite, False, None, [], '.', '.'), [suite])
        full = ['boundary', 'boundary-tests', 'query-process', 'query-cost']
        self.assertEqual(select_suites(None, True, None, full, '.', '.'), full)
        root = [*landing_suites('root'), 'root-doc']
        self.assertEqual(select_suites(None, False, root, full, '.', '.'), root)
        self.assertEqual(select_suites(None, True, root, full, '.', '.'), root)
        with self.assertRaisesRegex(ValueError, 'not both'):
            select_suites('query-process', True, None, full, '.', '.')
        for workspace in ('query', 'bank'):
            selected = select_suites(workspace+'-landing', False, None, full, '.', '.')
            self.assertEqual(selected, [*landing_suites(workspace), workspace+'-doc'])
            self.assertFalse(any(name.endswith('-scale') for name in selected))

    def test_qualification_requires_every_successful_native_and_documentation_observation(self):
        suites = [*landing_suites('query'), 'query-doc']
        results = [{'suite': name, 'ok': True, 'coverage': {'status': 'pass', 'documentation': 'observed'}}
                   for name in suites]
        self.assertEqual(workspace_coverage(['query-execution'], results[:1]), {'query': 'partial'})
        self.assertEqual(workspace_coverage(suites, results), {'query': 'full'})
        self.assertEqual(workspace_coverage(suites, results[:-1]), {'query': 'partial'})
        for key, value in (('ok', False), ('coverage', {'status': 'partial', 'documentation': 'observed'}),
                           ('coverage', {'status': 'pass', 'documentation': 'unmeasured'})):
            changed = [dict(row) for row in results]; changed[0][key] = value
            self.assertEqual(workspace_coverage(suites, changed), {'query': 'partial'})
        self.assertEqual(workspace_coverage(['boundary'], [{'suite': 'boundary', 'ok': True}]), {})


class GateCliSelectionTests(unittest.TestCase):
    def exercise(self, root, arguments, *, complete=False, failure=None, root_wrapper=False):
        import query_gate
        import root_gate
        output = root/'out'
        class Gate:
            selected = []
            calls = []
            def plan(self, suites): self.selected = suites
            def run(self, suite):
                self.calls.append(suite)
                return {'suite': suite, 'ok': suite != failure,
                        'coverage': {'status': 'pass' if complete else 'partial',
                                     'documentation': 'observed' if complete else 'unmeasured'}}
        gate = Gate()
        with patch.object(query_gate, 'ROOT', root), patch.object(query_gate.Path, 'cwd', return_value=root), \
             patch.object(query_gate, 'Gate', return_value=gate), contextlib.redirect_stdout(io.StringIO()):
            main = root_gate.main if root_wrapper else query_gate.main
            code = main([*arguments, '--out', str(output)])
        return code, gate, json.loads((output/'summary.json').read_text())

    def test_default_gate_runs_only_local_ordinary_selection_and_reports_partial(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            (root/'Cargo.toml').write_text('[workspace]\nmembers=[]\n', encoding='utf-8')
            code, gate, result = self.exercise(root, [])
            self.assertEqual(code, 0)
            self.assertEqual(gate.calls, ['relational', 'signal', 'execution', 'root-rest'])
            self.assertEqual(result['workspace_coverage'], {'root': 'partial'})

    def test_explicit_risk_request_remains_partial_and_does_not_expand_to_full(self):
        with tempfile.TemporaryDirectory() as temporary:
            code, gate, result = self.exercise(Path(temporary).resolve(), ['query-process'])
            self.assertEqual(code, 0)
            self.assertEqual(gate.calls, ['query-process'])
            self.assertEqual(result['workspace_coverage'], {'query': 'partial'})

    def test_explicit_full_gate_retains_all_old_categories_and_checker_tests(self):
        import query_gate
        with tempfile.TemporaryDirectory() as temporary:
            code, gate, result = self.exercise(Path(temporary).resolve(), ['--full'], complete=True)
            self.assertEqual(code, 0)
            self.assertEqual(gate.calls, query_gate.ORDER)
            self.assertEqual(gate.calls.count('boundary-tests'), 1)
            self.assertEqual(result['workspace_coverage'], {'root': 'full', 'query': 'full', 'bank': 'full'})

    def test_failed_full_request_reports_unrun_workspaces_partial(self):
        with tempfile.TemporaryDirectory() as temporary:
            code, gate, result = self.exercise(Path(temporary).resolve(), ['--full'], failure='fmt', complete=True)
            self.assertEqual(code, 1)
            self.assertEqual(gate.calls, ['fmt'])
            self.assertEqual(result['status'], 'red')
            self.assertEqual(set(result['workspace_coverage'].values()), {'partial'})

    def test_existing_root_landing_wrapper_keeps_its_complete_default(self):
        import root_gate
        with tempfile.TemporaryDirectory() as temporary:
            code, gate, result = self.exercise(Path(temporary).resolve(), [], root_wrapper=True, complete=True)
            self.assertEqual(code, 0)
            self.assertEqual(gate.calls, list(root_gate.ROOT_SUITES))
            self.assertEqual(result['workspace_coverage'], {'root': 'full'})

    def test_unsupported_workspace_default_refuses_before_constructing_a_gate(self):
        import query_gate
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve(); directory = root/'unregistered'; directory.mkdir()
            (directory/'Cargo.toml').write_text('[workspace]\nmembers=[]\n', encoding='utf-8')
            with patch.object(query_gate, 'ROOT', root), patch.object(query_gate.Path, 'cwd', return_value=directory), \
                 patch.object(query_gate, 'Gate') as gate, contextlib.redirect_stderr(io.StringIO()):
                with self.assertRaises(SystemExit) as denied:
                    query_gate.main(['--out', str(root/'out')])
            self.assertEqual(denied.exception.code, 2)
            gate.assert_not_called()
            self.assertFalse((root/'out').exists())


if __name__ == '__main__':
    unittest.main()
