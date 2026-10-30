"""Residue ranks require real event costs and keep unknown attribution explicit."""
import json
from pathlib import Path
import tempfile
import unittest

from residue_report import coverage_summary, generate, markdown, read_inputs, retained_notes


def event(name, seconds, outcome='ok'):
    return {'name': name, 'exec_time': seconds, 'event': outcome}


def lane(name, tests=(), setup=()):
    return {'log': name, 'tests': list(tests), 'setup': list(setup), 'cpu': []}


class ResidueEvidenceTests(unittest.TestCase):
    def test_implementation_and_captured_pass_failure_or_absence_are_independent(self):
        result = generate('store', {}, {}, runtime_lanes=[lane('actual.log', [
            event('court$branch_transitions_oracle::complete', 7),
            event('court$physical_checkpoint_oracle::denied', 9, 'failed')])])
        rows = {row['item']: row for row in result['ranked'] + result['unranked']}
        self.assertEqual(rows['branch']['implementation'], 'pool implemented')
        self.assertEqual(rows['branch']['execution_evidence'],
                         {'status': 'passed', 'passed': 1, 'failed': 0, 'terminal_observations': 1})
        self.assertIn('captured passed', rows['branch']['status'])
        self.assertEqual(rows['checkpoint']['execution_evidence']['status'], 'failed')
        self.assertIn('captured failed', rows['checkpoint']['status'])
        self.assertEqual(rows['anchored']['execution_evidence']['status'], 'unmeasured')
        self.assertEqual(rows['anchored']['implementation'], 'pool implemented')
        self.assertNotIn('runtime pending', markdown(result))
        self.assertNotIn('Their actual runtime results remain required', markdown(result))

    def test_observed_pass_without_cost_stays_unranked_without_losing_execution_evidence(self):
        result = generate('root', {}, {}, runtime_lanes=[lane('actual.log', [
            {'name': 'court$band_guard_cargo_contract::sentinel', 'event': 'ok'}])])
        self.assertEqual(result['ranked'], [])
        proof = next(row for row in result['unranked'] if row['item'] == 'proof')
        self.assertEqual(proof['implementation'], 'prebuild implemented')
        self.assertEqual(proof['execution_evidence']['status'], 'passed')
        self.assertNotIn('work_seconds', proof)

    def test_home_process_cost_uses_both_recorded_identity_formats_without_inventing_or_mixing_work(self):
        baseline = lane('historical.log', [event('worth_store_relational_home$process_kill::seams', 7)])
        current = lane('initial.log', [
            event('worth-store-court-relational::worth_store_court_relational$process::relational::relational_home::process_kill::seams', 11, 'failed'),
            event('court$process::relational::relational_home::process_kill::scheduled', 999, 'ignored'),
            {'name': 'court$process::relational::relational_home::process_kill::unmeasured', 'event': 'ok'}])
        result = generate('store', {}, {}, baseline_lanes=[baseline], runtime_lanes=[current])
        self.assertEqual([(row['item'], row['work_seconds'], row['frequency']) for row in result['baseline']],
                         [('home-process', 7, 1)])
        self.assertEqual([(row['item'], row['work_seconds'], row['frequency'], row['failed']) for row in result['ranked']],
                         [('home-process', 11, 1, 1)])

    def test_work_rank_is_observed_cost_times_frequency_and_excludes_ignored(self):
        observed = lane('scenario.log', [
            event('court$branch_transitions_oracle::one', 8),
            event('court$branch_transitions_oracle::two', 12),
            event('court$physical_checkpoint_oracle::one', 19),
            event('court$physical_checkpoint_oracle::scheduled', 999, 'ignored')])
        result = generate('store', {}, {}, baseline_lanes=[observed])
        self.assertEqual([row['item'] for row in result['ranked']], ['branch', 'checkpoint'])
        self.assertEqual(result['ranked'][0]['work_seconds'], 20)
        self.assertEqual(result['ranked'][0]['frequency'], 2)
        self.assertIn('baseline only', result['ranked'][0]['ranking_basis'])

    def test_compiler_binary_names_do_not_impersonate_runtime_family_cost(self):
        observed = lane('ui.log', [event(
            'owner::backend_qualification_public_boundary_compile_fail$sealed_boundary', 900)])
        result = generate('store', {}, {}, baseline_lanes=[observed])
        self.assertEqual([row['item'] for row in result['ranked']], ['unclassified'])
        self.assertEqual(result['ranked'][0]['binary'], 'owner::backend_qualification_public_boundary_compile_fail')
        self.assertEqual(result['ranked'][0]['test'], 'sealed_boundary')
        self.assertEqual(result['ranked'][0]['work_seconds'], 900)

    def test_unknown_slow_cases_are_ranked_by_exact_binary_full_test_and_log_without_deduplicating_occurrences(self):
        observed = lane('actual.log', [event('binary-a$owner::unexpected_slow_case', 4),
            event('binary-a$owner::unexpected_slow_case', 6, 'failed'),
            event('binary-b$owner::unexpected_slow_case', 7),
            event('binary-a$branch_transitions_oracle::known', 3),
            event('binary-a$owner::ignored', 999, 'ignored')])
        result = generate('store', {}, {}, runtime_lanes=[observed])
        self.assertEqual(len(result['ranked']), 3)
        unknown = [row for row in result['ranked'] if row['item'] == 'unclassified']
        self.assertEqual([(row['binary'], row['test'], row['frequency'], row['work_seconds'], row['failed'])
                          for row in unknown], [('binary-a', 'owner::unexpected_slow_case', 2, 10, 1),
                                                ('binary-b', 'owner::unexpected_slow_case', 1, 7, 0)])
        self.assertEqual(len(unknown[0]['tests']), 2)
        self.assertEqual([row['item'] for row in result['ranked']].count('branch'), 1)

    def test_fallback_preserves_distinct_logs_and_does_not_rank_missing_or_invalid_terminal_costs(self):
        first = lane('first.log', [event('court$unexpected', 4), event('court$pending', 50, 'started'),
            event('court$negative', -1), event('court$nonfinite', float('inf')), event('court$boolean', True),
            {'name': 'court$unknown', 'event': 'ok'}])
        second = lane('second.log', [event('court$unexpected', 5)])
        result = generate('query', {}, {}, runtime_lanes=[first, second])
        self.assertEqual([(row['log'], row['frequency'], row['work_seconds']) for row in result['ranked']],
                         [('second.log', 1, 5), ('first.log', 1, 4)])

    def test_markdown_bounds_the_readable_ranking_and_json_retains_every_record(self):
        result = generate('bank', {}, {}, runtime_lanes=[lane('actual.log',
            [event(f'court$unexpected_{i}', i) for i in range(100)])])
        rendered = markdown(result)
        self.assertEqual(len(result['ranked']), 100)
        self.assertIn('Showing 80 of 100 ranked records', rendered)
        self.assertIn('[residue.json](residue.json)', rendered)
        self.assertEqual(sum(line.startswith('| ') and not line.startswith('| Rank') for line in rendered.splitlines()), 80)
        self.assertNotIn('court$unexpected_0 |', rendered)

    def test_authority_prefix_family_preserves_real_charge_cut_claim_and_waits_for_terminal_observation(self):
        name = ('worth-query-execution::worth_query_execution$domain_computation::authority_cause::tests::'
                'authority_evidence_exposes_request_resources_by_value_through_the_commit_door')
        pending = generate('query', {}, {}, runtime_lanes=[lane('running.log', [event(name, 800, 'started')])])
        self.assertEqual(pending['ranked'], [])
        self.assertIn('authority-prefix', {row['item'] for row in pending['unranked']})
        complete = generate('query', {}, {}, runtime_lanes=[lane('finished.log', [event(name, 12)])])
        self.assertEqual(complete['ranked'][0]['item'], 'authority-prefix')
        self.assertEqual(complete['ranked'][0]['work_seconds'], 12)
        self.assertIn('every unique positive charged prefix at required - 1', complete['ranked'][0]['claim'])
        self.assertIn('installed_authorization_world(true)', complete['ranked'][0]['claim'])

    def test_court_relocated_home_process_names_keep_their_cost_family(self):
        observed = lane('run-process.log', [event(
            'court$process::relational::relational_home::process_kill::keeps_selected', 12)])
        result = generate('store', {}, {}, runtime_lanes=[observed])
        self.assertEqual(result['ranked'][0]['item'], 'home-process')

    def test_unknown_costs_receive_no_numeric_rank_or_inferred_zero(self):
        result = generate('query', {'source_counts': {'sleep': 7}},
                          {'source_counts': {'sleep': 15}})
        self.assertEqual(result['ranked'], [])
        self.assertTrue(all('rank' not in row for row in result['unranked']))
        self.assertEqual(result['current_source_counts']['sleep'], 15)
        rendered = markdown(result)
        self.assertIn('unknown | unknown', rendered)
        self.assertNotIn('measured speedup', rendered)

    def test_runtime_observations_replace_baseline_ranking_without_mixing_scales(self):
        baseline = lane('baseline.log', [event('court$branch_transitions_oracle::one', 500),
                                         event('court$physical_checkpoint_oracle::one', 700)])
        current = lane('current.log', [event('court$branch_transitions_oracle::one', 20)])
        result = generate('store', {}, {}, baseline_lanes=[baseline], runtime_lanes=[current])
        self.assertEqual(len(result['ranked']), 1)
        self.assertEqual(result['ranked'][0]['work_seconds'], 20)
        self.assertEqual(len(result['baseline']), 2)
        self.assertIn('checkpoint', {row['item'] for row in result['unranked']})

    def test_failed_attempt_cost_is_retained_and_not_labeled_a_pass(self):
        result = generate('bank', {}, {}, runtime_lanes=[lane('failed.log', [
            event('court$external_effect_dispatch::late_completion', 90, 'failed')])])
        self.assertEqual(result['ranked'][0]['failed'], 1)
        self.assertEqual(result['ranked'][0]['tests'][0]['event'], 'failed')
        self.assertEqual(result['ranked'][0]['work_seconds'], 90)

    def test_distinct_lanes_remain_separate_and_duplicate_log_inputs_refuse(self):
        first = lane('first.log', [event('court$branch_transitions_oracle::one', 4)])
        second = lane('second.log', [event('court$branch_transitions_oracle::one', 5)])
        result = generate('store', {}, {}, baseline_lanes=[first, second])
        self.assertEqual([row['work_seconds'] for row in result['ranked']], [5, 4])
        with self.assertRaises(ValueError):
            generate('store', {}, {}, baseline_lanes=[first, first])

    def test_inventory_count_comes_from_actual_proof_rows_without_claiming_execution(self):
        proof = {'status': 'pass', 'tests': [{'ignored': False}] * 4 + [{'ignored': True}],
                 'zero': [], 'multiple': []}
        result = coverage_summary(proof)
        self.assertEqual((result['inventory'], result['landing'], result['scheduled_only']), (5, 4, 1))
        self.assertIn('not execution evidence', result['runtime_executed'])
        self.assertIsNone(coverage_summary({'status': 'unmeasured'})['inventory'])

    def test_setup_frequency_and_ir_size_do_not_fabricate_exclusive_builder_seconds(self):
        current = {'llvm_lines': {'testkit': [{'name': 'owner::generic', 'lines': 4000, 'copies': 8}]}}
        setup = [{'builder': 'world', 'fingerprint': 'same-inputs'}] * 3
        result = generate('root', {}, current, runtime_lanes=[lane('setup.log', setup=setup)])
        self.assertEqual(result['telemetry']['setup'][0]['count'], 3)
        self.assertEqual(result['telemetry']['llvm'][0]['copies'], 8)
        self.assertEqual(result['ranked'], [])
        self.assertIn('Exclusive builder time unmeasured', markdown(result))

    def test_setup_sidecar_and_llvm_inputs_do_not_claim_a_current_runtime_run(self):
        baseline = lane('baseline.log', [event('court$branch_transitions_oracle::one', 12)])
        setup = lane('setup.jsonl', setup=[{'builder': 'world', 'fingerprint': 'same'}] * 2)
        result = generate('store', {}, {}, baseline_lanes=[baseline], telemetry_lanes=[setup],
                          llvm=[('testkit', [{'name': 'owner::helper', 'lines': 70, 'copies': 3}])])
        self.assertIn('baseline only', result['ranking_basis'])
        self.assertEqual(result['ranked'][0]['work_seconds'], 12)
        self.assertEqual(result['telemetry']['setup'][0]['count'], 2)
        self.assertEqual(result['telemetry']['llvm'][0]['lines'], 70)

    def test_raw_sidecar_replaces_its_census_summary_instead_of_double_counting(self):
        setup = lane('setup.jsonl', setup=[{'builder': 'world', 'fingerprint': 'same'}] * 2)
        current = {'lanes': {'telemetry': {'log': 'setup.jsonl', 'repeated_setup': {
            'per_lane': [{'builder': 'world', 'fingerprint': 'same', 'count': 2}]}}}}
        result = generate('store', {}, current, telemetry_lanes=[setup])
        self.assertEqual(len(result['telemetry']['setup']), 1)
        self.assertEqual(result['telemetry']['setup'][0]['count'], 2)
        with self.assertRaises(ValueError):
            generate('store', {}, {}, runtime_lanes=[setup], telemetry_lanes=[setup])

    def test_retained_family_judgment_excludes_superseded_source_status_paragraphs(self):
        store = ('stale initial counts\n## Safe-loop inventory\npreserve per-case witness\n'
                 '## Setup reuse\nroot-bound filesystem identity\n## Mechanical queue\nnot implemented\n')
        kept = retained_notes('store', store)
        self.assertIn('per-case witness', kept)
        self.assertIn('filesystem identity', kept)
        self.assertNotIn('stale initial', kept)
        self.assertNotIn('not implemented', kept)

    def test_small_fixture_log_is_parsed_without_workspace_scanning(self):
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / 'run.log'
            path.write_text(json.dumps(dict(type='test', **event('court$maintenance::bounded', .25))) + '\n')
            observed = read_inputs({}, [str(path)])
            result = generate('bank', {}, {}, runtime_lanes=observed)
            self.assertEqual(result['ranked'][0]['work_seconds'], .25)
            with self.assertRaises(FileNotFoundError):
                read_inputs({}, [str(path.with_name('missing.log'))])


if __name__ == '__main__':
    unittest.main()
