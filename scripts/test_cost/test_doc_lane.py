import json
import os
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

from doc_lane import execute
from run_lane import LaneExecutionFailure
from lane_environment import LaneEnvironmentRefusal
from compiler_units import declarations
from test_compiler_units import event as compiler_event, graph, unit


class DocLaneTests(unittest.TestCase):
    def test_documentation_checks_original_inputs_even_when_a_gate_normalized_its_environment(self):
        runner = SimpleNamespace(environment={}, logs=[])
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary)/'unused'
            with patch.dict(os.environ, {'CARGO_INCREMENTAL': '1'}), \
                 patch('doc_lane.collect') as discovery, patch('doc_lane.execute_documentation') as run:
                with self.assertRaisesRegex(LaneEnvironmentRefusal, 'CARGO_INCREMENTAL'):
                    execute(Path.cwd(), 'root', output, runner)
            discovery.assert_not_called()
            run.assert_not_called()
            self.assertFalse(output.exists())

    def test_doc_only_gate_refuses_compiler_overrides_before_discovery_or_execution(self):
        runner = SimpleNamespace(environment={'RUSTUP_TOOLCHAIN': 'other-compiler'}, logs=[])
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary)/'unused'
            with patch('doc_lane.collect') as discovery, patch('doc_lane.execute_documentation') as run:
                with self.assertRaises(LaneEnvironmentRefusal):
                    execute(Path.cwd(), 'root', output, runner)
            discovery.assert_not_called()
            run.assert_not_called()
            self.assertFalse(output.exists())

    def workspace(self, root):
        (root / 'src').mkdir()
        (root / 'src/lib.rs').write_text('', encoding='utf-8')
        (root / 'Cargo.toml').write_text('[package]\nname="owner"\nversion="0.1.0"\n', encoding='utf-8')

    def engine(self, root, copies=1, fresh=True, event='ok'):
        def run(manifest, output, runner, profile):
            output.mkdir(parents=True)
            snippet = {'binary': 'owner::doc', 'test': 'src/lib.rs (line 1)', 'ignored': False}
            required = unit('owner', mode='build')
            required['target']['src_path'] = str(root/'src/lib.rs')
            artifact = dict(compiler_event(required, linked=True), reason='compiler-artifact', fresh=not fresh)
            log = output / 'retained.log'
            lines = ['Command: ["cargo", "test", "--doc"]', json.dumps({
                'type': 'cargo-invocation', 'mode': 'build', 'cargo_profile': profile,
                'unit_plan': {'units': declarations(graph(required)), 'compiler_inputs': 'flags'}})]
            lines += [json.dumps(artifact)] * copies
            lines += [json.dumps({'type': 'test', 'event': event, 'name': 'owner::doc$src/lib.rs (line 1)'})]
            log.write_text('\n'.join(lines), encoding='utf-8')
            runner.logs.append(log)
            return {'inventory': [snippet], 'selections': [snippet],
                    'status': 'pass' if event == 'ok' else 'fail', 'exit_code': 0 if event == 'ok' else 101}
        return run

    def test_docs_get_census_execution_proof_and_declared_selection_budget(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            self.workspace(root)
            result = execute(root, 'root', root/'output', SimpleNamespace(logs=[]), engine=self.engine(root))
            self.assertEqual(result['coverage']['status'], 'partial')
            self.assertEqual(result['documentation_coverage']['status'], 'pass')
            self.assertEqual(result['executed_coverage']['status'], 'pass')
            self.assertEqual(result['executed_coverage']['scope'], 'documentation-only')
            self.assertEqual(result['build_ratchet'], {'budget_lane': 'selection', 'violations': []})
            for destination in (root/'output', root/'target/test-cost/root'):
                self.assertEqual(json.loads((destination/'census.json').read_text()), result)
                self.assertTrue((destination/'census.md').is_file())

    def test_source_invocations_refuse_before_doc_engine_and_retain_the_census(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            self.workspace(root)
            source = root / 'src/lib.rs'
            source.write_text('#[cfg(test)] mod tests { #[test] fn forbidden() { '
                              'std::process::Command::new("cargo"); } }', encoding='utf-8')
            with patch('doc_lane.execute_documentation') as engine, self.assertRaisesRegex(ValueError, 'invoke compilers'):
                execute(root, 'root', root/'output', SimpleNamespace(logs=[]))
            engine.assert_not_called()
            self.assertTrue(json.loads((root/'output/source-ratchet.json').read_text()))
            self.assertTrue((root/'target/test-cost/root/census.json').is_file())

    def test_duplicate_and_unmeasured_builds_cannot_bypass_the_ratchet(self):
        for copies in (2, 0):
            with self.subTest(copies=copies), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary).resolve()
                self.workspace(root)
                with self.assertRaisesRegex(ValueError, 'build budget exceeded'):
                    execute(root, 'root', root/'output', SimpleNamespace(logs=[]), engine=self.engine(root, copies))
                violations = json.loads((root/'output/build-ratchet.json').read_text())
                self.assertEqual(violations[0]['kind'], 'duplicate-graph' if copies else 'unmeasured-build')
                self.assertTrue((root/'target/test-cost/root/census.json').is_file())

    def test_doc_lane_budget_includes_repeated_failed_compilations_without_artifacts(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            self.workspace(root)
            cached = self.engine(root, fresh=False)
            def failing_compiler(manifest, output, runner, profile):
                result = cached(manifest, output, runner, profile)
                path = runner.logs[-1]
                lines = path.read_text().splitlines()
                artifact = json.loads(lines[2])
                error = json.dumps({'reason': 'compiler-message', 'package_id': artifact['package_id'],
                                    'target': artifact['target'], 'message': {'level': 'error', 'message': 'denied'}})
                path.write_text('\n'.join(lines + [error, lines[1], error]))
                return result
            with self.assertRaisesRegex(LaneExecutionFailure, 'build budget exceeded') as raised:
                execute(root, 'root', root/'output', SimpleNamespace(logs=[]), engine=failing_compiler)
            violations = raised.exception.result['build_ratchet']['violations']
            self.assertEqual(violations[0]['kind'], 'duplicate-graph')
            self.assertEqual(violations[0]['compiles'], 2)

    def test_doc_only_budget_failure_carries_counts_without_claiming_native_coverage(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            self.workspace(root)
            with self.assertRaises(LaneExecutionFailure) as raised:
                execute(root, 'root', root/'output', SimpleNamespace(logs=[]), engine=self.engine(root, copies=2))
            result = raised.exception.result
            self.assertEqual(len(result['documentation']['selections']), 1)
            self.assertEqual(result['executed_coverage']['passed'], 1)
            self.assertEqual(result['coverage']['scope'], 'documentation-only')
            self.assertEqual(result['coverage']['status'], 'partial')
            self.assertFalse(result['executed_coverage']['require_full_workspace'])
            self.assertIn('build budget exceeded', str(raised.exception))

    def test_warm_compiler_json_still_proves_zero_new_compilations(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            self.workspace(root)
            result = execute(root, 'root', root/'output', SimpleNamespace(logs=[]), engine=self.engine(root, fresh=False))
            self.assertFalse(result['build_ratchet']['violations'])

    def test_failed_observed_docs_remain_failed_with_census_evidence(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            self.workspace(root)
            with self.assertRaisesRegex(ValueError, 'documentation'):
                execute(root, 'root', root/'output', SimpleNamespace(logs=[]), engine=self.engine(root, event='failed'))
            result = json.loads((root/'output/census.json').read_text())
            self.assertEqual(result['documentation']['status'], 'fail')
            self.assertEqual(result['executed_coverage']['failed'], [['owner::doc', 'src/lib.rs (line 1)']])

    def test_invalid_execution_identity_refuses_with_retained_census_and_build_ratchet(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            self.workspace(root)
            valid = self.engine(root)
            def invalid(manifest, output, runner, profile):
                result = valid(manifest, output, runner, profile)
                path = runner.logs[-1]
                path.write_text(path.read_text().replace('owner::doc$src/lib.rs (line 1)', 'unowned'), encoding='utf-8')
                return result
            with self.assertRaisesRegex(ValueError, 'no binary identity'):
                execute(root, 'root', root/'output', SimpleNamespace(logs=[]), engine=invalid)
            result = json.loads((root/'output/census.json').read_text())
            self.assertEqual(result['executed_coverage']['status'], 'fail')
            self.assertIn('no binary identity', result['failure'])
            self.assertFalse(result['build_ratchet']['violations'])


if __name__ == '__main__':
    unittest.main()
