from pathlib import Path
import tempfile
import unittest

from measured_slots import load, retain
from slot_policy import generate, pending_claims
from policy_catalog import files as catalog_files, read as read_catalog


class MeasuredSchedulingPolicyTests(unittest.TestCase):
    def capture(self, directory, name, events):
        events = [dict(type='test-cost-cpu', exit=0, accounting='complete', **event) for event in events]
        path = Path(directory) / name
        path.write_text(f'Command: ["python", "phase_telemetry.py"]\nWorking directory: {directory}\nHEAD: capture-head\nGit status --short empty: True\n'
                        'Start: 2026-01-01T00:00:00+00:00\nEnd: 2026-01-01T00:00:01+00:00\nExit code: 0\n' +
                        '\n'.join(json.dumps(event) for event in events), encoding='utf-8')
        return path, events

    def test_only_measurement_changes_default_slots_and_removes_old_whole_pool_reservations(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            events = [dict(binary='court', test='sequential', cpu_seconds=1, wall_seconds=2, reserved_slots=8),
                      dict(binary='court', test='ordinary', cpu_seconds=1, wall_seconds=2, reserved_slots=1)]
            log, events = self.capture(directory, 'real.log', events)
            retain(directory, events, 'head', log)
            self.assertEqual(load(directory), {'court\nsequential': 1})

    def test_shards_stay_bounded_and_zero_samples_cannot_erase_a_measured_policy(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            events = [dict(binary='court', test=f'claim_{i}', cpu_seconds=3, wall_seconds=2, reserved_slots=1)
                      for i in range(201)]
            log, events = self.capture(directory, 'real.log', events)
            result = retain(directory, events, 'head', log)
            self.assertEqual(result['catalogs']['rules']['records'], 201)
            self.assertEqual(len(catalog_files(directory, result['catalogs']['rules'])), 4)
            self.assertTrue(all(len(path.read_text().splitlines()) <= 400 for path in directory.iterdir()))
            empty, _ = self.capture(directory, 'empty.log', [])
            self.assertEqual(retain(directory, [], 'new', empty)['latest_observation']['status'], 'unmeasured')
            self.assertEqual(len(load(directory)), 201)

    def test_remeasuring_a_reduced_reservation_does_not_restore_the_old_whole_pool_rule(self):
        with tempfile.TemporaryDirectory() as temporary:
            event = dict(binary='court', test='claim', cpu_seconds=1, wall_seconds=2, reserved_slots=8)
            baseline = {'court\nclaim': 8}
            before, events = self.capture(temporary, 'before.log', [event])
            retain(temporary, events, 'head', before, baseline)
            after, events = self.capture(temporary, 'after.log', [dict(event, reserved_slots=1)])
            retain(temporary, events, 'head', after, baseline)
            self.assertEqual(load(temporary), {'court\nclaim': 1})

    def test_partial_runtime_retention_keeps_archives_pending_claims_and_historical_maxima(self):
        with tempfile.TemporaryDirectory() as temporary:
            event = dict(binary='court', test='parallel', cpu_seconds=6, wall_seconds=2, reserved_slots=8)
            before, events = self.capture(temporary, 'before.log', [event, dict(event, test='other', cpu_seconds=1)])
            retain(temporary, events, 'execution-head', before)
            generate(temporary, [before], pending=['court$authority'])
            after, events = self.capture(temporary, 'after.log', [dict(event, cpu_seconds=1)])
            result = retain(temporary, events, 'later-execution-head', after)
            self.assertEqual(load(temporary), {'court\nparallel': 3, 'court\nother': 1})
            self.assertEqual(result['sample_count'], 3)
            self.assertEqual(pending_claims(temporary), ['court$authority'])
            self.assertEqual(len(read_catalog(temporary, result['catalogs']['samples'])), 3)
            self.assertTrue(any(row.get('execution_start_head') == 'execution-head'
                                for row in read_catalog(temporary, result['catalogs']['sources'])))

    def test_retirement_requires_current_inventory_and_never_discards_pending_evidence(self):
        with tempfile.TemporaryDirectory() as temporary:
            event = dict(binary='court', test='keep', cpu_seconds=1, wall_seconds=2, reserved_slots=8)
            before, events = self.capture(temporary, 'before.log', [dict(event, test=name) for name in ('keep', 'retired', 'pending')])
            retain(temporary, events, 'head', before)
            generate(temporary, [before], pending=['court$pending'])
            empty, _ = self.capture(temporary, 'empty.log', [])
            result = retain(temporary, [], 'head', empty, known={'court\nkeep'})
            self.assertEqual(load(temporary), {'court\nkeep': 1, 'court\npending': 1})
            self.assertEqual(result['retired_identities'], 1)
            self.assertEqual(result['sample_count'], 3)
            retain(temporary, [], 'head', empty)
            self.assertNotIn('court\nretired', load(temporary))

    def test_supplied_cpu_data_must_match_the_actual_retained_capture(self):
        with tempfile.TemporaryDirectory() as temporary:
            event = dict(binary='court', test='claim', cpu_seconds=1, wall_seconds=2, reserved_slots=8)
            log, events = self.capture(temporary, 'real.log', [event])
            retain(temporary, events, 'head', log)
            before = (Path(temporary)/'index.json').read_text()
            with self.assertRaisesRegex(ValueError, 'authoritative'):
                retain(temporary, [dict(events[0], cpu_seconds=99)], 'head', log)
            self.assertEqual((Path(temporary)/'index.json').read_text(), before)


if __name__ == '__main__':
    unittest.main()
import json
