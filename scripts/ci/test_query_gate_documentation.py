"""Canonical doc ownership preserves consumer claims without repeating mains."""
import json
from pathlib import Path
import tempfile
import tomllib
import unittest
from unittest.mock import Mock, patch

from query_gate_cost import CostGate
from query_gate_courts import ROOT
from workspace import members


class DocumentationGateTests(unittest.TestCase):
    def test_doc_only_gate_does_not_build_native_harnesses_and_runs_once(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            snippet = {'binary': 'owner::doc', 'test': 'src/lib.rs (line 1)', 'ignored': False}
            class Runner:
                def __init__(self, repository, environment):
                    self.environment, self.logs = environment, []
            def docs(repository, workspace, output, runner):
                self.assertEqual(repository, root)
                self.assertEqual(workspace, 'query')
                output.mkdir(parents=True)
                log = output / 'doctests.log'
                log.write_text('Command: ["cargo", "test", "--workspace", "--doc"]\n' + json.dumps(
                    {'type': 'test', 'event': 'ok', 'name': 'owner::doc$src/lib.rs (line 1)'}), encoding='utf-8')
                runner.logs.append(log)
                return {'documentation': {'inventory': [snippet], 'selections': [snippet],
                                           'status': 'pass', 'exit_code': 0}}
            native = Mock(side_effect=AssertionError('native build must not run'))
            gate = CostGate(root, root / 'out', {}, native, Runner)
            gate.configure(['query-doc'])
            with patch('query_gate_cost.execute_documentation_lane', side_effect=docs) as execute:
                first, second = gate.run('query-doc'), gate.run('query-doc')
            execute.assert_called_once()
            native.assert_not_called()
            self.assertIs(first, second)
            self.assertTrue(first['ok'])
            self.assertEqual(first['passed'], 1)
            self.assertEqual(first['coverage']['status'], 'partial')

    def test_previous_consumer_doc_selections_belong_to_canonical_full_workspaces(self):
        configuration = tomllib.loads((ROOT / 'tools/boundary-check/config/consumer-docs.toml').read_text())
        for selection in configuration['doctests']:
            with self.subTest(manifest=selection['manifest']):
                packages = {package['name'] for package in members(ROOT / selection['manifest'])}
                self.assertTrue(set(selection['packages']) <= packages)

    def test_owning_examples_are_real_harness_free_claims_in_the_native_graph(self):
        manifest = ROOT / 'workspaces/worth-query/crates/worth-query-certification/Cargo.toml'
        configuration = tomllib.loads(manifest.read_text())
        examples = {row['name']: row for row in configuration['example']}
        for name in ('authored_workflow', 'ordinary_product_workflow'):
            with self.subTest(example=name):
                self.assertTrue(examples[name]['test'])
                self.assertFalse(examples[name]['harness'])
                self.assertTrue((manifest.parent / examples[name]['path']).is_file())


if __name__ == '__main__':
    unittest.main()
