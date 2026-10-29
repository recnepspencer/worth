import unittest
from telemetry import setup_counts, slot_use


class TelemetryTests(unittest.TestCase):
    def test_identical_inputs_count_per_binary_and_across_lane(self):
        events = [dict(binary=b, builder='world', fingerprint='seed-7') for b in ['a', 'a', 'b']]
        result = setup_counts(events)
        self.assertEqual([r['count'] for r in result['per_binary']], [2, 1])
        self.assertEqual(result['per_lane'][0]['count'], 3)

    def test_reservation_recommendation_uses_cpu_including_descendants(self):
        events = [dict(binary='a', test='sweep', wall_seconds=10, cpu_seconds=11, reserved_slots=8),
                  dict(binary='a', test='parallel', wall_seconds=10, cpu_seconds=70, reserved_slots=8)]
        rows = slot_use(events)['tests']
        self.assertEqual([r['recommended_slots'] for r in rows], [2, 7])
        self.assertEqual([r['over_reserved'] for r in rows], [True, False])

    def test_missing_measurements_are_not_zero(self):
        self.assertEqual(slot_use([])['status'], 'unmeasured')
        self.assertEqual(setup_counts([])['status'], 'unmeasured')


if __name__ == '__main__':
    unittest.main()
