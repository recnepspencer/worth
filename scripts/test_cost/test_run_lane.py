import unittest
import os
import json
import subprocess
from pathlib import Path
from tempfile import TemporaryDirectory
from types import SimpleNamespace
from unittest.mock import patch

from lane_environment import LaneEnvironmentRefusal
from run_lane import (LANDING, LaneExecutionFailure, child_environment, execute,
                      selected_lanes, log_label, retain_recommendations)
from measured_slots import load as load_slots


class GovernedExecutionPlanTests(unittest.TestCase):
    def test_real_execute_retention_leaves_committed_inputs_byte_exact(self):
        with TemporaryDirectory() as temporary:
            root=Path(temporary).resolve();output=root/'target/run';output.mkdir(parents=True)
            def git(*args):return subprocess.check_output(['git',*args],cwd=root).decode().strip()
            git('init','-q');git('config','user.name','native adapter court')
            git('config','user.email','native-adapter@example.invalid')
            (root/'.gitignore').write_text('target/\n')
            (root/'.config').mkdir();config=root/'.config/nextest.toml'
            config.write_text('[profile.ci]\nthreads-required=8\n')
            git('add','.');git('commit','-qm','native adapter source')
            def capture(path,command,events):
                path.write_text('Command: '+json.dumps(command)+'\n'+
                    f'Working directory: {root}\nHEAD: {git("rev-parse","HEAD")}\n'+
                    f'Git status --short empty: {not bool(git("status","--short"))}\n'+
                    'Start: 2026-01-01T00:00:00+00:00\nEnd: 2026-01-01T00:00:02+00:00\nExit code: 0\n'+
                    '\n'.join(json.dumps(e) for e in events))
            cpu=dict(type='test-cost-cpu',binary='court',test='case',cpu_seconds=6,
                     wall_seconds=2,reserved_slots=8,exit=0,accounting='complete')
            seed=output/'seed.log';capture(seed,['python','phase_telemetry.py'],[cpu])
            committed=root/'scripts/test_cost/slots/root';known={'court\ncase'}
            retain_recommendations(output,committed,seed,git('rev-parse','HEAD'),{},known)
            git('add','.');git('commit','-qm','committed scheduler input')
            before={p.name:p.read_bytes() for p in committed.iterdir()}
            discovered={'rust-suites':{'court':{'package-name':'court','binary-name':'court',
                        'kind':'test','testcases':{'case':{'ignored':False}}}}}
            class Runner:
                environment={};logs=[]
                def __call__(self,command,cwd,path):
                    if command[:3]==['cargo','nextest','run']:
                        events=[dict(type='test',event='ok',name='court::court$case')]
                    elif len(command)>1 and Path(command[1]).name=='phase_telemetry.py':
                        events=[dict(cpu,cpu_seconds=1,reserved_slots=3)]
                    else:raise AssertionError('unplanned native-adapter command: '+repr(command))
                    capture(path,command,events);self.logs.append(path)
                    return subprocess.CompletedProcess(command,0,'','')
            def built(manifest,directory,runner,profile,**options):
                directory.mkdir(parents=True)
                (directory/'native-selection.json').write_text('{"scope":"selected-packages"}')
                (directory/'standalone.json').write_text('[]')
                return []
            with patch('run_lane.build',side_effect=built), \
                    patch('run_lane.coverage',return_value=(discovered,{'scale':[]},{})), \
                    patch('run_lane.list_tests',return_value=discovered), \
                    patch('run_lane.nextest_configuration_source',return_value=config), \
                    patch('run_lane.collect',return_value={'manifest':'Cargo.toml','source_sites':[],
                        'source_counts':{},'lanes':{},'llvm_lines':{},'coverage':{'status':'partial'}}):
                # This adapter supplies no compiler-unit evidence. The real
                # budget must still refuse rather than count the fake build as0.
                with self.assertRaises(LaneExecutionFailure) as refused:
                    execute(root,'root',['ordinary'],output,cargo_runner=Runner(),
                            recommendation_directory=output/'run-local-recommendations')
            self.assertIn('unmeasured-build',str(refused.exception))
            result=refused.exception.result
            self.assertEqual(result['executed_coverage']['status'],'partial')
            self.assertEqual(before,{p.name:p.read_bytes() for p in committed.iterdir()})
            self.assertEqual(git('status','--porcelain'),'')
            self.assertEqual(load_slots(output/'run-local-recommendations'),{'court\ncase':1})

    def test_run_local_destination_does_not_change_the_committed_scheduler_read(self):
        class ObservedPolicy(Exception): pass
        with TemporaryDirectory() as temporary:
            root=Path(temporary).resolve();output=root/'target/run';output.mkdir(parents=True)
            (root/'.config').mkdir();config=root/'.config/nextest.toml';config.write_text('[profile.ci]\n')
            def built(manifest,directory,runner,profile,**options):
                directory.mkdir(parents=True)
                (directory/'native-selection.json').write_text('{"scope":"selected-packages"}')
                (directory/'standalone.json').write_text('[]')
                return []
            discovered={'rust-suites':{'court':{'testcases':{'case':{'ignored':False}}}}}
            def observe(original,generated,profile,previous,wrapper):
                self.assertEqual(previous,{'court\ncase':3})
                raise ObservedPolicy()
            def no_command(*args,**kwargs):raise AssertionError('no native command expected')
            runner=SimpleNamespace(environment={},__call__=no_command)
            with patch('run_lane.build',side_effect=built), \
                    patch('run_lane.coverage',return_value=(discovered,{'scale':[]},{})), \
                    patch('run_lane.nextest_configuration_source',return_value=config), \
                    patch('run_lane.load_slots',return_value={'court\ncase':3}) as load, \
                    patch('run_lane.configuration',side_effect=observe):
                with self.assertRaises(ObservedPolicy):
                    execute(root,'root',[],output,cargo_runner=runner,
                            recommendation_directory=output/'run-local-recommendations')
            load.assert_called_once_with(root/'scripts/test_cost/slots/root',known={'court\ncase'})

    def test_run_local_output_keeps_committed_policy_and_legacy_still_accumulates(self):
        with TemporaryDirectory() as temporary:
            root=Path(temporary);output=root/'target/current';output.mkdir(parents=True)
            def git(*args):
                return subprocess.check_output(['git',*args],cwd=root).decode().strip()
            git('init','-q');git('config','user.name','retention court')
            git('config','user.email','retention@example.invalid')
            (root/'.gitignore').write_text('target/\n');git('add','.');git('commit','-qm','source')
            def capture(name,cpu):
                path=output/name
                event=dict(type='test-cost-cpu',binary='court',test='case',cpu_seconds=cpu,
                           wall_seconds=2,reserved_slots=8,exit=0,accounting='complete')
                path.write_text('Command: ["python", "phase_telemetry.py"]\n'+
                    f'Working directory: {root}\nHEAD: {git("rev-parse","HEAD")}\n'+
                    'Git status --short empty: True\nStart: 2026-01-01T00:00:00+00:00\n'+
                    'End: 2026-01-01T00:00:02+00:00\nExit code: 0\n'+json.dumps(event))
                return path
            committed=root/'scripts/test_cost/slots/root';known={'court\ncase'}
            retain_recommendations(output,committed,capture('seed.log',6),git('rev-parse','HEAD'),{},known)
            git('add','.');git('commit','-qm','existing cumulative scheduling input')
            before={p.name:p.read_bytes() for p in committed.iterdir()}
            local=output/'run-local-recommendations'
            retain_recommendations(output,local,capture('local.log',1),git('rev-parse','HEAD'),{},known)
            self.assertEqual(before,{p.name:p.read_bytes() for p in committed.iterdir()})
            self.assertEqual(git('status','--porcelain'), '')
            self.assertEqual(load_slots(committed),{'court\ncase':3})
            self.assertEqual(load_slots(local),{'court\ncase':1})
            self.assertEqual(json.loads((local/'index.json').read_text())['sample_count'],1)
            self.assertEqual(json.loads((output/'recommendations.json').read_text()),{'court\ncase':1})
            retain_recommendations(output,committed,capture('legacy.log',1),git('rev-parse','HEAD'),{},known)
            self.assertNotEqual(git('status','--porcelain'), '')
            self.assertEqual(load_slots(committed),{'court\ncase':3})
            self.assertEqual(json.loads((committed/'index.json').read_text())['sample_count'],2)

    def test_run_local_destination_cannot_escape_its_output_before_native_effects(self):
        with TemporaryDirectory() as temporary:
            root=Path(temporary);output=root/'target/run'
            for destination in (root/'scripts/test_cost/slots/root',output,output/'../foreign'):
                with self.subTest(destination=destination),patch('run_lane.build') as build:
                    with self.assertRaisesRegex(ValueError,'inside this run output'):
                        execute(root,'root',[],output,recommendation_directory=destination)
                    build.assert_not_called()
            self.assertFalse(output.exists())

    def test_focused_runner_checks_dependency_testkit_sources_before_compiling_any_target(self):
        with TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            names = ['owner', 'owner-testkit', 'unrelated-court']
            (root/'Cargo.toml').write_text('[workspace]\nmembers=["*"]\n')
            for name in names:
                package = root/name
                (package/'src').mkdir(parents=True)
                (package/'Cargo.toml').write_text(f'[package]\nname="{name}"\nversion="0.1.0"\n')
                (package/'src/lib.rs').write_text('' if name == 'owner' else
                    '#[test] fn bad() { std::process::Command::new("cargo"); }')
            metadata = {'workspace_members': names, 'packages': [
                {'id': name, 'name': name, 'targets': [{'name': name, 'kind': ['lib']}]} for name in names],
                'resolve': {'nodes': [{'id': name, 'dependencies': ['owner-testkit'] if name == 'owner' else []}
                                      for name in names]}}
            class Runner:
                environment = {}
                commands = []
                def __call__(self, command, cwd, log):
                    self.commands.append(command)
                    return subprocess.CompletedProcess(command, 0, json.dumps(metadata), '')
            runner = Runner()
            with patch('run_lane.child_environment', return_value={}):
                with self.assertRaisesRegex(ValueError, 'tests invoke compilers') as denied:
                    execute(root, 'root', ['owner-unit'], root/'out', cargo_runner=runner,
                            native_selection={'packages': ['owner'], 'targets': {'lib': True}})
            self.assertIn('owner-testkit', str(denied.exception))
            self.assertNotIn('unrelated-court', str(denied.exception))
            self.assertEqual([command[1] for command in runner.commands], ['metadata'])

    def test_rust_plan_identities_have_windows_safe_log_paths(self):
        self.assertEqual(log_label('scenario::nextest'), 'scenario--nextest')
        self.assertLessEqual(len(log_label('smoke::' + 'domain' * 100)), 80)

    def test_scheduled_scale_is_never_an_executable_landing_lane(self):
        self.assertNotIn('scale', LANDING)
        with self.assertRaises(ValueError):
            selected_lanes('scale')

    def test_repeated_partition_is_refused_before_execution(self):
        with self.assertRaises(ValueError):
            selected_lanes('process,process')

    def test_environment_keeps_every_core_and_uses_committed_compiler_profiles(self):
        with patch.dict(os.environ, {'CARGO_BUILD_JOBS': '24'}, clear=True):
            environment = child_environment()
        self.assertEqual(environment['CARGO_BUILD_JOBS'], '24')
        self.assertFalse(any(key.startswith('CARGO_PROFILE_') for key in environment))
        self.assertNotIn('CARGO_INCREMENTAL', environment)
        self.assertNotIn('RUSTUP_TOOLCHAIN', environment)
        self.assertEqual(environment['WORTH_IMPLEMENTATION_BUILD_ID'],
                         '00000000-0000-4000-8000-000000000001')

    def test_supplied_gate_profile_overrides_are_refused_before_source_discovery_or_build(self):
        with TemporaryDirectory() as temporary:
            output = Path(temporary)/'unused'
            runner = SimpleNamespace(environment={'CARGO_PROFILE_TEST_DEBUG': '0'})
            with patch('run_lane.collect') as discovery, patch('run_lane.build') as build:
                with self.assertRaises(LaneEnvironmentRefusal):
                    execute(Path.cwd(), 'store', [], output, cargo_runner=runner)
            discovery.assert_not_called()
            build.assert_not_called()
            self.assertFalse(output.exists())


if __name__ == '__main__':
    unittest.main()
