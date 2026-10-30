import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

from census import collect, includes_scheduled_selection, markdown
from ratchet import source_violations
from ratchet import build_violations
from compiler_units import declarations
from logs import read_lane
from test_compiler_units import event as compiler_event, graph, unit


class CensusFixtureTests(unittest.TestCase):
    def test_failed_attempt_inventory_does_not_change_fresh_artifact_measure(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            manifest = root/'Cargo.toml'
            manifest.write_text('[package]\nname="fixture-court-runtime"\nversion="0.1.0"\n')
            owner = unit('owner')
            marker = {'type': 'cargo-invocation', 'mode': 'check', 'cargo_profile': 'dev',
                      'unit_plan': {'units': declarations(graph(owner)), 'compiler_inputs': 'flags'}}
            error = {'reason': 'compiler-message', 'package_id': owner['pkg_id'], 'target': owner['target'],
                     'message': {'level': 'error', 'message': 'denied'}}
            log = root/'denied.log'
            log.write_text('\n'.join(map(json.dumps, [marker, error, error, marker, error])))
            result = collect(manifest, root, [('lane', log)])
            lane = result['lanes']['lane']
            self.assertEqual(lane['artifacts'], [])
            self.assertEqual(lane['duplicate_graphs'], [])
            self.assertEqual(len(lane['failed_compiler_attempts']), 2)
            self.assertEqual(lane['compiler_attempt_groups'][0]['repeated_units'][0]['compiles'], 2)
            self.assertIn('Combined compiler attempts', markdown(result))
            self.assertIn('2 attempts; 1 declared required units; 1 repeated identical units', markdown(result))

    def test_required_package_variants_and_identical_recompilations_are_reported_separately(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            manifest = root/'Cargo.toml'
            manifest.write_text('[package]\nname="fixture-court-runtime"\nversion="0.1.0"\n')
            (root/'src').mkdir()
            (root/'src/lib.rs').write_text('')
            owner = unit('owner', dependencies=[1])
            def invocation(proof):
                return [dict(type='cargo-invocation', cargo_profile='dev', mode='check',
                             unit_plan={'units': declarations(graph(owner, proof)), 'compiler_inputs': 'flags'}),
                        dict(compiler_event(owner), reason='compiler-artifact', fresh=False)]
            first = invocation(unit('proof'))
            second = invocation(unit('proof', features=['cert']))
            for records, required, repeated in [(first + second, 2, 0), (first + first, 1, 1)]:
                with self.subTest(required=required, repeated=repeated):
                    log = root/'build.log'
                    log.write_text('\n'.join(json.dumps(row) for row in records))
                    result = collect(manifest, root, [('lane', log)])
                    rendered = markdown(result)
                    self.assertIn('Packages compiled >1 time', rendered)
                    self.assertIn(f'{required} declared required units; {repeated} repeated identical units', rendered)
                    self.assertIn('They are not budget violations', rendered)
                    self.assertNotIn('Duplicate graphs (', rendered)
                    self.assertEqual(bool(build_violations(read_lane(log), {'max_compiles_per_unit': 1})),
                                     bool(repeated))

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
