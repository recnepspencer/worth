from pathlib import Path
import tempfile
import unittest

from measured_slots import load, retain


class MeasuredSchedulingPolicyTests(unittest.TestCase):
    def test_only_measurement_changes_default_slots_and_removes_old_whole_pool_reservations(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            events = [dict(binary='court', test='sequential', cpu_seconds=1, wall_seconds=2, reserved_slots=8),
                      dict(binary='court', test='ordinary', cpu_seconds=1, wall_seconds=2, reserved_slots=1)]
            retain(directory, events, 'head', 'real.log')
            self.assertEqual(load(directory), {'court\nsequential': 1})

    def test_shards_stay_bounded_and_zero_samples_cannot_erase_a_measured_policy(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            events = [dict(binary='court', test=f'claim_{i}', cpu_seconds=3, wall_seconds=2, reserved_slots=1)
                      for i in range(201)]
            result = retain(directory, events, 'head', 'real.log')
            self.assertEqual(len(result['files']), 3)
            self.assertTrue(all(len(path.read_text().splitlines()) <= 400 for path in directory.glob('*.json')))
            self.assertEqual(retain(directory, [], 'new', 'empty.log')['status'], 'unmeasured')
            self.assertEqual(len(load(directory)), 201)

    def test_remeasuring_a_reduced_reservation_does_not_restore_the_old_whole_pool_rule(self):
        with tempfile.TemporaryDirectory() as temporary:
            event = dict(binary='court', test='claim', cpu_seconds=1, wall_seconds=2, reserved_slots=8)
            baseline = {'court\nclaim': 8}
            retain(temporary, [event], 'head', 'before.log', baseline)
            retain(temporary, [dict(event, reserved_slots=1)], 'head', 'after.log', baseline)
            self.assertEqual(load(temporary), {'court\nclaim': 1})


if __name__ == '__main__':
    unittest.main()
