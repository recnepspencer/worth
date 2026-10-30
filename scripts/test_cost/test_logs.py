import json
from pathlib import Path
import tempfile
import unittest

from logs import duplicates, llvm_lines, read_lane
from telemetry import timing_summary
from compiler_units import declarations
from test_compiler_units import event as compiler_event, graph, unit


def compiled(**changes):
    value = dict(reason='compiler-artifact', package_id='path+file:///x#owner@1',
                 target={'name': 'owner', 'kind': ['lib']}, features=[],
                 profile={'opt_level': '0', 'test': False}, fresh=False,
                 filenames=['C:/x/target/debug/deps/owner-123.rlib'])
    value.update(changes)
    return value


class ArtifactObservationTests(unittest.TestCase):
    def header_lane(self, command, events, exit_code=101, prefix=''):
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary)/'header-only.log'
            path.write_text(prefix + 'Command: ' + json.dumps(command) + '\n\n' +
                            '\n'.join(map(json.dumps, events)) + f'\nExit code: {exit_code}\n')
            return read_lane(path)

    def test_markerless_compiler_failure_uses_only_the_leading_command_header(self):
        cached = compiled(fresh=True)
        for events in ([cached, {'reason': 'build-finished', 'success': False}], [cached]):
            with self.subTest(events=events):
                lane = self.header_lane(['cargo', 'check'], events)
                self.assertEqual(lane['artifacts'], [])
                self.assertEqual(lane['failed_compiler_attempts'], [])
                self.assertEqual(lane['compiler_attempt_refusals'][0]['kind'], 'unattributed-compiler-outcome')
        nested = self.header_lane(['cargo', 'check'], [cached], prefix='A body heading first\n')
        self.assertEqual(nested['compiler_attempt_refusals'][0]['kind'], 'undeclared-compiler-invocation')
        self.assertNotIn('invocation', nested['compiler_attempt_refusals'][0])

    def test_markerless_successful_build_retains_zero_observation_but_cannot_verify_missing_graph(self):
        lane = self.header_lane(['cargo', 'test'], [compiled(fresh=True),
                                {'reason': 'build-finished', 'success': True}], exit_code=101)
        self.assertEqual(lane['artifacts'], [])
        self.assertEqual(lane['failed_compiler_attempts'], [])
        self.assertEqual(lane['compiler_attempt_refusals'][0]['kind'], 'undeclared-compiler-invocation')

    def test_markerless_prebuilt_runtime_and_declaration_headers_do_not_infer_compilation(self):
        for command in (['cargo', 'nextest', 'run', '--binaries-metadata=built.json'],
                        ['cargo', 'check', '--unit-graph']):
            with self.subTest(command=command):
                lane = self.header_lane(command, [compiled(fresh=True), {'reason': 'build-finished', 'success': False}])
                self.assertEqual(lane['failed_compiler_attempts'], [])
                self.assertEqual(lane['compiler_attempt_refusals'], [])

    def test_unscoped_cached_artifact_observation_cannot_verify_a_unit_budget(self):
        lane = self.lane([compiled(fresh=True)])
        self.assertEqual(lane['artifacts'], [])
        self.assertEqual(lane['failed_compiler_attempts'], [])
        self.assertEqual(lane['compiler_attempt_refusals'][0]['kind'], 'undeclared-compiler-invocation')

    def test_unsuccessful_build_without_error_target_is_unknown_not_zero_attempts(self):
        marker, cached = self.declared(unit('owner'))
        cached['fresh'] = True
        for finished in (False, None):
            events = [marker, cached]
            if finished is not None:
                events.append({'reason': 'build-finished', 'success': finished})
            events.append({'type': 'cargo-result', 'exit_code': 101})
            with self.subTest(finished=finished):
                lane = self.lane(events)
                self.assertEqual(lane['failed_compiler_attempts'], [])
                self.assertEqual(lane['artifacts'], [])
                self.assertEqual(lane['compiler_attempt_refusals'][0]['kind'], 'unattributed-compiler-outcome')

    def test_build_success_then_test_runtime_failure_has_no_unknown_compile(self):
        marker, cached = self.declared(unit('owner'))
        marker['command'] = ['cargo', 'test']
        cached['fresh'] = True
        lane = self.lane([marker, cached, {'reason': 'build-finished', 'success': True},
                          self.error(unit('foreign-runtime-print')), {'reason': 'build-finished', 'success': False},
                          {'type': 'cargo-result', 'exit_code': 101}])
        self.assertEqual(lane['compiler_attempt_refusals'], [])
        self.assertEqual(lane['failed_compiler_attempts'], [])

    def test_outcome_refusal_belongs_to_its_invocation_not_the_last_exit(self):
        marker, cached = self.declared(unit('owner'))
        cached['fresh'] = True
        lane = self.lane([marker, cached, {'reason': 'build-finished', 'success': False},
                          {'type': 'cargo-result', 'exit_code': 101}, marker, cached,
                          {'reason': 'build-finished', 'success': True}, {'type': 'cargo-result', 'exit_code': 0}])
        self.assertEqual(len(lane['compiler_attempt_refusals']), 1)
        self.assertTrue(lane['compiler_attempt_refusals'][0]['invocation'].endswith('#0'))

    def test_noncompiling_declaration_failure_does_not_invent_a_compiler_attempt(self):
        lane = self.lane([{'type': 'cargo-invocation', 'command': ['cargo', 'check', '--unit-graph']},
                          {'type': 'cargo-result', 'exit_code': 101}])
        self.assertEqual(lane['failed_compiler_attempts'], [])
        self.assertEqual(lane['compiler_attempt_refusals'], [])

    def error(self, row, level='error'):
        return {'reason': 'compiler-message', 'package_id': row['pkg_id'], 'target': row['target'],
                'message': {'level': level, 'message': 'compiler denied this target'}}

    def test_multiple_error_diagnostics_prove_one_attempt_per_declared_invocation(self):
        owner = unit('owner')
        marker = self.declared(owner)[0]
        row = self.lane([marker, self.error(owner), self.error(owner)])
        self.assertEqual(row['artifacts'], [])
        self.assertEqual(row['artifact_messages'], 0)
        self.assertEqual(row['compiler_error_messages'], 2)
        self.assertEqual(len(row['failed_compiler_attempts']), 1)
        self.assertEqual(row['build_observation'], 'json')
        twice = self.lane([marker, self.error(owner), marker, self.error(owner)])
        self.assertEqual(len(twice['failed_compiler_attempts']), 2)
        self.assertEqual(twice['compiler_attempt_groups'][0]['repeated_units'][0]['compiles'], 2)

    def test_cached_warning_or_test_printing_prebuilt_diagnostic_does_not_invent_a_compile(self):
        owner = unit('owner')
        marker, cached = self.declared(owner)
        cached['fresh'] = True
        row = self.lane([marker, cached, self.error(owner, 'warning')])
        self.assertEqual(row['artifacts'], [])
        self.assertEqual(row['failed_compiler_attempts'], [])
        printed = self.lane([dict(type='cargo-invocation', command=['cargo', 'nextest', 'run',
                             '--binaries-metadata', 'built.json']), self.error(owner)])
        self.assertEqual(printed['failed_compiler_attempts'], [])
        self.assertEqual(printed['compiler_attempt_refusals'], [])

    def test_ambiguous_failed_target_refuses_instead_of_inventing_two_attempts(self):
        first = unit('owner', dependencies=[1, 2])
        dependency, linked = unit('dependency'), unit('dependency', mode='build')
        marker = dict(type='cargo-invocation', unit_plan={
            'units': declarations(graph(first, dependency, linked)), 'compiler_inputs': 'flags'})
        row = self.lane([marker, self.error(dependency), self.error(dependency)])
        self.assertEqual(row['failed_compiler_attempts'], [])
        self.assertEqual(len(row['compiler_attempt_refusals']), 1)

    def declared(self, row, dependencies=(), directory='target'):
        return [dict(type='cargo-invocation', cargo_profile='dev', mode='check', target_dir=directory,
                     unit_plan={'units': declarations(graph(row, *dependencies)), 'compiler_inputs': 'flags'}),
                dict(compiler_event(row, linked=row['mode'] != 'check'), reason='compiler-artifact', fresh=False)]

    def test_required_transitive_variants_are_attributed_without_calling_them_duplicate_units(self):
        owner = unit('owner', dependencies=[1])
        first = self.declared(owner, [unit('proof')])
        second = self.declared(owner, [unit('proof', features=['cert'])])
        second[1]['filenames'] = ['target/debug/deps/owner-different-hash.rmeta']
        row = self.lane(first + second)['duplicates'][0]
        self.assertEqual(row['causes'], ['required_dependency_graph'])
        self.assertEqual(row['compiles'], 2)
        self.assertEqual(row['distinct_required_units'], 2)
        self.assertEqual(row['repeated_units'], [])
        self.assertEqual(row['unit_observation'], 'declared')

    def test_repeated_identical_unit_is_distinct_from_required_package_variants(self):
        required = self.declared(unit('owner'))
        repeated = self.declared(unit('owner'), directory='another-target')
        row = self.lane(required + repeated)['duplicates'][0]
        self.assertEqual(row['distinct_required_units'], 1)
        self.assertEqual(row['repeated_units'][0]['compiles'], 2)
        self.assertIn('target_dir', row['causes'])
        self.assertIn('repeated identical required unit', row['causes'])

    def test_declared_output_profile_and_physical_platform_differences_remain_visible(self):
        for field, value, cause in [('mode', 'build', 'required_output'),
                                    ('platform', 'wasm32-unknown-unknown', 'target_platform'),
                                    ('profile', {**unit('owner')['profile'], 'incremental': False}, 'declared_profile')]:
            first, second = unit('owner'), unit('owner')
            second[field] = value
            with self.subTest(field=field):
                row = self.lane(self.declared(first) + self.declared(second))['duplicates'][0]
                self.assertIn(cause, row['causes'])
                self.assertEqual(row['distinct_required_units'], 2)
                self.assertNotIn('unattributed compiler inputs', row['causes'])

    def test_plain_libtest_summaries_retain_counts_without_inventing_test_identities(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'legacy.log'
            path.write_text('test result: ok. 3 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out\n'
                            'test result: FAILED. 5 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out\n')
            lane = read_lane(path)
            self.assertEqual(lane['tests'], [])
            self.assertEqual(lane['plain_test_counts'], {'pass': 8, 'fail': 1, 'ignored': 2})
            self.assertEqual(timing_summary(lane)['pass'], 8)

    def test_aggregate_phase_wall_is_distinct_from_the_aggregation_command_duration(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'phases.log'
            path.write_text('Start: 2026-10-29T00:00:00+00:00\n' + json.dumps(dict(
                type='test-cost-phases', build_seconds=50, wall_seconds=240)) +
                '\nEnd: 2026-10-29T00:00:01+00:00\n')
            lane = read_lane(path)
            self.assertEqual(lane['telemetry']['wall_seconds'], 240)
            self.assertEqual(lane['telemetry']['run_seconds'], 190)
            self.assertEqual(lane['telemetry']['capture_wall_seconds'], 1)

    def lane(self, rows):
        with tempfile.TemporaryDirectory() as directory:
            log = Path(directory) / 'lane.log'
            log.write_text('\n'.join(json.dumps(r) for r in rows), encoding='utf-8')
            return read_lane(log)

    def test_fresh_artifacts_do_not_count_as_compilation(self):
        result = self.lane([compiled(), compiled(fresh=True), compiled(features=['cert'])])
        self.assertEqual(len(result['artifacts']), 2)
        self.assertEqual(result['duplicates'][0]['causes'], ['features'])
        warm = self.lane([compiled(fresh=True)])
        self.assertEqual(warm['build_observation'], 'json')
        self.assertEqual(warm['artifacts'], [])

    def test_profile_target_directory_and_check_drift_are_distinguished(self):
        result = self.lane([compiled(),
                            dict(type='cargo-invocation', target_dir='alternate', mode='check'),
                            compiled(profile={'test': False, 'opt_level': '2'}, filenames=['alternate/debug/deps/owner.rmeta'])])
        self.assertEqual(result['duplicates'][0]['causes'],
                         ['profile', 'target_dir', 'check_mode'])

    def test_one_cargo_check_can_link_a_host_dependency_and_check_its_target_library(self):
        result = self.lane([dict(type='cargo-invocation', mode='check'), compiled(),
                            compiled(filenames=['C:/x/target/debug/deps/owner-456.rmeta'])])
        self.assertEqual([row['check_mode'] for row in result['artifacts']], ['build', 'check'])
        self.assertEqual([row['invocation_mode'] for row in result['artifacts']], ['check', 'check'])
        self.assertEqual(result['duplicates'][0]['causes'], ['check_mode'])

    def test_equal_own_features_do_not_prove_identical_dependency_or_compiler_inputs(self):
        result = self.lane([compiled(), compiled(filenames=['C:/x/target/debug/deps/owner-456.rlib'])])
        self.assertEqual(result['duplicates'][0]['causes'], ['unattributed compiler inputs'])
        repeated = self.lane([compiled(), compiled()])
        self.assertEqual(repeated['duplicates'][0]['causes'], ['repeated observed compiler unit'])

    def test_text_only_build_is_unknown_rather_than_zero_duplicates(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'lane.log'
            path.write_text('   Compiling owner v1\n')
            self.assertEqual(read_lane(path)['build_observation'], 'unmeasured')

    def test_formal_build_and_run_telemetry_are_distinct_from_summed_test_work(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'formal.log'
            path.write_text('Finished `ci-test` profile [unoptimized] target(s) in 26m 39s\n'
                            'Summary [   0.265s] 2 tests run: 2 passed\n')
            observation = read_lane(path)
            self.assertEqual(observation['telemetry']['build_seconds'], 1599)
            self.assertEqual(observation['telemetry']['run_seconds'], .265)
            self.assertEqual(observation['build_observation'], 'unmeasured')

    def test_cargo_json_inside_test_output_is_also_observed(self):
        result = self.lane([dict(type='test', event='failed', name='compiler',
                                 stdout=json.dumps(compiled()), exec_time=900)])
        self.assertEqual(len(result['artifacts']), 1)
        self.assertEqual(result['telemetry']['test_work_seconds'], 900)

    def test_nextest_ignored_inventory_counts_do_not_require_an_executed_test_event(self):
        result = self.lane([dict(type='suite', event='ok', passed=1, failed=0, ignored=1)])
        self.assertEqual(timing_summary(result)['ignored'], 1)
        self.assertEqual(timing_summary(result)['pass'], 0)

    def test_llvm_totals_and_generic_helper_families_are_retained(self):
        rows = llvm_lines('1000 (100%) 30 (100%) TOTAL\n500 (50%) 4 (10%) kit::world<T>',
                          ['kit::world'])
        self.assertEqual(rows[0]['lines'], 1000)
        self.assertEqual(rows[1]['testkit_helpers'], ['kit::world'])
        with self.assertRaises(ValueError):
            llvm_lines('error: no such command')

    def test_actual_llvm_file_mode_table_retains_parenthesized_total_and_percentages(self):
        rows = llvm_lines('  Lines            Copies           Function name\n'
                          '  -----            ------           -------------\n'
                          '  1                1                (TOTAL)\n'
                          '  1 (100.0%,100.0%) 1 (100.0%,100.0%) probe::admitted\n')
        self.assertEqual(rows, [dict(lines=1, copies=1, name='TOTAL', testkit_helpers=[]),
                                dict(lines=1, copies=1, name='probe::admitted', testkit_helpers=[])])


if __name__ == '__main__':
    unittest.main()
