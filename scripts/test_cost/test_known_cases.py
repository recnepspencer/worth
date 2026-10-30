"""Known-case reports require actual source and execution observations."""
import copy
import json
from pathlib import Path
import tempfile
import unittest

from known_cases import diagnostic_hits, markdown, pool_match, validate
from logs import read_lane


def lane(events=(), telemetry=None, log='lane.log'):
    return {'log': log, 'tests': list(events), 'telemetry': telemetry or {}}


def event(name, seconds=1, outcome='ok'):
    result = {'type': 'test', 'event': outcome, 'name': name, 'exec_time': seconds}
    if outcome == 'failed':
        result['reason'] = 'time limit exceeded'
    return result


def get(result, identity):
    return next(row for row in result['cases'] if row['case'] == identity)


class KnownDiscoveryTests(unittest.TestCase):
    def test_no_observations_never_satisfy_the_known_case_list(self):
        result = validate({}, {}, {}, {})
        self.assertEqual(result['status'], 'missing-evidence')
        self.assertEqual(len(result['missing']), 13)
        self.assertTrue(all(row['checks'] for row in result['cases']))
        self.assertIn('matching whole-pool reservation', markdown(result))

    def test_helper_discovery_requires_both_source_calls_and_historical_events(self):
        before = {'source_sites': [{'kind': 'cargo-command',
                  'path': 'support/independent_executable.rs', 'line': 1}]}
        source = {'support/independent_executable.rs': 'let command = Command::new("cargo");',
                  'physical_record_journeys/observer.rs': 'OFFLINE_ROOT_OBSERVER.path();'}
        observations = {'scenario': lane([
            event('store::physical_record_journeys$crash::whole_value', 287.1),
            event('store::physical_record_journeys$record_world_survives_fresh_processes', 600.2, 'failed')])}
        result = get(validate(before, source, observations, {}), 'helper-executable-builds')
        self.assertEqual(result['status'], 'found')
        del source['support/independent_executable.rs']
        self.assertIn('source compiler launch', get(validate(before, source, observations, {}),
                                                   'helper-executable-builds')['missing'])

    def test_authority_lock_evidence_is_bound_to_its_own_failure_output(self):
        unrelated = ' TIMEOUT [600s] store::other failure\nBlocking waiting for file lock on package cache\n'
        attributed = (' TIMEOUT [900s] recovery::authority_compile recovery_entry\n'
                      'Blocking waiting for file lock on package cache\n'
                      'Checking worth-store v0.1.0\n'
                      '{"type":"test","event":"failed","name":"recovery::authority_compile$recovery_entry"}\n')
        self.assertEqual(diagnostic_hits({'log': unrelated}, '::authority_compile ', 'file lock'), [])
        observed = diagnostic_hits({'log': unrelated + attributed}, '::authority_compile ', 'file lock')
        self.assertEqual(len(observed), 1)
        self.assertEqual(observed[0]['line'], 4)
        self.assertIn('authority_compile', observed[0]['attribution'])

    def test_environment_total_is_derived_from_distinct_source_verified_calls(self):
        inventory = {'environment_count': 18, 'environments': []}
        documents = {}
        for index in range(18):
            identity = f'suite-{index}'
            path = f'courts/{index}.rs'
            documents[path] = f'run_cargo_ui_fixture_suite("{identity}", "production", "diagnostic-test")'
            inventory['environments'].append({'suite_identity': identity,
                'feature_identity': 'production', 'profile_identity': 'diagnostic-test',
                'call_location': {'path': path, 'line': 1}, 'fixtures': []})
        result = get(validate({}, documents, {}, {}, inventory), 'compiler-boundary-per-fixture-environments')
        measured = next(check for check in result['checks'] if check['name'].startswith('18 '))
        self.assertEqual(len(measured['evidence']), 18)
        inventory['environments'][-1] = copy.deepcopy(inventory['environments'][0])
        result = get(validate({}, documents, {}, {}, inventory), 'compiler-boundary-per-fixture-environments')
        self.assertIn('18 source-verified environments', result['missing'])

    def test_parameterized_environment_requires_its_real_caller_and_fixture_literal(self):
        inventory = {'environments': []}
        helper = ('fn run_case(suite: &str) { run_cargo_ui_fixture_suite(suite, '
                  '"production", "diagnostic-test"); }')
        documents = {'support.rs': helper}
        for index in range(18):
            path = f'caller-{index}.rs'
            documents[path] = f'run_case("suite-{index}", "fixture-{index}.rs");'
            inventory['environments'].append({'suite_identity': f'suite-{index}',
                'feature_identity': 'production', 'profile_identity': 'diagnostic-test',
                'call_location': {'path': 'support.rs', 'line': 1},
                'fixtures': [{'name': f'fixture-{index}.rs',
                              'declaration_location': {'path': path, 'line': 1}}]})
        identity = 'compiler-boundary-per-fixture-environments'
        row = get(validate({}, documents, {}, {}, inventory), identity)
        self.assertNotIn('18 source-verified environments', row['missing'])
        documents['caller-17.rs'] = 'run_case("wrong-suite", "fixture-17.rs");'
        row = get(validate({}, documents, {}, {}, inventory), identity)
        self.assertIn('18 source-verified environments', row['missing'])

    def test_whole_pool_claim_needs_a_matching_actual_filter(self):
        name = 'pkg::production_entry$ordered::branch_transitions_oracle::all_points'
        source = {'process/branch_transitions_oracle/mod.rs': 'for nth in 1..=count { run(nth); }'}
        observed = {'scenario': lane([event(name, 211)])}
        config = {'profile': {'default': {'overrides': [
            {'filter': 'test(other_case)', 'threads-required': 'num-test-threads'}]}}}
        identity = 'sequential-pool:branch_transitions_oracle'
        self.assertIn('matching whole-pool reservation', get(validate({}, source, observed, {}, configuration=config), identity)['missing'])
        config['profile']['default']['overrides'][0]['filter'] = 'test(branch_transitions_oracle)'
        self.assertEqual(get(validate({}, source, observed, {}, configuration=config), identity)['status'], 'found')
        self.assertIsNone(pool_match('not test(branch_transitions_oracle)', {'name': name}))
        self.assertFalse(pool_match('binary_id(=different) & test(branch_transitions_oracle)', {'name': name}))

    def test_formal_build_and_run_are_measured_separately_from_test_work(self):
        observations = {'formal': lane([event('formal::court$proof', .3)],
                                       {'build_seconds': 1599, 'run_seconds': .3})}
        self.assertEqual(get(validate({}, {}, observations, {}), 'formal-build-dominates-run')['status'], 'found')
        observations['formal']['telemetry'] = {'test_work_seconds': .3}
        self.assertEqual(len(get(validate({}, {}, observations, {}), 'formal-build-dominates-run')['missing']), 2)

    def test_serial_overlap_uses_complete_module_names(self):
        observations = {'workspace': lane(log='workspace.log'),
                        'scenario': lane([event('pkg::court$domain::proof')], log='partition.log')}
        raw = {'workspace.log': 'Command: cargo test --workspace --no-fail-fast\n Running tests/court.rs (target/debug/deps/court-0123456789abcdef.exe)\ntest other::proof ... ok\n'}
        identity = 'serial-workspace-repeats-partitions'
        self.assertIn('same binary and full test names repeated', get(validate({}, {}, observations, raw), identity)['missing'])
        raw['workspace.log'] += ' Running tests/other.rs (target/debug/deps/other-0123456789abcdef.exe)\ntest domain::proof ... ok\n'
        self.assertIn('same binary and full test names repeated', get(validate({}, {}, observations, raw), identity)['missing'])
        raw['workspace.log'] += ' Running tests/court.rs (target/debug/deps/court-0123456789abcdef.exe)\ntest domain::proof ... ok\n'
        self.assertEqual(get(validate({}, {}, observations, raw), identity)['status'], 'found')

    def test_raw_nextest_json_is_accepted_amid_human_cargo_progress(self):
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / 'formal.log'
            path.write_text('Finished `ci-test` profile target(s) in 26m 39s\n' +
                            json.dumps(event('formal::court$proof', .3)) + '\n Summary [0.305s]\n')
            measured = read_lane(path)
        row = get(validate({}, {}, {'formal': measured}, {}), 'formal-build-dominates-run')
        self.assertEqual(row['status'], 'found')
        self.assertEqual(row['checks'][0]['evidence'][0]['build_seconds'], 1599)


if __name__ == '__main__':
    unittest.main()
