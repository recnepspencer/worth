import copy
import hashlib
import json
from pathlib import Path
import tempfile
import unittest

from llvm_artifacts import attributed, ir_file


class LlvmArtifactAttributionTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.deps = self.root / 'target/debug/deps'
        self.deps.mkdir(parents=True)
        self.manifest = self.root / 'Cargo.toml'
        self.source = self.root / 'src/lib.rs'
        self.wanted = [dict(package='worth-testkit-world', target='lib', family='lib',
                            cargo_target='worth_testkit_world', manifest=str(self.manifest),
                            source=str(self.source))]

    def artifact(self, *, test=False, digest='123456789abcdef0', fresh=False):
        stem = 'worth_testkit_world-' + digest
        path = self.deps / (stem + '.exe' if test else 'lib' + stem + '.rlib')
        path.write_bytes(b'linked artifact')
        (self.deps / (stem + '.ll')).write_bytes(b'define void @build() {}\n')
        return {'reason': 'compiler-artifact', 'manifest_path': str(self.manifest),
                'package_id': 'path+file:///fixture#worth-testkit-world@0.0.0',
                'target': {'name': 'worth_testkit_world', 'kind': ['lib'], 'src_path': str(self.source)},
                'profile': {'test': test, 'opt_level': '0', 'debug_assertions': True},
                'features': ['owned-profile'], 'fresh': fresh, 'filenames': [str(path)],
                'executable': str(path) if test else None}

    def test_normal_and_harness_are_distinct_actual_units_with_ir_provenance(self):
        normal = self.artifact()
        harness = self.artifact(test=True, digest='fedcba9876543210')
        result = attributed('\n'.join(map(json.dumps, [normal, harness])), self.wanted)
        self.assertEqual(set(result), {'worth-testkit-world-lib', 'worth-testkit-world-lib-test-harness'})
        actual = result['worth-testkit-world-lib']
        self.assertEqual(actual['profile'], normal['profile'])
        self.assertEqual(actual['features'], ['owned-profile'])
        self.assertEqual(actual['target']['src_path'], str(self.source))
        self.assertEqual(actual['ir']['sha256'], hashlib.sha256(b'define void @build() {}\n').hexdigest())
        self.assertEqual(actual['ir']['bytes'], len(b'define void @build() {}\n'))

    def test_same_named_dependency_and_wrong_source_cannot_satisfy_a_unit(self):
        for field in ('manifest_path', 'source'):
            event = self.artifact()
            if field == 'source':
                event['target']['src_path'] = str(self.root / 'other/lib.rs')
            else:
                event[field] = str(self.root / 'other/Cargo.toml')
            with self.subTest(field=field), self.assertRaisesRegex(ValueError, 'did not emit required'):
                attributed(json.dumps(event), self.wanted)

    def test_right_source_with_wrong_kind_is_refused(self):
        event = self.artifact()
        event['target']['kind'] = ['custom-build']
        with self.assertRaisesRegex(ValueError, 'wrong Cargo target kind'):
            attributed(json.dumps(event), self.wanted)

    def test_cached_ir_must_exist_and_old_codegen_partitions_do_not_substitute(self):
        event = self.artifact(fresh=True)
        path = Path(ir_file(event)['path'])
        path.unlink()
        path.with_suffix('.0.rcgu.ll').write_text('stale partition')
        with self.assertRaisesRegex(ValueError, 'no exact default LLVM IR'):
            attributed(json.dumps(event), self.wanted)

    def test_cached_default_ir_is_valid_only_with_actual_linked_output(self):
        event = self.artifact(fresh=True)
        self.assertTrue(attributed(json.dumps(event), self.wanted)['worth-testkit-world-lib']['fresh'])
        Path(event['filenames'][0]).unlink()
        with self.assertRaisesRegex(ValueError, 'linked artifact is missing'):
            ir_file(event)

    def test_two_output_hashes_refuse_ambiguous_attribution(self):
        event = self.artifact()
        other = self.artifact(digest='fedcba9876543210')
        event['filenames'] += other['filenames']
        with self.assertRaisesRegex(ValueError, 'no unique hashed linked output'):
            ir_file(event)

    def test_matching_metadata_only_artifact_does_not_prove_retained_linking(self):
        event = self.artifact()
        metadata = Path(event['filenames'][0]).with_suffix('.rmeta')
        metadata.write_bytes(b'metadata')
        event['filenames'] = [str(metadata)]
        with self.assertRaisesRegex(ValueError, 'no unique hashed linked output'):
            ir_file(event)

    def test_linked_metadata_pair_maps_one_exact_ir_file(self):
        event = self.artifact()
        metadata = Path(event['filenames'][0]).with_suffix('.rmeta')
        event['filenames'].append(str(metadata))
        self.assertEqual(Path(ir_file(event)['path']).name, 'worth_testkit_world-123456789abcdef0.ll')

    def test_duplicate_cargo_events_are_refused_instead_of_double_counted(self):
        event = self.artifact()
        with self.assertRaisesRegex(ValueError, 'duplicate measured Cargo unit'):
            attributed(json.dumps(event) + '\n' + json.dumps(event), self.wanted)

    def test_missing_actual_profile_features_or_source_never_gets_defaults(self):
        original = self.artifact()
        for field in ('profile', 'features', 'target', 'fresh', 'package_id'):
            event = copy.deepcopy(original)
            del event[field]
            with self.subTest(field=field), self.assertRaises(ValueError):
                attributed(json.dumps(event), self.wanted)

    def test_unrelated_artifacts_are_not_measured(self):
        event = self.artifact()
        dependency = copy.deepcopy(event)
        dependency['manifest_path'] = str(self.root / 'dependency/Cargo.toml')
        result = attributed(json.dumps(dependency) + '\n' + json.dumps(event), self.wanted)
        self.assertEqual(len(result), 1)


if __name__ == '__main__':
    unittest.main()
