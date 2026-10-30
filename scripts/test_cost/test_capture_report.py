import json
from pathlib import Path
import subprocess
import tempfile
import unittest

from capture_report import captured_selections, recover, retained_logs, telemetry_capture
from cargo_capture import retention_name


class CaptureRecoveryTests(unittest.TestCase):
    def capture(self, root, path, command, body):
        path.parent.mkdir(parents=True, exist_ok=True)
        head, start = 'a'*40, '2026-01-01T00:00:00+00:00'
        text = (f'Command: {json.dumps(command)}\nWorking directory: {root}\nHEAD: {head}\n'
                f'Git status --short empty: True\nStart: {start}\n\n{body}\n'
                'End: 2026-01-01T00:01:00+00:00\nExit code: 0\n')
        path.write_text(text, encoding='utf-8')
        canonical = root/'target/lane-evidence'/retention_name(path.relative_to(root/'target'), head, start)
        canonical.parent.mkdir(parents=True, exist_ok=True)
        canonical.write_text(text, encoding='utf-8')
        return canonical

    def fixture(self, root):
        (root/'src').mkdir()
        (root/'src/lib.rs').write_text('', encoding='utf-8')
        (root/'Cargo.toml').write_text('[package]\nname="owner"\nversion="0.1.0"\n', encoding='utf-8')
        source = root/'target/test-cost/root'
        (source/'build').mkdir(parents=True)
        native = {'rust-suites': {'owner': {'package-name': 'owner', 'binary-name': 'owner', 'kind': 'lib',
                    'testcases': {'case': {'ignored': False}}}}}
        for name in ('inventory', 'selection-owner-unit'):
            (source/f'build/{name}.json').write_text(json.dumps(native), encoding='utf-8')
        artifact = {'reason': 'compiler-artifact', 'fresh': False, 'package_id': 'owner',
                    'target': {'name': 'owner', 'kind': ['lib']}, 'features': [], 'profile': {'test': False},
                    'filenames': [str(root/'target/debug/deps/libowner.rlib')]}
        marker = json.dumps({'type': 'cargo-invocation', 'cargo_profile': 'dev', 'mode': 'build'})
        self.capture(root, source/'build/binaries-build.log', ['cargo', 'test'], marker + '\n' + '\n'.join([json.dumps(artifact)]*2))
        event = {'type': 'test', 'event': 'failed', 'name': 'owner::owner$case', 'exec_time': 1.0}
        self.capture(root, source/'run-owner-unit.log', ['cargo', 'nextest', 'run'], json.dumps(event))
        self.capture(root, source/'documentation/doctests.log', ['cargo', 'test', '--doc'], '')
        (source/'documentation/result.json').write_text(json.dumps({'inventory': [], 'selections': [],
            'events': [], 'status': 'pass', 'exit_code': 0}), encoding='utf-8')
        (source/'reservations.json').write_text('{}', encoding='utf-8')
        (source/'census.json').write_text('current repair census', encoding='utf-8')
        cpu, setup = source/'cpu/initial', source/'setup/initial'
        cpu.mkdir(parents=True); setup.mkdir(parents=True)
        (cpu/'one.json').write_text(json.dumps({'type': 'test-cost-cpu', 'binary': 'owner', 'test': 'case',
            'cpu_seconds': .1, 'wall_seconds': .3, 'reserved_slots': 8, 'accounting': 'complete', 'exit': 1}), encoding='utf-8')
        return source, cpu, setup

    def test_snapshot_retains_failures_and_duplicate_budget_without_overwriting_current_census(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            source, cpu, setup = self.fixture(root)
            test = self
            class Runner:
                logs = []
                def __call__(self, command, cwd, path):
                    test.assertNotEqual(command[0], 'cargo')
                    result = subprocess.run(command, capture_output=True, text=True, check=True)
                    self.logs.append(test.capture(root, path, command, result.stdout))
                    return result
            result = recover(root, 'root', source, source/'initial-run', cpu, setup, Runner())
            self.assertEqual(result['status'], 'fail')
            self.assertEqual(len(result['executed_coverage']['failed']), 1)
            self.assertEqual(len(result['build_ratchet']['violations']), 1)
            self.assertEqual((source/'census.json').read_text(), 'current repair census')
            self.assertEqual(result['capture_recovery']['wall_seconds_telemetry'], 60)
            self.assertIsNone(result['capture_recovery']['phase_build_seconds'])
            self.assertEqual(result['capture_recovery']['measured_slots']['status'], 'unmeasured')

    def test_changed_retained_capture_is_refused_instead_of_merging_a_repair(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            source, _, _ = self.fixture(root)
            path = source/'run-owner-unit.log'
            path.write_text(path.read_text().replace('"failed"', '"ok"'), encoding='utf-8')
            with self.assertRaisesRegex(ValueError, 'differs from original'):
                retained_logs(root, source)

    def test_missing_worktree_status_header_refuses_provenance(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            source, _, _ = self.fixture(root)
            path = source/'run-owner-unit.log'
            text = path.read_text().replace('Git status --short empty: True\n', '')
            path.write_text(text, encoding='utf-8')
            with self.assertRaisesRegex(ValueError, 'incomplete original command evidence'):
                retained_logs(root, source)


    def test_nonzero_execution_command_cannot_be_hidden_by_passing_test_events(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            source, cpu, setup = self.fixture(root)
            build = source/'build/binaries-build.log'
            records = [json.loads(line) for line in build.read_text().splitlines()
                       if line.startswith('{')]
            self.capture(root, build, ['cargo', 'test'],
                         json.dumps(records[0]) + '\n' + json.dumps(records[1]))
            run = source/'run-owner-unit.log'
            event = {'type': 'test', 'event': 'ok', 'name': 'owner::owner$case', 'exec_time': 1.0}
            retained = self.capture(root, run, ['cargo', 'nextest', 'run'], json.dumps(event))
            for path in (run, retained):
                path.write_text(path.read_text().replace('Exit code: 0', 'Exit code: 7'), encoding='utf-8')
            test = self
            class Runner:
                logs = []
                def __call__(self, command, cwd, path):
                    result = subprocess.run(command, capture_output=True, text=True, check=True)
                    self.logs.append(test.capture(root, path, command, result.stdout))
                    return result
            result = recover(root, 'root', source, source/'initial-run', cpu, setup, Runner())
            self.assertEqual(result['executed_coverage']['status'], 'pass')
            self.assertEqual(result['build_ratchet']['violations'], [])
            self.assertEqual(result['status'], 'fail')
            self.assertEqual(len(result['capture_recovery']['execution_command_failures']), 1)


    def test_captured_gate_suites_keep_exact_main_and_native_identities(self):
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary)/'gate-selection.json'
            native = [{'binary': 'owner', 'test': 'case', 'ignored': False},
                      {'binary': 'main-owner', 'test': '<main>', 'ignored': False},
                      {'binary': 'owner', 'test': 'scheduled', 'ignored': True}]
            value = {'coverage': {'status': 'pass', 'selected': 2, 'ignored': 1}, 'suites': {
                'query-execution': {'tests': [['owner', 'case']], 'ignored': [['owner', 'scheduled']]},
                'query-rest': {'tests': [['main-owner', '<main>']], 'ignored': []}}}
            path.write_text(json.dumps(value))
            chosen, evidence = captured_selections(path, native)
            self.assertEqual(chosen['query-rest'][0]['test'], '<main>')
            self.assertEqual(set(chosen), {'query-execution', 'query-rest'})
            self.assertEqual(evidence['kind'], 'captured-gate-selection')
            self.assertEqual(evidence['proof']['status'], 'pass')
            for mutation in ('duplicate', 'missing', 'ignored', 'unknown'):
                altered = json.loads(json.dumps(value))
                tests = altered['suites']['query-rest']['tests']
                if mutation == 'duplicate':
                    tests.append(['owner', 'case'])
                elif mutation == 'missing':
                    tests.clear()
                elif mutation == 'ignored':
                    tests[0] = ['owner', 'scheduled']
                else:
                    tests[0] = ['owner', 'invented']
                path.write_text(json.dumps(altered))
                with self.subTest(mutation=mutation), self.assertRaises(ValueError):
                    captured_selections(path, native)


    def test_explicit_telemetry_reuse_requires_exact_inputs_and_unknown_phase_clocks(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            path = root/'target/recovered/telemetry.log'
            cpu, setup = root/'cpu', root/'setup'
            command = ['python', 'phase_telemetry.py', '--cpu', str(cpu), '--setup', str(setup),
                       '--build-seconds', 'nan', '--wall-seconds', 'nan']
            self.capture(root, path, command, '')
            selected, _ = telemetry_capture(path, cpu, setup)
            self.assertEqual(selected, path)
            with self.assertRaisesRegex(ValueError, 'other process observation'):
                telemetry_capture(path, root/'another-cpu', setup)
            path.write_text(path.read_text().replace('"nan"', '"1.0"'))
            with self.assertRaisesRegex(ValueError, 'phase clocks'):
                telemetry_capture(path, cpu, setup)



if __name__ == '__main__':
    unittest.main()
