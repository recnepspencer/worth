import json
from pathlib import Path
import tempfile
import unittest

from census import collect, markdown
from ratchet import source_violations


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

    def test_source_rule_refuses_compiler_launches_but_does_not_gate_telemetry(self):
        sites = [{'kind': kind} for kind in ('cargo-command', 'cargo-environment',
                                            'cargo-metadata', 'trybuild', 'sleep', 'polling-loop')]
        self.assertEqual(len(source_violations(sites)), 4)


if __name__ == '__main__':
    unittest.main()
