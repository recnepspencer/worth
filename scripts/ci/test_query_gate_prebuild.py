import contextlib
import io
from pathlib import Path
from types import SimpleNamespace
import tempfile
import unittest
from unittest.mock import patch

from query_gate_prebuild import GatePrebuild, PrerequisiteFailure, refused_scale_suites
import query_gate_courts
import query_gate
import root_gate


class WorkspacePrerequisiteHandoffTests(unittest.TestCase):
    def test_bank_prebuild_runs_once_and_its_paths_reach_the_child_environment(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            environment, calls = {}, []
            runner = object()

            def prepare(repository, workspace, output, selected_runner):
                calls.append((repository, workspace, output, selected_runner))
                return {'WORTH_PREBUILT_BANK_METADATA': str(output / 'metadata.json')}

            prerequisites = GatePrebuild(root, root / 'prebuilt', environment, runner, prepare)
            binaries = [{'package': 'worth-query-bank-court-estate-certification'}]
            prerequisites.prepare('bank', binaries)
            prerequisites.prepare('bank', binaries)
            self.assertEqual(calls, [(root, 'bank', root / 'prebuilt/bank', runner)])
            self.assertEqual(environment['WORTH_PREBUILT_BANK_METADATA'],
                             str(root / 'prebuilt/bank/metadata.json'))

    def test_root_compiler_prebuild_does_not_build_unselected_proof_specimens(self):
        calls, proof_calls = [], []
        prerequisites = GatePrebuild(Path('.'), Path('out'), {}, object(),
                                     lambda *args, **kwargs: calls.append((args, kwargs)) or {},
                                     lambda *args: proof_calls.append(args) or Path('proof'))
        prerequisites.prepare('root', [{'package': 'worth-court-server-certification-suite'}])
        prerequisites.prepare('root', [])
        self.assertEqual(len(calls), 1)
        self.assertEqual(calls[0][1], {'include_proof': False})
        self.assertEqual(proof_calls, [])
        prerequisites.prepare('root', [{'package': 'worth-court-proof'}])
        self.assertEqual(len(calls), 1)
        self.assertEqual(len(proof_calls), 1)
        self.assertEqual(prerequisites.environment['WORTH_PREBUILT_PROOF'], 'proof')
        prerequisites.prepare('root', [{'package': 'worth-court-proof'}])
        self.assertEqual(len(proof_calls), 1)

    def test_first_selected_proof_court_uses_the_shared_root_prebuild_once(self):
        calls = []
        prerequisites = GatePrebuild(Path('.'), Path('out'), {}, object(),
                                     lambda *args, **kwargs: calls.append(kwargs) or {})
        prerequisites.prepare('root', [{'package': 'worth-court-proof'}])
        prerequisites.prepare('root', [{'package': 'worth-court-proof'}])
        self.assertEqual(calls, [{'include_proof': True}])

    def test_failed_prebuild_publishes_no_paths_or_prepared_state(self):
        environment = {'existing': 'value'}
        def fail(*args):
            raise ValueError('malformed Cargo observation')
        prerequisites = GatePrebuild(Path('.'), Path('out'), environment, object(), fail)
        with self.assertRaises(PrerequisiteFailure) as failure:
            prerequisites.prepare('bank', [{'package': 'court'}])
        self.assertEqual(failure.exception.workspace, 'bank')
        self.assertEqual(prerequisites.prepared, set())
        self.assertEqual(environment, {'existing': 'value'})

    def test_census_refuses_scale_without_removing_scheduled_entrypoints(self):
        suites = ['root-rest', 'root-scale', 'query-scale', 'bank-scale']
        self.assertEqual(refused_scale_suites(suites, True), suites[1:])
        self.assertEqual(refused_scale_suites(suites, False), [])


class RootOrdinaryCoverageTests(unittest.TestCase):
    def test_ordinary_root_suites_select_each_product_frontier_exactly_once(self):
        ordinary = ['relational', 'signal', 'execution', 'root-rest']
        packages = ['worth-foundational', 'worth-proof', 'worth-server', 'worth-runtime-world',
                    'worth-runtime-bridge', 'worth-court-foundational-expressions',
                    'worth-court-proof', 'worth-court-server-certification-suite',
                    'worth-court-runtime-world', 'worth-court-runtime-bridge',
                    'worth-court-relational-certification', 'worth-court-signal-owner-services',
                    'worth-court-execution', 'worth-value-shape-derive']
        for package in packages:
            binary = {'package': package}
            selected = [suite for suite in ordinary
                        if query_gate_courts.TEST_SUITES[suite][1](binary)
                        and query_gate_courts.selected(suite, binary, 'domain::behaviour')]
            self.assertEqual(len(selected), 1, (package, selected))

    def test_root_rest_does_not_repeat_the_category_partitions(self):
        binary = {'package': 'worth-court-proof'}
        for category in query_gate_courts.CATEGORIES:
            self.assertFalse(query_gate_courts.selected('root-rest', binary,
                                                       category + '::domain::behaviour'))


class GateProcessHandoffTests(unittest.TestCase):
    def test_governed_tests_use_the_shared_cost_owner_without_nonlanding_chunks(self):
        with tempfile.TemporaryDirectory() as temporary:
            gate = query_gate.Gate(Path(temporary))
            with patch.object(gate.cost, 'run', return_value={'ok': True}) as cost, \
                    patch.object(gate, '_nonlanding_tests') as nonlanding:
                result = gate.tests('root-rest')
            self.assertTrue(result['ok'])
            cost.assert_called_once_with('root-rest')
            nonlanding.assert_not_called()

    def test_shared_prerequisite_failure_never_falls_back_to_the_nonlanding_path(self):
        with tempfile.TemporaryDirectory() as temporary:
            gate = query_gate.Gate(Path(temporary))
            failure = {'ok': False, 'failure': 'missing compiler output'}
            with patch.object(gate.cost, 'run', return_value=failure), \
                    patch.object(gate, '_nonlanding_tests') as nonlanding:
                result = gate.tests('root-rest')
            self.assertEqual(result, failure)
            nonlanding.assert_not_called()

    def test_census_refusal_happens_before_build_or_test_execution(self):
        with tempfile.TemporaryDirectory() as temporary:
            gate = query_gate.Gate(Path(temporary), census=True)
            with patch.object(gate.cost, 'run') as run:
                with self.assertRaises(ValueError):
                    gate.tests('query-scale')
            run.assert_not_called()


class RootGateEntrypointTests(unittest.TestCase):
    def test_root_defaults_cover_all_nonscheduled_root_partitions(self):
        with patch.object(root_gate, 'gate_main', return_value=0) as main:
            self.assertEqual(root_gate.main(['--census']), 0)
        self.assertEqual(main.call_args.kwargs['default_suites'], root_gate.ROOT_SUITES)
        self.assertNotIn('root-scale', root_gate.ROOT_SUITES)
        self.assertEqual(main.call_args.kwargs['allowed_suites'],
                         (*root_gate.ROOT_SUITES, 'root-scale'))

    def test_root_gate_refuses_foreign_suites_before_constructing_a_gate(self):
        with patch.object(query_gate, 'Gate') as gate, contextlib.redirect_stderr(io.StringIO()):
            with self.assertRaises(SystemExit) as refusal:
                root_gate.main(['query-rest'])
        self.assertEqual(refusal.exception.code, 2)
        gate.assert_not_called()

    def test_census_mode_refuses_every_workspace_scale_suite_before_execution(self):
        for suite in ('query-scale', 'bank-scale', 'root-scale'):
            with patch.object(query_gate, 'Gate') as gate, contextlib.redirect_stderr(io.StringIO()):
                with self.assertRaises(SystemExit) as refusal:
                    query_gate.main([suite, '--census'])
            self.assertEqual(refusal.exception.code, 2)
            gate.assert_not_called()


if __name__ == '__main__':
    unittest.main()
