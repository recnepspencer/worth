import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

from census import collect, includes_scheduled_selection, markdown
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

    def test_all_workspaces_keep_separate_reports_under_an_explicit_output_root(self):
        from workspace import WORKSPACES
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / '.git').mkdir()
            for workspace, relative in WORKSPACES.items():
                manifest = root / relative
                manifest.parent.mkdir(parents=True, exist_ok=True)
                manifest.write_text(f'[package]\nname="fixture-court-{workspace}"\nversion="0.1.0"')
            output = root / 'separate-output'
            command = [sys.executable, str(Path(__file__).with_name('census.py').resolve()),
                       '--all', '--check-sources', '--output', str(output)]
            observed = subprocess.run(command, cwd=root, capture_output=True, text=True)
            self.assertEqual(observed.returncode, 0, observed.stdout + observed.stderr)
            self.assertEqual({path.name for path in output.iterdir()}, set(WORKSPACES))
            for workspace, relative in WORKSPACES.items():
                result = json.loads((output / workspace / 'census.json').read_text())
                self.assertEqual(Path(result['manifest']), Path(relative))
                self.assertTrue((output / workspace / 'census.md').is_file())
            self.assertFalse((root / 'target').exists())

    def test_scheduled_forms_are_refused_before_measurement_starts(self):
        for arguments in [
                ['--ignored'], ['--include-ignored'], ['--include-ignored=true'],
                ['--run-ignored', 'all'], ['--run-ignored=all'],
                ['--run-ignored=only'], ['--run-ignored'],
                ['query-scale'], ['query-execution,query-scale'],
                ['--suites=bank,bank-scale'], ['root-rest,root-scale']]:
            with self.subTest(arguments=arguments):
                self.assertTrue(includes_scheduled_selection(['cargo', 'test', *arguments]))
        for arguments in [[], ['--run-ignored', 'default'], ['--run-ignored=default'],
                          ['query-landing'], ['bank-landing'], ['--test', 'ordinary']]:
            with self.subTest(arguments=arguments):
                self.assertFalse(includes_scheduled_selection(['cargo', 'test', *arguments]))

    def test_source_rule_refuses_compiler_launches_but_does_not_gate_telemetry(self):
        sites = [{'kind': kind} for kind in ('cargo-command', 'cargo-environment',
                                            'cargo-metadata', 'trybuild', 'sleep', 'polling-loop')]
        self.assertEqual(len(source_violations(sites)), 4)


if __name__ == '__main__':
    unittest.main()
