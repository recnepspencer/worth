import json
from pathlib import Path
import tempfile
import unittest

from cargo_dependency_features import retain_dependencies
from compiler_units import CompilerUnitRefusal, attribute, declarations
from test_compiler_units import event, graph, unit


class CargoDependencyFeaturesTests(unittest.TestCase):
    def fixture(self, root):
        owner = unit('owner', mode='build', dependencies=[3])
        other = unit('owner', mode='build', dependencies=[4])
        rows = declarations(graph(unit('root', dependencies=[1, 2]), owner, other,
                                  unit('digest', features=['std']),
                                  unit('digest', features=['mac', 'std'])))
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


if __name__ == '__main__':
    unittest.main()
