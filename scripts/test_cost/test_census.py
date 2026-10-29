import json
from pathlib import Path
import tempfile
import unittest

from census import collect, markdown


class CensusFixtureTests(unittest.TestCase):
    def test_small_workspace_reports_cost_sites_logs_and_exact_partition(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            manifest = root / 'Cargo.toml'
            manifest.write_text('[package]\nname="fixture-court-runtime"\nversion="0.1.0"')
            (root / 'tests').mkdir()
            (root / 'tests/court.rs').write_text('fn compiler() { trybuild::TestCases::new(); }')
            log = root / 'run.log'
            log.write_text(json.dumps(dict(type='test', name='compiler', event='ok', exec_time=0.3)))
            rows = [dict(binary='court', test='compiler', ignored=False)]
            result = collect(manifest, root, [('lane', log)], rows, {'lane': rows})
            self.assertEqual(result['source_counts'], {'trybuild': 1})
            self.assertEqual(result['coverage']['status'], 'pass')
            self.assertEqual(result['lanes']['lane']['timing_telemetry']['pass'], 1)
            self.assertIn('unknown', markdown(result))


if __name__ == '__main__':
    unittest.main()
