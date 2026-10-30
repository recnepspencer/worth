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

    def test_unusable_clock_samples_are_unmeasured_and_never_fail_a_lane(self):
        event = dict(binary='a', test='claim', wall_seconds=0, cpu_seconds=0, reserved_slots=1)
        self.assertEqual(slot_use([event])['status'], 'unmeasured')
        self.assertEqual(len(slot_use([event])['unmeasured']), 1)

    def test_standalone_cpu_is_measured_but_never_a_nextest_reservation_recommendation(self):
        event = dict(binary='example', test='<main>', wall_seconds=1, cpu_seconds=2, reserved_slots=1)
        result = slot_use([event])
        self.assertEqual(result['status'], 'standalone-only')
        self.assertEqual(result['tests'], [])
        self.assertEqual(result['unmeasured'], [])
        self.assertEqual(result['standalone'][0]['measured_parallelism'], 2)
        self.assertEqual(result['standalone'][0]['reservation_scope'], 'not-nextest-managed')
        ordinary = slot_use([dict(event, test='main')])
        self.assertEqual(ordinary['tests'][0]['recommended_slots'], 2)


if __name__ == '__main__':
    unittest.main()
