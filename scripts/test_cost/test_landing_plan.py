"""Landing selection contracts use fixture maps and syntax, never metadata."""
import ast
import copy
from pathlib import Path
import tempfile
import unittest

from landing_plan import (BANK, QUERY, QUERY_LANDING, ROOT, STORE, STORE_LANDING,
                          captured_gate_command, configuration, consumer_document_lanes, phase34_violations, violations)


def workspaces():
    return {ROOT: ['root: cargo test --workspace'], BANK: ['bank: cargo test --workspace'],
            STORE: [STORE_LANDING, 'store: cargo clippy --workspace',
                    'store: scripts (phase34 dependency-direction, formal-toolchain, integrity)'],
            QUERY: [QUERY_LANDING, 'query: python scripts/ci/query_gate.py lint-query']}


class LandingPlanTests(unittest.TestCase):
    def test_serial_partitions_smoke_and_duplicate_queries_cannot_return(self):
        expected = workspaces()
        self.assertEqual(violations(expected), [])
        for manifest, addition in [(STORE, 'store: cargo test --workspace --no-fail-fast'),
                                   (STORE, 'store: store-test-runner smoke'),
                                   (STORE, 'store: store-test-runner partitions owner-unit, scenario'),
                                   (QUERY, 'query: python scripts/ci/query_gate.py query-execution,query-rest'),
                                   (QUERY, 'query: python scripts/ci/query_gate.py doc'),
                                   (QUERY, 'query: observable-workflow example'),
                                   (QUERY, QUERY_LANDING)]:
            altered = copy.deepcopy(expected)
            altered[manifest].append(addition)
            with self.subTest(addition=addition):
                self.assertTrue(violations(altered))

    def test_root_bank_and_warning_descriptors_stay_outside_this_cutover(self):
        before = workspaces()
        for manifest in (ROOT, BANK):
            changed = copy.deepcopy(before)
            changed[manifest] = ['new plan']
            self.assertTrue(any('unapproved workspace' in row for row in violations(changed, before)))
        changed = copy.deepcopy(before)
        changed[QUERY].remove('query: python scripts/ci/query_gate.py lint-query')
        self.assertTrue(violations(changed, before))

    def test_literal_inspection_does_not_execute_external_imports_main_or_metadata(self):
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary)/'external.py'
            path.write_text('FIXTURE="fixture/Cargo.toml"\nWORKSPACES={FIXTURE:["fixture lane"]}\n'
                            'ALWAYS=["structural"]\nraise RuntimeError("metadata must never run")\n')
            values, _, _ = configuration(path)
            self.assertEqual(values['WORKSPACES'], {'fixture/Cargo.toml': ['fixture lane']})
            path.write_text('WORKSPACES=discover_with_cargo_metadata()\n')
            with self.assertRaisesRegex(ValueError, 'literal data'):
                configuration(path)

    def test_consumer_doc_routes_skip_every_already_selected_workspace(self):
        source = '''def consumer_document_lanes(reached_workspaces):
    documentation=(("Cargo.toml","root"),("workspaces/worth-query/Cargo.toml","query"),("workspaces/worth-query-bank-world/Cargo.toml","bank"))
    return [f"docs: python scripts/ci/query_gate.py {workspace}-doc" for manifest,workspace in documentation if manifest not in reached_workspaces]
'''
        tree = ast.parse(source)
        self.assertEqual(consumer_document_lanes(tree, [ROOT, QUERY, BANK]), [])
        self.assertEqual(consumer_document_lanes(tree, [QUERY]), [
            'docs: python scripts/ci/query_gate.py root-doc', 'docs: python scripts/ci/query_gate.py bank-doc'])
        self.assertEqual(len(consumer_document_lanes(tree, [])), 3)
        unsafe = ast.parse('def consumer_document_lanes(reached):\n return run_metadata()\n')
        with self.assertRaisesRegex(ValueError, 'pure literal selection'):
            consumer_document_lanes(unsafe, [])

    def test_explicit_running_gate_command_proves_alias_parity_without_claiming_completion(self):
        import json
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary)/'gate.log'
            command = ['python', 'scripts/ci/query_gate.py', 'bank,bank-doc', '--census']
            path.write_text('Command: '+json.dumps(command)+'\nHEAD: captured-head\nStart: captured-start\n')
            observed = captured_gate_command(path, 'bank', ['bank', 'bank-doc'])
            self.assertIn('not runtime or completion', observed['scope'])
            with self.assertRaisesRegex(ValueError, 'differs'):
                captured_gate_command(path, 'bank', ['bank'])

    def test_phase34_retains_only_dependency_direction_and_failure_propagation(self):
        source = ('& python (Join-Path $root "scripts/ci/check_worth_store_phase34_dependencies.py")\n'
                  'if ($LASTEXITCODE -ne 0) { throw "direction failed" }\n')
        self.assertEqual(phase34_violations(source), [])
        for extra in ['& cargo test --workspace', 'Invoke-Checked "old target"', 'param([switch]$Workspace)']:
            self.assertTrue(phase34_violations(source+'\n'+extra))
        self.assertTrue(phase34_violations(source.replace('throw ', 'Write-Host ')))

    def test_scheduled_c9_exclusion_is_explicit_and_no_normal_descriptor_runs_it(self):
        self.assertEqual(violations(workspaces(), source='# C9 phases scheduled-only and excluded from landing'), [])
        self.assertTrue(violations(workspaces(), source='# no rule'))
        changed = workspaces()
        changed[STORE].append('store: scripts C9')
        self.assertTrue(violations(changed))


if __name__ == '__main__':
    unittest.main()
