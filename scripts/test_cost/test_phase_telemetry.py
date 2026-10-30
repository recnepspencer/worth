import json
from pathlib import Path
import tempfile
import unittest

from phase_telemetry import aggregate


class OptionalPhaseObservationTests(unittest.TestCase):
    def test_corrupt_observation_does_not_discard_the_other_real_observations(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            event = dict(type='test-cost-cpu', binary='court', test='claim', cpu_seconds=2,
                         wall_seconds=3, reserved_slots=1, accounting='complete', exit=7)
            (root / 'cpu.json').write_text(json.dumps(event))
            (root / 'broken.json').write_text('{incomplete')
            (root / 'setup.jsonl').write_text(json.dumps(dict(type='test-cost-setup', binary='court',
                                                            builder='world', fingerprint='recipe')))
            rows = list(aggregate(root, root, 4, 10))
            self.assertEqual([row for row in rows if row['type'] == 'test-cost-cpu'], [event])
            self.assertEqual(sum(row['type'] == 'test-cost-observation-error' for row in rows), 1)
            self.assertEqual(rows[-1], dict(type='test-cost-phases', build_seconds=4, wall_seconds=10))

    def test_missing_optional_directories_and_invalid_clocks_remain_unmeasured(self):
        with tempfile.TemporaryDirectory() as temporary:
            missing = Path(temporary) / 'missing'
            rows = list(aggregate(missing, missing, float('nan'), 1))
            self.assertEqual([row['type'] for row in rows], ['test-cost-observation-error'])


if __name__ == '__main__':
    unittest.main()
