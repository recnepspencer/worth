import json
import hashlib
from pathlib import Path
from types import SimpleNamespace
import tempfile
import tomllib
import unittest

from store_prebuild import ExecutablePrebuildFailure, REQUESTS, executable_features, prepare, retain
from filesystem_path import command_path, filesystem_path
from native_features import NativeFeatureGraph, NativeFeatureRefusal, library_identity, signature


class StoreExecutablePrebuildTests(unittest.TestCase):
    def test_executable_prerequisite_features_belong_to_actual_declaring_packages(self):
        workspace = Path(__file__).resolve().parents[2] / 'workspaces/worth-store/crates'
        for request in REQUESTS:
            for qualified in request['features']:
                package, feature = qualified.split('/', 1)
                manifest = workspace / package / 'Cargo.toml'
                with self.subTest(binary=request['binary'], feature=qualified):
                    declared = tomllib.loads(manifest.read_text(encoding='utf-8'))
                    self.assertEqual(declared['package']['name'], package)
                    self.assertIn(feature, declared.get('features', {}))

    def fixture(self, root):
        events = []
        for request in REQUESTS:
            binary = root / (request['binary'] + '.exe')
            binary.write_bytes(b'independent product')
            events.append(dict(reason='compiler-artifact', package_id='path+file:///owner#' + request['package'] + '@1',
                               target={'name': request['binary'], 'kind': ['bin']},
                               profile={'test': False}, features=[], executable=str(binary)))
        library = dict(reason='compiler-artifact', package_id='authority', target={'name': 'authority', 'kind': ['lib']},
                       profile={'test': False}, features=['observed'], fresh=True, filenames=['authority-native.rlib'])
        owners = sorted({request['package'] for request in REQUESTS})
        metadata = {'packages': [{'id': owner, 'name': owner} for owner in [*owners, 'authority', 'unrelated']],
                    'resolve': {'nodes': [{'id': owner, 'deps': [{'pkg': 'authority', 'dep_kinds': [{'kind': None}]}]} for owner in owners]
                                + [{'id': 'authority', 'deps': []}, {'id': 'unrelated', 'deps': []}]}}
        source = root / 'cargo-metadata.json'
        source.write_text(json.dumps(metadata))
        inputs = ({'path': str(source), 'sha256': hashlib.sha256(source.read_bytes()).hexdigest()},)
        native = NativeFeatureGraph(root / 'workspaces/worth-store', ('authority/observed', 'unrelated/unused'),
                                    {library_identity(library): {signature(library)}}, inputs)
        return native, [*events, library]

    def test_only_independently_compiled_binary_can_be_retained(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            binary = root / 'observer.exe'
            binary.write_bytes(b'ordinary independently built executable')
            request = dict(package='owner', binary='observer', features=[])
            event = dict(reason='compiler-artifact', package_id='path+file:///owner#owner@1',
                         target={'name': 'observer', 'kind': ['bin']},
                         profile={'test': False}, features=[], executable=str(binary))
            env = retain([event], root / 'retained', [request])
            manifest = json.loads(Path(env['WORTH_PREBUILT_EXECUTABLES']).read_text())
            retained = Path(manifest['executables'][0]['path'])
            self.assertEqual(retained.read_bytes(), binary.read_bytes())
            self.assertNotEqual(retained, binary)
            with self.assertRaises(ExecutablePrebuildFailure):
                retain([dict(event, profile={'test': True})], root / 'retained', [request])

    def test_retained_executable_supports_long_output_paths_without_changing_its_identity(self):
        with tempfile.TemporaryDirectory(dir=filesystem_path(Path(tempfile.gettempdir()))) as temporary:
            root = command_path(Path(temporary))
            binary = root / 'observer.exe'
            binary.write_bytes(b'independent executable under a long retention path')
            request = dict(package='owner', binary='observer', features=[])
            event = dict(reason='compiler-artifact', package_id='path+file:///owner#owner@1',
                         target={'name': 'observer', 'kind': ['bin']},
                         profile={'test': False}, features=[], executable=str(binary))
            directory = root / ('retained-' + 'a' * 90) / ('nested-' + 'b' * 90)
            result = retain([event], directory, [request])
            manifest_path = Path(result['WORTH_PREBUILT_EXECUTABLES'])
            self.assertEqual(manifest_path, directory.resolve() / 'executables.json')
            manifest = json.loads(filesystem_path(manifest_path).read_text())
            retained = Path(manifest['executables'][0]['path'])
            self.assertGreater(len(str(retained)), 260)
            self.assertEqual(filesystem_path(retained).read_bytes(), binary.read_bytes())
            self.assertEqual(retained.parent, directory.resolve())

    def test_missing_binary_cannot_be_replaced_by_the_test_process(self):
        with tempfile.TemporaryDirectory() as temporary:
            with self.assertRaises(ExecutablePrebuildFailure):
                retain([], temporary, [dict(package='owner', binary='absent', features=[])])

    def test_product_build_uses_the_observed_native_graph_before_publishing_paths(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            native, events = self.fixture(root)
            calls = []
            def runner(command, cwd, log):
                calls.append(command)
                return SimpleNamespace(returncode=0, stdout='\n'.join(map(json.dumps, events)))
            result = prepare(root, root / 'output', runner, 'ci-test', native=native)
            self.assertEqual(calls[0][calls[0].index('--features') + 1], 'authority/observed')
            self.assertNotIn('--workspace', calls[0])
            self.assertNotIn('--bins', calls[0])
            packages = {calls[0][index + 1] for index, value in enumerate(calls[0]) if value == '--package'}
            binaries = {calls[0][index + 1] for index, value in enumerate(calls[0]) if value == '--bin'}
            self.assertEqual(packages, {request['package'] for request in REQUESTS})
            self.assertEqual(binaries, {request['binary'] for request in REQUESTS})
            self.assertTrue(Path(result['WORTH_PREBUILT_EXECUTABLES']).is_file())
            receipt = json.loads((root / 'output/native-feature-graph.json').read_text())
            self.assertEqual(receipt['validated_shared_libraries'], 1)

    def test_unrelated_native_member_features_do_not_expand_executable_roots(self):
        with tempfile.TemporaryDirectory() as temporary:
            native, _ = self.fixture(Path(temporary).resolve())
            self.assertEqual(executable_features(native), ('authority/observed',))

    def test_changed_native_metadata_cannot_choose_executable_feature_scope(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            native, _ = self.fixture(root)
            (root / 'cargo-metadata.json').write_text('{}')
            with self.assertRaisesRegex(NativeFeatureRefusal, 'changed after capture'):
                executable_features(native)

    def test_product_adapter_refuses_cached_library_graph_drift_before_path_publication(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            native, events = self.fixture(root)
            events[-1]['filenames'] = ['authority-other-graph.rlib']
            def runner(command, cwd, log):
                return SimpleNamespace(returncode=0, stdout='\n'.join(map(json.dumps, events)))
            with self.assertRaisesRegex(NativeFeatureRefusal, 'differs from native'):
                prepare(root, root / 'output', runner, 'ci-test', native=native)
            self.assertFalse((root / 'output/executables/executables.json').exists())


if __name__ == '__main__':
    unittest.main()
