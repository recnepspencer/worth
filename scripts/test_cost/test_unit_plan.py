import json
from pathlib import Path
import tempfile
import unittest

from compiler_units import CompilerUnitRefusal, attribute
from test_compiler_units import event, graph, unit
from unit_plan import command_for, declare, verify


class RequiredUnitPlanTests(unittest.TestCase):
    def test_all_targets_checked_harness_cannot_be_charged_as_its_normal_library(self):
        library = unit('owner', dependencies=[3])
        source = graph(unit('court', kind='test', dependencies=[1]), library, library, unit('dependency'))
        source['roots'] = [0, 1, 2]
        with tempfile.TemporaryDirectory() as temporary:
            for subcommand in ('check', 'clippy'):
                plan = declare(['cargo', subcommand, '--all-targets'], temporary, {}, json.dumps(source))
                ordinary = attribute(event(library), plan)
                checked = event(library)
                checked['profile']['test'] = True
                harness = attribute(checked, plan)
                self.assertNotEqual(ordinary['identity'], harness['identity'])
                self.assertFalse(ordinary['test_harness'])
                self.assertTrue(harness['test_harness'])

    def test_standalone_identical_checked_stages_preserve_both_compiler_obligations(self):
        library = unit('owner', dependencies=[2])
        source = graph(library, library, unit('dependency'))
        source['roots'] = [0, 1]
        with tempfile.TemporaryDirectory() as temporary:
            plan = declare(['cargo', 'clippy', '--all-targets'], temporary, {}, json.dumps(source))
            stages = [row for row in plan['units'] if row['target']['name'] == 'owner']
            self.assertEqual({row['test_harness'] for row in stages}, {False, True})
            self.assertEqual(len({row['identity'] for row in stages}), 2)

    def test_explicit_checked_tests_distinguish_library_harness_from_dependency(self):
        library = unit('owner', dependencies=[3])
        observed_graph = graph(unit('court', kind='test', dependencies=[1]),
                               library, library, unit('dependency'))
        observed_graph['roots'] = [0, 2]
        with tempfile.TemporaryDirectory() as temporary:
            declaration = declare(['cargo', 'check', '--tests'], temporary, {}, json.dumps(observed_graph))
            ordinary = attribute(event(library), declaration)
            checked_test = event(library)
            checked_test['profile']['test'] = True
            harness = attribute(checked_test, declaration)
            self.assertFalse(ordinary['test_harness'])
            self.assertTrue(harness['test_harness'])
            self.assertNotEqual(ordinary['identity'], harness['identity'])
            unknown = declare(['cargo', 'check'], temporary, {}, json.dumps(observed_graph))
            with self.assertRaises(CompilerUnitRefusal):
                attribute(checked_test, unknown)

    def test_nextest_build_declares_the_same_cargo_targets_profile_and_features(self):
        command = ['cargo', 'nextest', 'list', '--cargo-message-format=json', '--workspace',
                   '--all-targets', '--cargo-profile', 'ci-test', '--manifest-path', 'Cargo.toml',
                   '--features', 'owner/cert', '--list-type', 'binaries-only', '--message-format', 'json']
        self.assertEqual(command_for(command), ['cargo', 'test', '--no-run', '--workspace',
                         '--all-targets', '--profile', 'ci-test', '--manifest-path', 'Cargo.toml',
                         '--features', 'owner/cert', '--unit-graph', '-Z', 'unstable-options'])

    def test_prebuilt_execution_and_metadata_have_no_new_graph(self):
        self.assertIsNone(command_for(['cargo', 'metadata']))
        self.assertIsNone(command_for(['cargo', 'nextest', 'run', '--binaries-metadata', 'built.json']))
        self.assertIsNone(command_for(['cargo', 'nextest', 'run', '--binaries-metadata=built.json']))

    def test_rustc_flags_are_after_the_unit_graph_options(self):
        command = ['cargo', 'rustc', '--lib', '--', '--emit=llvm-ir=observed.ll']
        self.assertEqual(command_for(command), ['cargo', 'rustc', '--lib', '--unit-graph', '-Z',
                                                'unstable-options', '--', '--emit=llvm-ir=observed.ll'])

    def test_timing_telemetry_is_removed_only_from_the_noncompiling_declaration(self):
        for timing in ('--timings', '--timings=html'):
            command = ['cargo', 'test', '--message-format=json', '--no-run', timing,
                       '--features', 'owner/cert', '--profile', 'ci-test']
            before = list(command)
            declared = command_for(command)
            self.assertEqual(command, before)
            self.assertNotIn(timing, declared)
            self.assertEqual(declared, [value for value in command if value != timing] +
                             ['--unit-graph', '-Z', 'unstable-options'])
        command = ['cargo', 'rustc', '--timings', '--', '--cfg', 'timings_observer']
        self.assertEqual(command_for(command), ['cargo', 'rustc', '--unit-graph', '-Z',
                                                'unstable-options', '--', '--cfg', 'timings_observer'])

    def test_extra_json_records_remain_invalid_unit_graph_framing(self):
        with tempfile.TemporaryDirectory() as temporary:
            observed = json.dumps(graph(unit('owner')))
            for extra in (json.dumps({'reason': 'build-started', 'run_id': 'observed'})+'\n'+observed,
                          observed+'\n'+observed):
                with self.subTest(extra=extra[:80]), self.assertRaises(CompilerUnitRefusal):
                    declare(['cargo', 'check'], temporary, {}, extra)

    def test_command_feature_profile_or_flag_drift_after_declaration_is_refused(self):
        with tempfile.TemporaryDirectory() as temporary:
            command = ['cargo', 'check', '--profile', 'dev', '--features', 'cert']
            environment = {'RUSTFLAGS': '-C overflow-checks=on'}
            plan = declare(command, temporary, environment, json.dumps(graph(unit('owner'))))
            verify(plan, command, temporary, environment)
            for changed in (command + ['--features', 'other'],
                            ['cargo', 'check', '--profile', 'release'], command + ['--config', 'build.rustflags=[]']):
                with self.subTest(changed=changed), self.assertRaises(CompilerUnitRefusal):
                    verify(plan, changed, temporary, environment)
            with self.assertRaises(CompilerUnitRefusal):
                verify(plan, command, temporary, {'RUSTFLAGS': '-C overflow-checks=off'})

    def test_config_content_drift_is_refused_before_compilation(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / '.cargo').mkdir()
            config = root / '.cargo/config.toml'
            config.write_text('[build]\nrustflags=[]\n')
            command = ['cargo', 'check']
            plan = declare(command, root, {}, json.dumps(graph(unit('owner'))))
            config.write_text('[build]\nrustflags=["--cfg", "drift"]\n')
            with self.assertRaises(CompilerUnitRefusal):
                verify(plan, command, root, {})

    def test_cargo_build_and_rustdoc_flag_overrides_cannot_drift_after_declaration(self):
        with tempfile.TemporaryDirectory() as temporary:
            command = ['cargo', 'check']
            plan = declare(command, temporary, {}, json.dumps(graph(unit('owner'))))
            for name in ('CARGO_BUILD_RUSTFLAGS', 'CARGO_ENCODED_RUSTDOCFLAGS', 'CARGO_INCREMENTAL'):
                with self.subTest(name=name), self.assertRaises(CompilerUnitRefusal):
                    verify(plan, command, temporary, {name: 'changed'})

    def test_file_valued_cli_config_content_cannot_drift_after_declaration(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            extra = root/'extra.toml'
            extra.write_text('[build]\nrustflags=[]\n')
            for flag in (['--config', 'extra.toml'], ['--config=' + str(extra)]):
                extra.write_text('[build]\nrustflags=[]\n')
                command = ['cargo', 'check', *flag]
                plan = declare(command, root, {}, json.dumps(graph(unit('owner'))))
                self.assertIn(str(extra.resolve()), plan['contract']['compiler_inputs']['config_files'])
                extra.write_text('[build]\nrustflags=["--cfg", "unplanned"]\n')
                with self.subTest(flag=flag), self.assertRaises(CompilerUnitRefusal):
                    verify(plan, command, root, {})

    def test_local_dependency_manifest_or_workspace_lock_drift_refuses_before_compilation(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            dependency = root/'dependency'
            dependency.mkdir()
            (root/'Cargo.toml').write_text('[workspace]\nmembers=["dependency"]\n')
            manifest = dependency/'Cargo.toml'
            manifest.write_text('[package]\nname="owner"\nversion="0.1.0"\n')
            lock = root/'Cargo.lock'
            lock.write_text('version=4\n')
            required = unit('owner')
            required['pkg_id'] = 'path+' + dependency.as_uri() + '#0.1.0'
            command = ['cargo', 'check', '--manifest-path', str(root/'Cargo.toml')]
            plan = declare(command, root, {}, json.dumps(graph(required)))
            self.assertIn(str(manifest), plan['contract']['manifests'])
            self.assertIn(str(lock), plan['contract']['manifests'])
            for path in (manifest, lock):
                original = path.read_text()
                path.write_text(original + '\n# changed dependency graph\n')
                with self.subTest(path=path), self.assertRaises(CompilerUnitRefusal):
                    verify(plan, command, root, {})
                path.write_text(original)
            verify(plan, command, root, {})

    def test_unsupported_graph_cannot_become_an_accepted_zero_compilation(self):
        with tempfile.TemporaryDirectory() as temporary:
            with self.assertRaises(CompilerUnitRefusal):
                declare(['cargo', 'check'], temporary, {}, '{}')


if __name__ == '__main__':
    unittest.main()
