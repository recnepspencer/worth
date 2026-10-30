import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch


from query_gate_cost import (CostGate, GateSelectionFailure, SuiteQueries, executed_observations,
                             DOC_SUITES, expand_suites, landing_suites)
from run_lane import LaneExecutionFailure


def binary(package, kind, names, harness=True):
    return {'package-name': package, 'binary-name': package.replace('-', '_'),
            'kind': kind, 'status': 'listed', 'harness': harness,
            'testcases': {name: {'ignored': ignored} for name, ignored in names}}


class SharedGateSelectionTests(unittest.TestCase):
    def test_real_package_ownership_and_categories_produce_disjoint_suite_queries(self):
        suites = ['relational', 'root-rest', 'root-process', 'root-compile-fail']
        discovered = {'rust-suites': {
            'relational-court': binary('worth-court-relational-certification', 'test',
                                       [('relational::ordinary', False), ('process::crash', False)]),
            'proof-court': binary('worth-court-proof', 'test',
                                 [('proof::ordinary', False), ('compile_fail::sealed', False)]),
            'world-main': binary('worth-runtime-world', 'example', [('<main>', False)], False),
        }}
        with tempfile.TemporaryDirectory() as temporary:
            plan = SuiteQueries(suites, Path(temporary))
            jobs = plan(discovered)
        self.assertEqual(plan.coverage['status'], 'pass')
        self.assertEqual(plan.expected['relational'], {('relational-court', 'relational::ordinary')})
        self.assertEqual(plan.expected['root-process'], {('relational-court', 'process::crash')})
        self.assertIn('world-main', jobs[1][4])
        self.assertIn('test(/^compile_fail::/)', jobs[3][1])

    def test_partial_requests_are_labeled_partial_and_empty_categories_create_no_jobs(self):
        discovered = {'rust-suites': {'court': binary('worth-query-court-host', 'test',
                                                      [('host::ordinary', False), ('cost::scale', True)])}}
        with tempfile.TemporaryDirectory() as temporary:
            plan = SuiteQueries(['query-process'], Path(temporary))
            self.assertEqual(plan(discovered), [])
            self.assertEqual(plan.coverage['status'], 'partial')
            self.assertEqual(plan.coverage['unselected'], ['court$host::ordinary'])

    def test_standalone_only_selection_has_no_native_query_and_keeps_main_identity(self):
        discovered = {'rust-suites': {'main': binary('worth-query-certification', 'example',
                                                     [('<main>', False)], False)}}
        with tempfile.TemporaryDirectory() as temporary:
            jobs = SuiteQueries(['query-rest'], Path(temporary))(discovered)
        self.assertEqual(jobs, [('query-rest', None, None, 'ci', ['main'])])

    def test_repeated_suite_membership_is_refused_before_test_execution(self):
        discovered = {'rust-suites': {'court': binary('worth-query-court-host', 'test', [('host::ordinary', False)])}}
        with tempfile.TemporaryDirectory() as temporary:
            with self.assertRaises(GateSelectionFailure):
                SuiteQueries(['query-rest', 'query-rest'], Path(temporary))(discovered)

    def test_all_selected_suites_share_one_workspace_execution_and_actual_result_names(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            calls = []
            discovered = {'rust-suites': {'court': binary('worth-query-court-host', 'test',
                [('host::ordinary', False), ('process::crash', False)])}}

            class Runner:
                def __init__(self, repository, environment):
                    self.environment, self.logs = environment, []

            def execute(repository, workspace, lanes, output, cargo_runner, queries, include_docs=False):
                self.assertFalse(include_docs)
                calls.append(workspace)
                jobs = queries(discovered)
                for suite, _, _, _, _ in jobs:
                    log = output / f'run-{suite}.log'
                    events = [dict(type='test', event='ok', name=(
                        f'{discovered["rust-suites"][binary]["package-name"]}::'
                        f'{discovered["rust-suites"][binary]["binary-name"]}${name}'), exec_time=0.1)
                        for binary, name in queries.expected[suite]]
                    log.write_text('Command: ["cargo", "nextest", "run"]\n' +
                                   '\n'.join(json.dumps(event) for event in events), encoding='utf-8')
                    cargo_runner.logs.append(log)

            gate = CostGate(root, root/'out', {}, execute, Runner)
            gate.configure(['query-rest', 'query-process'])
            ordinary, process = gate.run('query-rest'), gate.run('query-process')
            self.assertEqual(calls, ['query'])
            self.assertEqual(ordinary['names'], ['court$host::ordinary'])
            self.assertEqual(process['names'], ['court$process::crash'])
            self.assertTrue(ordinary['ok'] and process['ok'])

    def test_documentation_aliases_select_each_canonical_workspace_without_example_commands(self):
        self.assertEqual(expand_suites(['doc']), ['root-doc', 'query-doc', 'bank-doc'])
        self.assertEqual(set(DOC_SUITES.values()), {'root', 'query', 'bank'})

    def test_workspace_landing_aliases_derive_every_normal_category_and_own_docs(self):
        for workspace in ('query', 'bank'):
            expanded = expand_suites([workspace+'-landing'])
            self.assertEqual(expanded, [*landing_suites(workspace), workspace+'-doc'])
            self.assertEqual(len(expanded), len(set(expanded)))
            self.assertFalse(any(name.endswith('-scale') for name in expanded))
            self.assertIn(workspace+'-cost', expanded)
            self.assertNotIn('doc', expanded)

    def test_docs_share_the_native_execution_and_preserve_actual_ignored_observations(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            calls = []
            discovered = {'rust-suites': {'court': binary('worth-query-court-host', 'test',
                                                        [('host::ordinary', False)])}}
            snippet = {'binary': 'worth-query-host::doc', 'test': 'src/lib.rs (line 1)', 'ignored': False}
            ignored = dict(snippet, test='src/lib.rs (line 9)', ignored=True)
            class Runner:
                def __init__(self, repository, environment):
                    self.environment, self.logs = environment, []
            def execute(repository, workspace, lanes, output, cargo_runner, queries, include_docs=False):
                calls.append((workspace, include_docs))
                queries(discovered)
                log = output / 'native.log'
                log.write_text('Command: ["cargo", "nextest", "run"]\n' + json.dumps(dict(
                    type='test', event='ok', name='worth-query-court-host::worth_query_court_host$host::ordinary')),
                    encoding='utf-8')
                doc = output / 'documentation.log'
                doc.write_text('Command: ["cargo", "test", "--doc"]\n' + '\n'.join(json.dumps(dict(
                    type='test', event=event, name=row['binary'] + '$' + row['test']))
                    for row, event in ((snippet, 'ok'), (ignored, 'ignored'))), encoding='utf-8')
                cargo_runner.logs += [log, doc]
                return {'documentation': {'inventory': [snippet, ignored], 'selections': [snippet],
                                          'status': 'pass', 'exit_code': 0}}
            gate = CostGate(root, root/'out', {}, execute, Runner)
            gate.configure(['query-rest', 'query-doc'])
            native, docs = gate.run('query-rest'), gate.run('query-doc')
            self.assertEqual(calls, [('query', True)])
            self.assertTrue(native['ok'] and docs['ok'])
            self.assertEqual(docs['passed'], 1)
            self.assertEqual(docs['ignored'], 1)
            self.assertEqual(docs['names'], ['worth-query-host::doc$src/lib.rs (line 1)'])

    def test_completed_budget_failure_retains_native_and_documentation_observations(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            discovered = {'rust-suites': {'court': binary('worth-query-court-host', 'test',
                                                        [('host::ordinary', False)])}}
            snippet = {'binary': 'worth-query-host::doc', 'test': 'src/lib.rs (line 1)', 'ignored': False}
            class Runner:
                def __init__(self, repository, environment):
                    self.environment, self.logs = environment, []
            def execute(repository, workspace, lanes, output, cargo_runner, queries, include_docs=False):
                self.assertTrue(include_docs)
                queries(discovered)
                native = output/'native.log'
                native.write_text('Command: ["cargo", "nextest", "run"]\n'+json.dumps(dict(
                    type='test', event='ok', name='worth-query-court-host::worth_query_court_host$host::ordinary')))
                docs = output/'documentation.log'
                docs.write_text('Command: ["cargo", "test", "--doc"]\n'+json.dumps(dict(
                    type='test', event='ok', name=snippet['binary']+'$'+snippet['test'])))
                cargo_runner.logs += [native, docs]
                raise LaneExecutionFailure('build budget exceeded: duplicate graph', {
                    'documentation': {'inventory': [snippet], 'selections': [snippet], 'status': 'pass', 'exit_code': 0},
                    'executed_coverage': {'executed': 2, 'passed': 2}})
            gate = CostGate(root, root/'out', {}, execute, Runner)
            gate.configure(['query-rest', 'query-doc'])
            native, docs = gate.run('query-rest'), gate.run('query-doc')
            self.assertFalse(native['ok'] or docs['ok'])
            self.assertEqual((native['passed'], docs['passed']), (1, 1))
            self.assertEqual(docs['tests'], 1)
            self.assertEqual(docs['missing'], [])
            self.assertEqual(docs['coverage']['documentation'], 'observed')
            self.assertEqual(docs['failure'], 'build budget exceeded: duplicate graph')
            self.assertNotIn('unexpected executions', docs['failure'])

    def test_prebuild_failure_without_result_leaves_documentation_unmeasured(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            discovered = {'rust-suites': {'court': binary('worth-query-court-host', 'test',
                                                        [('host::ordinary', False)])}}
            class Runner:
                def __init__(self, repository, environment):
                    self.environment, self.logs = environment, []
            def execute(repository, workspace, lanes, output, cargo_runner, queries, include_docs=False):
                queries(discovered)
                raise ValueError('prebuild rejected before test execution')
            gate = CostGate(root, root/'out', {}, execute, Runner)
            gate.configure(['query-rest', 'query-doc'])
            native, docs = gate.run('query-rest'), gate.run('query-doc')
            self.assertFalse(native['ok'] or docs['ok'])
            self.assertEqual(native['missing'], ['court$host::ordinary'])
            self.assertEqual(docs['passed'], 0)
            self.assertEqual(docs['coverage']['documentation'], 'unmeasured')
            self.assertEqual(docs['failure'], 'prebuild rejected before test execution')

    def test_doc_only_typed_failure_keeps_doc_counts_and_native_scope_unknown(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            snippet = {'binary': 'worth-query-host::doc', 'test': 'src/lib.rs (line 1)', 'ignored': False}
            class Runner:
                def __init__(self, repository, environment):
                    self.environment, self.logs = environment, []
            def docs_engine(repository, workspace, output, runner):
                output.mkdir(parents=True, exist_ok=True)
                log = output/'documentation.log'
                log.write_text('Command: ["cargo", "test", "--doc"]\n'+json.dumps(dict(
                    type='test', event='ok', name=snippet['binary']+'$'+snippet['test'])))
                runner.logs.append(log)
                raise LaneExecutionFailure('build budget exceeded: doc graph', {
                    'documentation': {'inventory': [snippet], 'selections': [snippet], 'status': 'pass', 'exit_code': 0}})
            gate = CostGate(root, root/'out', {}, runner_factory=Runner)
            gate.configure(['query-doc'])
            with patch('query_gate_cost.execute_documentation_lane', side_effect=docs_engine):
                docs = gate.run('query-doc')
            self.assertFalse(docs['ok'])
            self.assertEqual(docs['passed'], 1)
            self.assertEqual(docs['coverage']['status'], 'partial')
            self.assertEqual(docs['coverage']['reason'], 'native inventory not requested')
            self.assertEqual(docs['failure'], 'build budget exceeded: doc graph')

    def test_library_display_name_uses_the_actual_metadata_binary_identity(self):
        # Actual cargo-nextest 0.9.140 transport probe output: inventory and CPU
        # identify the library by package, while libtest JSON names its target.
        package = 'test-cost-transport-probe'
        alias = package + '::test_cost_transport_probe'
        with tempfile.TemporaryDirectory() as temporary:
            log = Path(temporary) / 'run-probe.log'
            event = dict(type='test', event='ok',
                         name=alias + '$compiler_transport_is_live', exec_time=0.1)
            log.write_text('Command: ["cargo", "nextest", "run"]\n' + json.dumps(event), encoding='utf-8')
            events, _ = executed_observations([log], {alias: package})
        self.assertEqual(events[0]['binary'], package)
        self.assertEqual(events[0]['name'], package + '$compiler_transport_is_live')
        self.assertEqual(events[0]['reported_name'], event['name'])

    def test_ambiguous_display_aliases_refuse_before_any_execution_plan_is_returned(self):
        shared = binary('owner', 'lib', [('ordinary', False)])
        other = dict(shared, kind='bin')
        with tempfile.TemporaryDirectory() as temporary:
            with self.assertRaises(GateSelectionFailure):
                SuiteQueries(['root-rest'], Path(temporary))({'rust-suites': {'lib-id': shared, 'bin-id': other}})

    def test_missing_unrequested_or_repeated_executions_cannot_report_green(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            plan = SuiteQueries(['query-rest'], root)
            plan({'rust-suites': {'court': binary('worth-query-court-host', 'test',
                                                   [('host::ordinary', False)])}})
            gate = CostGate(root, root/'out', {})
            gate.configure(['query-rest'])
            actual = dict(binary='court', test='host::ordinary', name='court$host::ordinary',
                          event='ok', log='run.log', exec_time=0.1)
            foreign = dict(actual, test='host::unrequested', name='court$host::unrequested')
            for observed in [[], [actual, foreign], [actual, actual]]:
                gate.record('query', plan, observed, {'run.log': 1.0}, None)
                self.assertFalse(gate.results['query-rest']['ok'])

    def test_empty_category_suites_are_valid_but_empty_ordinary_suites_are_not(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            plan = SuiteQueries(['query-rest', 'query-process'], root)
            self.assertEqual(plan({'rust-suites': {}}), [])
            gate = CostGate(root, root/'out', {})
            gate.configure(['query-rest', 'query-process'])
            gate.record('query', plan, [], {}, None)
            self.assertFalse(gate.results['query-rest']['ok'])
            self.assertTrue(gate.results['query-rest']['no_tests'])
            self.assertTrue(gate.results['query-process']['ok'])


if __name__ == '__main__':
    unittest.main()
