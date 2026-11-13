"""Committed-Git driver/consumer protocol with injected command effects, no Cargo."""
import contextlib
import copy
import hashlib
import io
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import query_gate
from query_gate_normal import compile_plan, command_plan, assert_unchanged
from query_gate_cli import run_normal
from landing_plan import assess_normal
from measured_slots import load as load_slots, retain as retain_slots
from run_lane import retain_recommendations
import test_query_gate_normal as fixtures


def rebind_artifacts(output, summary):
    summary = copy.deepcopy(summary)
    summary['artifacts'] = {p.relative_to(output).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest()
                            for p in output.rglob('*') if p.is_file() and p.name != 'summary.json'}
    (output/'summary.json').write_text(json.dumps(summary))


def compiler_contract(output, path, repository, environment, command):
    # Inject the existing capture shape; no compiler or graph execution is claimed.
    from unit_plan import inputs
    value = dict(schema=1, type='compiler-unit-plan', units=[],
                 graph=dict(version=1, units=[], roots=[]),
                 contract=inputs(command, repository, environment))
    path = output/path; path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value))
    path.with_name(path.name.removesuffix('-units.json')+'.log').write_text(json.dumps(
        dict(type='cargo-invocation', command=command, compiler_contract=value['contract'])))


class NormalProtocolTests(unittest.TestCase):
    setUp = fixtures.NormalSourceTests.setUp
    git = fixtures.NormalSourceTests.git
    write = fixtures.NormalSourceTests.write
    commit = fixtures.NormalSourceTests.commit
    def cpu_capture(self, path, cpu, slots=8):
        path.parent.mkdir(parents=True, exist_ok=True)
        event = dict(type='test-cost-cpu', binary='court', test='contract::case',
                     cpu_seconds=cpu, wall_seconds=2, reserved_slots=slots, exit=0, accounting='complete')
        path.write_text('Command: ["python", "phase_telemetry.py"]\n'
            f'Working directory: {self.root}\nHEAD: {self.git("rev-parse", "HEAD")}\n'
            f'Git status --short empty: {not bool(self.git("status", "--short"))}\nStart: 2026-01-01T00:00:00+00:00\n'
            'End: 2026-01-01T00:00:02+00:00\nExit code: 0\n'+json.dumps(event))
        return [event]

    def prepare(self, rust=False):
        self.write('.gitignore', '__pycache__/\ntarget/\n')
        self.commit()
        self.base = self.git('rev-parse', 'HEAD')
        self.git('update-ref', 'refs/remotes/origin/master', self.base)
        path = 'crates/worth-execution/src/lib.rs' if rust else 'scripts/ci/query_gate_normal.py'
        self.write(path, '// real edit\n' if rust else '# source-only implementation\n')
        self.commit()
        return compile_plan(self.root, self.base)

    def gate(self, plan, fail=None, move=False):
        root = self.root
        owner = self
        class Gate:
            def __init__(self, out, affected):
                self.out, self.plan_value = out, affected
                self.env = {'WORTH_IMPLEMENTATION_BUILD_ID': 'b8d62c25-8c20-4204-b12f-04b27574955f'}
            def plan(self, suites):
                owner.assertEqual(suites, plan['suites'])
            def command(self, label, argv):
                (self.out / (label + '.log')).write_text('injected command boundary, not execution proof\n')
                if move:
                    (root / 'moved').write_text('dirty input')
                return dict(name=label, argv=argv, exit=1 if label == fail else 0,
                            log=label+'.log', seconds=0)
            def run(self, suite):
                raise AssertionError('source-only must not call a Rust executor')
        return Gate

    def test_source_only_first_adoption_and_manual_consumer_are_honest(self):
        plan = self.prepare()
        output = self.root / 'target/fresh'
        with patch.object(query_gate, 'ROOT', self.root), patch.object(query_gate, 'Gate', self.gate(plan)), \
                contextlib.redirect_stdout(io.StringIO()):
            self.assertEqual(query_gate.main(['--changed-since', self.base, '--out', str(output)]), 0)
            result = assess_normal(self.root, output, plan['source']['commit'], self.base)
        self.assertFalse(result['push_authority'])
        self.assertEqual(result['workspace_coverage'], {})
        self.assertIn('NO RUST COURT', result['label'])
        summary = json.loads((output / 'summary.json').read_text())
        changed = dict(summary, application_build='067dcfd0-0f4d-4ffb-b80e-7e3a37012372')
        (output/'summary.json').write_text(json.dumps(changed))
        with self.assertRaises(ValueError):
            assess_normal(self.root, output, plan['source']['commit'], self.base)
        (output/'summary.json').write_text(json.dumps(summary))
        self.assertNotIn('normal-clippy', summary['requested_suites'])
        with self.assertRaisesRegex(ValueError, 'fresh output'):
            run_normal(query_gate, output, plan)
        for key, value in [('schema', 'unknown'), ('workspace_coverage', {'root':'full'}),
                           ('normal_plan', {}), ('requested_suites', []), ('output', 'another-run'), ('status', 'red')]:
            altered = dict(summary); altered[key] = value
            (output / 'summary.json').write_text(json.dumps(altered))
            with self.subTest(key=key), self.assertRaises(ValueError):
                assess_normal(self.root, output, plan['source']['commit'], self.base)
        (output / 'summary.json').write_text(json.dumps(summary))
        (output / 'constitution.log').write_text('overwritten')
        with self.assertRaisesRegex(ValueError, 'output'):
            assess_normal(self.root, output, plan['source']['commit'], self.base)

    def test_failure_and_source_movement_do_not_qualify(self):
        plan = self.prepare()
        with patch.object(query_gate, 'ROOT', self.root), contextlib.redirect_stdout(io.StringIO()):
            with patch.object(query_gate, 'Gate', self.gate(plan, fail='constitution')):
                self.assertEqual(run_normal(query_gate, self.root/'target/red', plan), 1)
            with self.assertRaises(ValueError):
                assess_normal(self.root, self.root/'target/red', plan['source']['commit'], self.base)
            with patch.object(query_gate, 'Gate', self.gate(plan, move=True)), self.assertRaises(ValueError):
                run_normal(query_gate, self.root/'target/moved', plan)
        self.assertFalse((self.root/'target/moved/summary.json').exists())

    def test_scoped_clippy_and_bootstrap_contract_are_literal_and_locked(self):
        plan = self.prepare(rust=True)
        commands = command_plan(plan, query_gate)
        argv = commands['normal-clippy'][0][1]
        self.assertEqual([argv[i+1] for i,p in enumerate(argv) if p == '-p'], plan['packages'])
        self.assertNotIn('--workspace', argv)
        self.assertEqual(argv[-3:], ['--', '-D', 'warnings'])
        self.assertIn('--locked', argv); self.assertIn('--offline', argv)
        self.assertNotIn('prepare-constitution', [name for name,_ in commands['source-checks']])
        script = (query_gate.ROOT/'scripts/prepare-constitution-hook.ps1').read_text()
        builds = [line for line in script.splitlines() if '& cargo build' in line]
        self.assertEqual(len(builds), 2)
        self.assertTrue(all('--locked --offline' in line for line in builds))

    def test_cli_rejects_mixed_request_and_unsupported_input_before_effects(self):
        self.prepare()
        with patch.object(query_gate, 'ROOT', self.root), patch.object(query_gate, 'Gate') as effect, \
                contextlib.redirect_stderr(io.StringIO()):
            for flags in (['--full'], ['execution'], ['--census'], ['--list']):
                with self.subTest(flags=flags), self.assertRaises(SystemExit):
                    query_gate.main(['--changed-since', self.base, *flags])
            self.write('unknown.png', 'not an admitted asset'); self.commit()
            with self.assertRaises(SystemExit):
                query_gate.main(['--changed-since', self.base])
            effect.assert_not_called()

    def test_real_cost_owner_positive_and_failed_missing_extra_repeated_outcomes(self):
        from query_gate_cost import CostGate
        from test_query_gate_cost import binary
        self.write('.gitignore', '__pycache__/\ntarget/\n')
        committed = self.root/'scripts/test_cost/slots/root'
        seed = self.root/'target/seed-telemetry.log'
        retain_slots(committed, self.cpu_capture(seed, 6), self.git('rev-parse', 'HEAD'), seed)
        original_policy = {p.name: p.read_bytes() for p in committed.iterdir()}
        plan = self.prepare(rust=True)
        real_gate = query_gate.Gate
        for mutation in (None, 'empty-docs', 'failed', 'missing', 'extra', 'repeated'):
            with self.subTest(mutation=mutation):
                output = self.root / ('target/native-' + str(mutation))
                class Runner:
                    def __init__(self, repository, environment):
                        self.environment, self.logs = environment, []
                def execute(repository, workspace, lanes, out, cargo_runner, queries,
                            include_docs, documentation_packages, recommendation_directory=None):
                    self.assertEqual(documentation_packages, plan['doc_packages'])
                    discovered = {'rust-suites': {
                        'owner': binary('worth-execution', 'lib', [('owner_case', False)]),
                        'court': binary('worth-court-execution', 'test', [('contract::case', False)])}}
                    queries(discovered)
                    events = [dict(type='test', event='ok', name=(
                        discovered['rust-suites'][b]['package-name'] + '::' +
                        discovered['rust-suites'][b]['binary-name'] + '$' + name))
                        for cases in queries.expected.values() for b, name in sorted(cases)]
                    if mutation == 'failed': events[0]['event'] = 'failed'
                    if mutation == 'missing': events.pop()
                    if mutation == 'extra': events.append(dict(events[0], name=events[0]['name']+'extra'))
                    if mutation == 'repeated': events.append(dict(events[0]))
                    native = out/'native.log'
                    native.write_text('Command: ["cargo", "nextest", "run"]\n'+
                                      '\n'.join(json.dumps(r) for r in events))
                    snippets = [dict(binary=name+'::doc', test='src/lib.rs (line 1)', ignored=False)
                                for name in documentation_packages]
                    if mutation == 'empty-docs': snippets = []
                    docs = out/'doctests.log'
                    docs.write_text('Command: ["cargo", "test", "--doc"]\n'+'\n'.join(
                        json.dumps(dict(type='test', event='ok', name=r['binary']+'$'+r['test'])) for r in snippets))
                    cargo_runner.logs += [native, docs]
                    compiler_contract(output, 'test-cost/root/build/binaries-build-units.json',
                        self.root, cargo_runner.environment, ['cargo', 'nextest', 'list',
                        '--manifest-path', str(self.root/'Cargo.toml'), '--list-type', 'binaries-only'])
                    compiler_contract(output, 'test-cost/root/documentation/doctests-units.json',
                        self.root, cargo_runner.environment, ['cargo', 'test', '--doc'])
                    telemetry = out/'telemetry.log'
                    self.cpu_capture(telemetry, 1, slots=3)
                    destination = recommendation_directory or committed
                    retain_recommendations(out, destination, telemetry, plan['source']['commit'], {},
                                           {'court\ncontract::case'})
                    return dict(documentation=dict(status='pass', exit_code=0,
                                                   selections=snippets, inventory=snippets))
                def factory(out, affected):
                    gate = real_gate(out, affected=affected)
                    gate.cost = CostGate(self.root, out/'test-cost', gate.env, execute, Runner, affected=affected)
                    def command(label, argv):
                        (out/(label+'.log')).write_text('injected command effect')
                        if argv[:2] == ['cargo', 'clippy']:
                            compiler_contract(output, label+'-units.json', self.root, gate.env, argv)
                        return dict(name=label, argv=argv, exit=0, log=label+'.log')
                    gate.command = command
                    return gate
                with patch.object(query_gate, 'ROOT', self.root), patch.object(query_gate, 'Gate', factory), \
                        contextlib.redirect_stdout(io.StringIO()):
                    code = run_normal(query_gate, output, plan)
                self.assertEqual(code, 0 if mutation in (None, 'empty-docs') else 1)
                self.assertEqual(original_policy, {p.name: p.read_bytes() for p in committed.iterdir()})
                self.assertEqual(load_slots(committed), {'court\ncontract::case': 3})
                self.assertEqual(load_slots(output/'test-cost/root/run-local-recommendations'),
                                 {'court\ncontract::case': 1})
                if mutation in (None, 'empty-docs'):
                    assessed = assess_normal(self.root, output, plan['source']['commit'], self.base)
                    self.assertEqual(assessed['workspace_coverage'], {'root':'partial'})
                    if mutation is None: self.check_assessment_refusals(output, plan)
                else:
                    with self.assertRaises(ValueError):
                        assess_normal(self.root, output, plan['source']['commit'], self.base)

    def check_assessment_refusals(self, output, plan):
        summary = json.loads((output/'summary.json').read_text())
        def assess():
            return assess_normal(self.root, output, plan['source']['commit'], self.base)
        for suite in ('execution', 'root-doc'):
            path = output/(suite+'.json'); original = path.read_text(); value = json.loads(original)
            for key in ('tests', 'passed', 'ignored', 'failed', 'missing', 'names', 'chunks',
                        'logs', 'failure', 'no_tests'):
                changed = copy.deepcopy(value); del changed[key]
                path.write_text(json.dumps(changed)); rebind_artifacts(output, summary)
                with self.subTest(suite=suite, missing=key), self.assertRaises(ValueError):
                    assess()
            if suite == 'root-doc':
                changed = copy.deepcopy(value); del changed['coverage']['documentation']
                path.write_text(json.dumps(changed)); rebind_artifacts(output, summary)
                with self.assertRaises(ValueError): assess()
            for key, replacement in (('application_build', '067dcfd0-0f4d-4ffb-b80e-7e3a37012372'),
                                     ('passed', True), ('chunks', []), ('names', [])):
                changed = dict(value); changed[key] = replacement
                path.write_text(json.dumps(changed)); rebind_artifacts(output, summary)
                with self.subTest(suite=suite, changed=key), self.assertRaises(ValueError): assess()
            path.write_text(original)
        rebind_artifacts(output, summary)
        for identity in ('067dcfd0-0f4d-4ffb-b80e-7e3a37012372', 'invalid'):
            changed = dict(summary, application_build=identity)
            (output/'summary.json').write_text(json.dumps(changed))
            with self.assertRaises(ValueError): assess()
        for path in output.rglob('*-units.json'):
            original = path.read_text(); changed = json.loads(original)
            changed['contract']['compiler_inputs']['environment']['WORTH_IMPLEMENTATION_BUILD_ID'] = (
                '067dcfd0-0f4d-4ffb-b80e-7e3a37012372')
            path.write_text(json.dumps(changed)); rebind_artifacts(output, summary)
            with self.subTest(identity=path.name), self.assertRaises(ValueError): assess()
            path.write_text(original)
            log = path.with_name(path.name.removesuffix('-units.json')+'.log')
            original_log = log.read_text(); event = json.loads(original_log)
            event['compiler_contract']['compiler_inputs']['environment']['WORTH_IMPLEMENTATION_BUILD_ID'] = (
                '067dcfd0-0f4d-4ffb-b80e-7e3a37012372')
            log.write_text(json.dumps(event)); rebind_artifacts(output, summary)
            with self.subTest(invocation=log.name), self.assertRaises(ValueError): assess()
            log.write_text(original_log)
            path.unlink(); rebind_artifacts(output, summary)
            with self.subTest(missing_artifact=path.name), self.assertRaises(ValueError): assess()
            path.write_text(original)
        rebind_artifacts(output, summary)
        assess()
        policy = self.root/'scripts/test_cost/slots/root/index.json'
        original_policy = policy.read_text(); policy.write_text(original_policy+'\n')
        with self.assertRaisesRegex(ValueError, 'clean committed source'): assess()
        policy.write_text(original_policy)

    def test_original_compiler_overrides_refuse_before_commands(self):
        from lane_environment import LaneEnvironmentRefusal
        plan = self.prepare()
        with patch.dict('os.environ', {'CARGO_PROFILE_DEV_DEBUG': '0'}), \
                patch.object(query_gate.Gate, 'command') as command:
            with self.assertRaises(LaneEnvironmentRefusal):
                query_gate.Gate(self.root/'target/not-run', affected=plan)
        command.assert_not_called()

    def test_normal_issues_fresh_identity_instead_of_reusing_environment_or_receipt(self):
        plan = self.prepare()
        old = '067dcfd0-0f4d-4ffb-b80e-7e3a37012372'
        with patch.dict('os.environ', {'WORTH_IMPLEMENTATION_BUILD_ID': old,
                                    'WORTH_CONSTITUTION_ROOT': '/foreign/source', 'WORTH_REPO': '/foreign/source'}), \
                patch('query_gate.verification_identity') as reuse:
            gate = query_gate.Gate(self.root/'target/not-run', affected=plan)
            gate.plan([])
        reuse.assert_not_called()
        self.assertNotEqual(gate.env['WORTH_IMPLEMENTATION_BUILD_ID'], old)
        self.assertEqual(gate.env['WORTH_CONSTITUTION_ROOT'], str(query_gate.ROOT))
        self.assertEqual(gate.env['WORTH_REPO'], str(query_gate.ROOT))
