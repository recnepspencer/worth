"""Fixture attempts prove lineage without running Cargo or masking failures."""
import hashlib
import json
from pathlib import Path
import tempfile
import unittest

from executed_coverage import prove
from verification_lineage import compose, load_attempt, markdown


def claim(name, ignored=False):
    return {'binary': 'court', 'test': name, 'ignored': ignored}


def event(name, fate='ok'):
    return {'binary': 'court', 'test': name, 'name': 'court$'+name, 'event': fate,
            'log': 'synthetic.log', 'exec_time': 1}


def attempt(name, known, selected, events):
    return {'name': name, 'known': known, 'selected': selected, 'events': events}


class CompositionTests(unittest.TestCase):
    def test_repairs_preserve_all_failed_and_passing_occurrences_without_a_full_green_run(self):
        first = [claim('one'), claim('two')]
        final = first + [claim('new_dispatch')]
        attempts = [attempt('initial', first, first, [event('one'), event('two', 'failed')]),
                    attempt('repair46', final, final[1:], [event('two', 'failed'), event('new_dispatch')]),
                    attempt('repair40', final, [claim('two')], [event('two')])]
        result = compose(final, attempts)
        self.assertEqual(result['status'], 'composed-evidence-complete')
        self.assertFalse(result['certified'])
        self.assertEqual(result['full_workspace_green_attempts'], [])
        self.assertEqual(result['total_recorded_executions'], 5)
        self.assertEqual(result['added_claims'], [('court', 'new_dispatch')])
        history = next(row['observations'] for row in result['claims'] if row['test'] == 'two')
        self.assertEqual([row['event'] for row in history], ['failed', 'failed', 'ok'])
        self.assertEqual(result['attempts'][1]['prior_failed_claims_rerun'], [('court', 'two')])
        self.assertIn('not one full green run', markdown(result))

    def test_pending_failure_remains_until_a_future_attempt_really_passes_it(self):
        rows = [claim('authority')]
        first = attempt('initial', rows, rows, [event('authority', 'failed')])
        result = compose(rows, [first])
        self.assertEqual(result['status'], 'composed-evidence-incomplete')
        self.assertEqual(result['unresolved'][0]['test'], 'authority')
        fixed = attempt('future', rows, rows, [event('authority')])
        self.assertEqual(compose(rows, [first, fixed])['unresolved'], [])

    def test_previously_passing_affected_claim_requires_a_new_execution(self):
        rows = [claim('one'), claim('two')]
        attempts = [attempt('initial', rows, rows, [event('one'), event('two')]),
                    attempt('repair', rows, [claim('two')], [event('two')])]
        obligation = {'binary': 'court', 'test': 'one', 'since': 'repair', 'reason': 'changed assertion'}
        result = compose(rows, attempts, [obligation])
        self.assertEqual(result['unresolved'][0]['test'], 'one')
        self.assertIn('affected claim not rerun', result['unresolved'][0]['reason'])
        self.assertEqual(result['attempts'][1]['previously_passing_affected_claims_rerun'], [('court', 'two')])

    def test_repeated_continuations_within_one_attempt_are_retained_and_flagged(self):
        rows = [claim('continuation')]
        result = compose(rows, [attempt('one', rows, rows, [event('continuation'), event('continuation')])])
        self.assertEqual(result['total_recorded_executions'], 2)
        self.assertEqual(len(result['claims'][0]['observations']), 2)
        self.assertEqual(result['anomalies'][0]['kind'], 'repeated-within-attempt')
        self.assertEqual(result['status'], 'composed-evidence-incomplete')

    def test_removed_inventory_and_ignored_execution_cannot_disappear(self):
        initial = [claim('retired'), claim('scheduled', True)]
        final = [claim('scheduled', True)]
        result = compose(final, [attempt('initial', initial, [], [event('scheduled')])])
        self.assertEqual(result['removed_claims'], [('court', 'retired')])
        self.assertEqual(result['status'], 'composed-evidence-incomplete')
        self.assertTrue(any(row['kind'] == 'unexpected-or-ignored-execution' for row in result['anomalies']))

    def test_same_log_cannot_be_counted_as_two_attempts(self):
        rows = [claim('one')]
        first = attempt('first', rows, rows, [event('one')])
        second = attempt('second', rows, rows, [event('one')])
        for row in (first, second):
            row['captures'] = [{'path': 'same.log', 'headers': {'Exit code': '0'}}]
        with self.assertRaisesRegex(ValueError, 'same execution log'):
            compose(rows, [first, second])

    def test_command_failure_without_a_failed_claim_remains_an_anomaly(self):
        rows = [claim('one')]
        current = attempt('current', rows, rows, [event('one')])
        current['captures'] = [{'path': str(Path('synthetic.log').resolve()),
                                'headers': {'Exit code': '7'}}]
        result = compose(rows, [current])
        self.assertEqual(result['anomalies'][0]['kind'], 'unexplained-nonzero-command')
        self.assertEqual(result['status'], 'composed-evidence-incomplete')

    def test_an_empty_final_inventory_cannot_certify_an_empty_selection(self):
        with self.assertRaisesRegex(ValueError, 'final inventory'):
            compose([], [attempt('empty', [], [], [])])


class RawAttemptTests(unittest.TestCase):
    def fixture(self, root, fate='failed'):
        native = {'rust-suites': {'owner': {'package-name': 'owner', 'binary-name': 'owner_lib',
                  'kind': 'lib', 'testcases': {'case': {'ignored': False}}}}}
        raw_event = {'type': 'test', 'event': fate, 'name': 'owner::owner_lib$case', 'exec_time': 1}
        path = root/'run.log'
        command = ['cargo', 'nextest', 'run']
        path.write_text('Command: '+json.dumps(command)+'\nHEAD: '+('a'*40)+'\n'
                        'Git status --short empty: False\nStart: 2026-01-01T00:00:00+00:00\n'+
                        json.dumps(raw_event)+'\nEnd: 2026-01-01T00:01:00+00:00\nExit code: '+
                        ('1' if fate == 'failed' else '0')+'\n')
        canonical = {'binary': 'owner', 'test': 'case', 'name': 'owner$case', 'event': fate}
        known = [{'binary': 'owner', 'test': 'case', 'ignored': False}]
        checked = prove(known, {'selection': known}, [canonical], require_full_workspace=True)
        checked['logs'] = {str(path): 60}
        native_path = root/'inventory.json'; native_path.write_text(json.dumps(native))
        report = root/'census.json';report.write_text(json.dumps({'executed_coverage': checked}))
        return {'name': 'initial', 'report': str(report), 'native_inventory': str(native_path)}, path

    def test_raw_logs_recompute_counts_and_normalize_actual_library_alias(self):
        with tempfile.TemporaryDirectory() as temporary:
            specification, path = self.fixture(Path(temporary))
            loaded = load_attempt(specification)
            self.assertEqual(loaded['events'][0]['name'], 'owner$case')
            self.assertEqual(loaded['check']['failed'], [('owner', 'case')])
            self.assertEqual(loaded['captures'][0]['sha256'], hashlib.sha256(path.read_bytes()).hexdigest())
            self.assertEqual(loaded['captures'][0]['headers']['Git status --short empty'], 'False')

    def test_modified_aggregate_cannot_relabel_failed_raw_events(self):
        with tempfile.TemporaryDirectory() as temporary:
            specification, _ = self.fixture(Path(temporary))
            path = Path(specification['report']);report = json.loads(path.read_text())
            report['executed_coverage']['failed'] = []
            path.write_text(json.dumps(report))
            with self.assertRaisesRegex(ValueError, 'contradicts raw observations'):
                load_attempt(specification)

    def test_missing_required_execution_header_is_refused(self):
        with tempfile.TemporaryDirectory() as temporary:
            specification, path = self.fixture(Path(temporary))
            path.write_text(path.read_text().replace('Git status --short empty: False\n', ''))
            with self.assertRaisesRegex(ValueError, 'incomplete execution provenance'):
                load_attempt(specification)


if __name__ == '__main__':
    unittest.main()
