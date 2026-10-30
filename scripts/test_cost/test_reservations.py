from pathlib import Path
import tempfile
import tomllib
import unittest

from reservations import configuration, original_reservations, recommended


class MeasuredReservationsTests(unittest.TestCase):
    def test_recommendation_uses_peak_observed_cpu_parallelism_without_a_timing_gate(self):
        rows = [dict(binary='court', test='claim', cpu_seconds=cpu, wall_seconds=2,
                     reserved_slots=8, accounting='complete') for cpu in (1, 5)]
        self.assertEqual(recommended(rows), {'court\nclaim': 3})

    def test_generated_config_preserves_timeouts_and_places_measured_rule_first(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / 'original.toml'
            source.write_text('[profile.default]\nslow-timeout={period="60s",terminate-after=10}\n'
                              '[[profile.default.overrides]]\nfilter="all()"\nthreads-required=8\n')
            output = root / 'generated.toml'
            configuration(source, output, 'default', {'court\nclaim': 2}, root / 'account.py')
            value = tomllib.loads(output.read_text())
            self.assertEqual(value['profile']['default']['slow-timeout']['terminate-after'], 10)
            self.assertEqual([row['threads-required'] for row in value['profile']['default']['overrides']], [2, 8])

    def test_nextest_filter_match_precedence_is_preserved(self):
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / 'nextest.toml'
            path.write_text('[[profile.default.overrides]]\nfilter="first"\nthreads-required=2\n'
                            '[[profile.default.overrides]]\nfilter="second"\nthreads-required=8\n')
            rows = [dict(binary='court', test='claim', ignored=False)]
            self.assertEqual(original_reservations(path, 'default', rows, lambda *_: rows), {'court\nclaim': 2})


if __name__ == '__main__':
    unittest.main()
