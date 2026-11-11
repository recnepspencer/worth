import json
from pathlib import Path
import tempfile
import unittest

from cargo_dependency_features import retain_dependencies
from compiler_units import CompilerUnitRefusal, attribute, declarations
from test_compiler_units import event, graph, unit


class CargoDependencyFeaturesTests(unittest.TestCase):
    def fixture(self, root, mode='build', distinct_package=False):
        owner = unit('owner', mode=mode, dependencies=[3])
        other = unit('owner', mode=mode, dependencies=[4])
        other_dependency = unit('digest', features=['mac', 'std'])
        if distinct_package:
            other_dependency['pkg_id'] = 'registry+https://example.invalid#index#digest@0.2.0'
        rows = declarations(graph(unit('root', dependencies=[1, 2]), owner, other,
                                  unit('digest', features=['std']),
                                  other_dependency))
        declaration = {'units': rows, 'compiler_inputs': 'flags'}
        observed = dict(event(owner, linked=True), reason='compiler-artifact', fresh=False)
        observed['filenames'] = [str(root / 'debug/deps/libowner-first.rlib')]
        fingerprints = root / 'debug/.fingerprint'
        selected = fingerprints / 'owner-first/lib-owner.json'
        selected.parent.mkdir(parents=True)
        selected.write_text(json.dumps({'deps': [[1, 'digest', False, 17]]}))
        dependency = fingerprints / 'digest-selected/lib-digest'
        dependency.parent.mkdir()
        dependency.write_text((17).to_bytes(8, 'little').hex())
        dependency.with_suffix('.json').write_text(json.dumps({'features': '["std"]'}))
        unrelated = fingerprints / 'digest-other/lib-digest'
        unrelated.parent.mkdir()
        unrelated.write_text((18).to_bytes(8, 'little').hex())
        unrelated.with_suffix('.json').write_text(json.dumps({'features': '["mac", "std"]'}))
        return observed, declaration, dependency

    def test_actual_dependency_features_distinguish_same_profile_artifacts(self):
        with tempfile.TemporaryDirectory() as temporary:
            observed, declaration, _ = self.fixture(Path(temporary))
            with self.assertRaises(CompilerUnitRefusal):
                attribute(observed, declaration)
            text = retain_dependencies(json.dumps(observed) + '\n', declaration)
            enriched = json.loads(text)
            self.assertEqual(enriched['cargo_dependency_features'], [['digest', ['std']]])
            result = attribute(enriched, declaration)
            self.assertEqual(result['dependency_units'], declaration['units'][1]['dependency_units'])
            # The captured observation remains sufficient after build files disappear.
            self.assertEqual(result, attribute(json.loads(text), declaration))

    def test_missing_or_unmatched_dependency_evidence_still_refuses(self):
        with tempfile.TemporaryDirectory() as temporary:
            observed, declaration, dependency = self.fixture(Path(temporary))
            dependency.unlink()
            enriched = json.loads(retain_dependencies(json.dumps(observed), declaration))
            self.assertNotIn('cargo_dependency_features', enriched)
            with self.assertRaises(CompilerUnitRefusal):
                attribute(enriched, declaration)
            observed['cargo_dependency_features'] = [['digest', ['foreign']]]
            with self.assertRaises(CompilerUnitRefusal):
                attribute(observed, declaration)

    def test_metadata_only_artifact_retains_actual_dependency_features(self):
        with tempfile.TemporaryDirectory() as temporary:
            observed, declaration, _ = self.fixture(Path(temporary), mode='check')
            observed['filenames'][0] = observed['filenames'][0].replace('.rlib', '.rmeta')
            enriched = json.loads(retain_dependencies(json.dumps(observed), declaration))
            self.assertEqual(enriched['cargo_dependency_features'], [['digest', ['std']]])
            self.assertEqual(attribute(enriched, declaration)['dependency_units'],
                             declaration['units'][1]['dependency_units'])

    def test_retained_dependency_features_do_not_choose_between_package_versions(self):
        with tempfile.TemporaryDirectory() as temporary:
            observed, declaration, _ = self.fixture(Path(temporary), distinct_package=True)
            observed['cargo_dependency_features'] = [['digest', ['std']]]
            with self.assertRaises(CompilerUnitRefusal):
                attribute(observed, declaration)

    def test_incomplete_fingerprint_refuses_without_inventing_compiler_identity(self):
        with tempfile.TemporaryDirectory() as temporary:
            observed, declaration, _ = self.fixture(Path(temporary))
            own = Path(temporary) / 'debug/.fingerprint/owner-first/lib-owner.json'
            for record in ({'deps': []}, {'features': '{}'}, {'features': '[]'}):
                with self.subTest(record=record):
                    own.write_text(json.dumps(record))
                    with self.assertRaises(CompilerUnitRefusal):
                        attribute(observed, declaration)


if __name__ == '__main__':
    unittest.main()
