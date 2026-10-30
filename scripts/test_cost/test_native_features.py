import copy
import json
from pathlib import Path
import tempfile
import unittest

from native_features import load, validate, NativeFeatureRefusal


class NativeFeatureGraphTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name).resolve()
        self.metadata = {'workspace_root': str(self.root), 'workspace_members': ['store', 'authority'],
                         'packages': [{'id': 'store', 'name': 'store', 'features': {'cert': []}},
                                      {'id': 'authority', 'name': 'authority', 'features': {'witness': []}},
                                      {'id': 'external', 'name': 'external', 'features': {'host': []}}]}
        self.events = [self.artifact('store', ['cert']), self.artifact('authority', ['witness']),
                       self.artifact('external', ['host'])]

    def artifact(self, package, features):
        return {'reason': 'compiler-artifact', 'package_id': package,
                'target': {'name': package, 'kind': ['lib']},
                'profile': {'test': False, 'opt_level': '0'}, 'features': features,
                'fresh': True, 'filenames': [str(self.root / (package + '-compiled.rlib'))]}

    def graph(self):
        (self.root / 'cargo-metadata.json').write_text(json.dumps(self.metadata))
        (self.root / 'binaries-build.log').write_text('\n'.join(map(json.dumps, self.events)))
        return load(self.root, self.root)

    def test_actual_workspace_features_are_carried_without_external_package_flags(self):
        graph = self.graph()
        self.assertEqual(graph.features, ('authority/witness', 'store/cert'))
        self.assertEqual(len(graph.inputs), 2)
        self.assertTrue(all(len(row['sha256']) == 64 for row in graph.inputs))

    def test_cached_identical_libraries_share_the_graph(self):
        self.assertEqual(validate(self.events, self.graph()), 3)

    def test_cached_different_libraries_refuse_even_when_no_compilation_occurs(self):
        graph = self.graph()
        for field, value in [('features', []), ('filenames', [str(self.root / 'other-graph.rlib')])]:
            changed = copy.deepcopy(self.events)
            changed[1][field] = value
            with self.subTest(field=field), self.assertRaisesRegex(NativeFeatureRefusal, 'differs from native'):
                validate(changed, graph)

    def test_missing_capture_and_foreign_workspace_refuse_before_product_build(self):
        with self.assertRaises(NativeFeatureRefusal):
            load(None, self.root)
        self.graph()
        with self.assertRaisesRegex(NativeFeatureRefusal, 'another workspace'):
            load(self.root, self.root / 'foreign')

    def test_multiple_native_feature_variants_are_not_silently_unioned(self):
        self.events.append(self.artifact('authority', []))
        with self.assertRaisesRegex(NativeFeatureRefusal, 'multiple feature variants'):
            self.graph()

    def test_missing_or_invented_compiler_feature_observations_refuse(self):
        self.events[1]['features'] = ['invented']
        with self.assertRaisesRegex(NativeFeatureRefusal, 'not declared'):
            self.graph()
        self.events = []
        with self.assertRaisesRegex(NativeFeatureRefusal, 'no compiler artifact'):
            self.graph()


if __name__ == '__main__':
    unittest.main()
