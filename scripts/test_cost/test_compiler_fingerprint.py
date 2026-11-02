import json
from pathlib import Path
import tempfile
import unittest

from compiler_units import attribute, declarations, CompilerUnitRefusal
from test_compiler_units import event, graph, unit


class CompilerDependencyFingerprintTests(unittest.TestCase):
    def test_real_dependency_fingerprint_distinguishes_equal_parent_features(self):
        parent = unit('sha2', mode='build', dependencies=[2])
        other = unit('sha2', mode='build', dependencies=[3])
        source = graph(unit('root', dependencies=[1, 4]), parent,
                       unit('digest', mode='build'),
                       unit('digest', mode='build', features=['mac']), other)
        rows = declarations(source)
        declaration = {'units': rows, 'compiler_inputs': 'flags'}
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory) / 'debug'
            own = base / '.fingerprint/sha2-observed/lib-sha2.json'
            child = base / '.fingerprint/digest-real/lib-digest.json'
            own.parent.mkdir(parents=True)
            child.parent.mkdir(parents=True)
            own.write_text(json.dumps({'features': '[]', 'deps': [[0, 'digest', False, 42]]}))
            child.write_text(json.dumps({'features': '["mac"]', 'deps': []}))
            child.with_suffix('').write_text((42).to_bytes(8, 'little').hex())
            observed = event(parent, linked=True)
            observed['filenames'] = [str(base / 'deps/libsha2-observed.rlib')]
            self.assertEqual(attribute(observed, declaration)['identity'], rows[4]['identity'])
            # A different actual dependency selects the other real declaration,
            # independently of the parent artifact's path/hash.
            child.write_text(json.dumps({'features': '[]', 'deps': []}))
            self.assertEqual(attribute(observed, declaration)['identity'], rows[1]['identity'])
            child.with_suffix('').write_text((43).to_bytes(8, 'little').hex())
            with self.assertRaises(CompilerUnitRefusal):
                attribute(observed, declaration)

    def test_dependency_name_does_not_choose_between_different_package_versions(self):
        first = unit('sha2', mode='build', dependencies=[2])
        other = unit('sha2', mode='build', dependencies=[3])
        digest = unit('digest', mode='build')
        different = unit('digest', mode='build', features=['mac'])
        different['pkg_id'] = 'registry+https://example.invalid#index#digest@0.2.0'
        rows = declarations(graph(unit('root', dependencies=[1, 4]), first,
                                  digest, different, other))
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory) / 'debug'
            own = base / '.fingerprint/sha2-observed/lib-sha2.json'
            child = base / '.fingerprint/digest-real/lib-digest.json'
            own.parent.mkdir(parents=True)
            child.parent.mkdir(parents=True)
            own.write_text(json.dumps({'features': '[]', 'deps': [[0, 'digest', False, 42]]}))
            child.write_text(json.dumps({'features': '["mac"]', 'deps': []}))
            child.with_suffix('').write_text((42).to_bytes(8, 'little').hex())
            observed = event(first, linked=True)
            observed['filenames'] = [str(base / 'deps/libsha2-observed.rlib')]
            with self.assertRaises(CompilerUnitRefusal):
                attribute(observed, {'units': rows, 'compiler_inputs': 'flags'})

    def test_profile_only_ambiguity_cannot_be_guessed_from_an_artifact_suffix(self):
        first = unit('owner', mode='build')
        second = unit('owner', mode='build')
        second['profile']['panic'] = 'abort'
        rows = declarations(graph(unit('root', dependencies=[1, 2]), first, second))
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory) / 'debug'
            own = base / '.fingerprint/owner-observed/lib-owner.json'
            own.parent.mkdir(parents=True)
            own.write_text(json.dumps({'features': '[]', 'deps': []}))
            observed = event(first, linked=True)
            observed['filenames'] = [str(base / 'deps/libowner-observed.rlib')]
            with self.assertRaises(CompilerUnitRefusal):
                attribute(observed, {'units': rows, 'compiler_inputs': 'flags'})


if __name__ == '__main__':
    unittest.main()
