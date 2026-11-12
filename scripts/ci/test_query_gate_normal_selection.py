"""Affected metadata, native category and scoped documentation boundaries."""
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from query_gate_cost import GateSelectionFailure, SuiteQueries, CostGate
from query_gate_normal import suites_for
from test_query_gate_cost import binary


class AffectedSelectionTests(unittest.TestCase):
    def scope(self):
        return dict(packages=['worth-execution', 'worth-court-execution'],
                    required_courts=['worth-court-execution'], doc_packages=['worth-execution'])

    def discovered(self):
        return {'rust-suites': {
            'owner': binary('worth-execution', 'lib', [('ordinary', False)]),
            'court': binary('worth-court-execution', 'test', [
                ('contract::ordinary', False), ('process::isolation', False),
                ('cost::ordinary', False), ('cost::scheduled', True)])}}

    def test_full_category_labels_do_not_promote_affected_metadata_to_workspace(self):
        scope = self.scope()
        names = scope['packages'] + ['worth-court-proof']
        metadata = dict(workspace_members=names, packages=[dict(id=name, name=name,
            targets=[dict(name=name.replace('-', '_'), kind=['test' if '-court-' in name else 'lib'])])
            for name in names])
        with tempfile.TemporaryDirectory() as directory:
            query = SuiteQueries(suites_for(scope['packages']), directory, scope)
            selection = query.build_selection(metadata)
            self.assertEqual(selection, dict(packages=sorted(scope['packages']), targets={}))
            jobs = query(self.discovered())
            self.assertFalse(query.workspace_inventory)
            self.assertEqual(query.coverage['status'], 'partial')
            self.assertEqual(query.expected['execution'], {('owner', 'ordinary'), ('court', 'contract::ordinary')})
            self.assertEqual(query.expected['root-process'], {('court', 'process::isolation')})
            self.assertEqual(query.expected['root-cost'], {('court', 'cost::ordinary')})
            self.assertEqual(query.ignored['root-cost'], {('court', 'cost::scheduled')})
            self.assertFalse(any('include-ignored' in str(job) for job in jobs))

    def test_missing_ignored_or_category_only_required_court_refuses(self):
        for cases in (None, [('contract::ordinary', True)], [('process::isolation', False)]):
            with self.subTest(cases=cases), tempfile.TemporaryDirectory() as directory:
                discovered = self.discovered()
                if cases is None:
                    del discovered['rust-suites']['court']
                else:
                    discovered['rust-suites']['court'] = binary('worth-court-execution', 'test', cases)
                with self.assertRaisesRegex(GateSelectionFailure, 'required forward court'):
                    SuiteQueries(suites_for(self.scope()['packages']), directory, self.scope())(discovered)
                self.assertFalse((Path(directory)/'gate-selection.json').exists())

    def test_foreign_discovered_owner_and_missing_metadata_court_refuse(self):
        with tempfile.TemporaryDirectory() as directory:
            scope = self.scope()
            discovered = self.discovered()
            discovered['rust-suites']['foreign'] = binary('worth-court-proof', 'test', [('other', False)])
            query = SuiteQueries(suites_for(scope['packages']), directory, scope)
            with self.assertRaisesRegex(GateSelectionFailure, 'outside affected'):
                query(discovered)
            with self.assertRaisesRegex(GateSelectionFailure, 'required forward court'):
                query.build_selection(dict(workspace_members=[], packages=[]))

    def test_wrong_target_and_default_filtered_court_cannot_satisfy_coverage(self):
        scope = self.scope()
        with tempfile.TemporaryDirectory() as directory:
            metadata = dict(workspace_members=scope['packages'], packages=[dict(id=name, name=name,
                targets=[dict(name=name.replace('-', '_'), kind=['test' if '-court-' in name else 'lib'])])
                for name in scope['packages']])
            query = SuiteQueries(suites_for(scope['packages']), directory, scope)
            query.build_selection(metadata)
            discovered = self.discovered()
            discovered['rust-suites']['court']['binary-name'] = 'foreign_target'
            with self.assertRaisesRegex(GateSelectionFailure, 'selected Cargo targets'):
                query(discovered)
            discovered = self.discovered()
            discovered['rust-suites']['court']['testcases']['contract::ordinary']['filter-match'] = {'status':'mismatch'}
            with self.assertRaisesRegex(GateSelectionFailure, 'filtered'):
                SuiteQueries(suites_for(scope['packages']), directory, scope)(discovered)

    def test_scoped_docs_are_forwarded_into_same_native_coordinator(self):
        scope = self.scope()
        with tempfile.TemporaryDirectory() as directory:
            def execute(*args, **kwargs):
                self.assertEqual(kwargs['documentation_packages'], ['worth-execution'])
                self.assertTrue(kwargs['include_docs'])
                kwargs['queries'](self.discovered())
                return {}  # Missing requested docs must refuse rather than claim green.
            class Runner:
                logs = []
                def __init__(self, *args): pass
            gate = CostGate(directory, directory, {}, execute, Runner, affected=scope)
            gate.configure([*suites_for(scope['packages']), 'root-doc'])
            result = gate.run('execution')
            self.assertFalse(result['ok'])
            self.assertIn('documentation has no execution', result['failure'])
