from pathlib import Path
import json
import tempfile
import unittest

from fixture_graph import configured_inputs, dependency_manifest, dependency_records, dependency_surface, dependency_targets, merge_dependencies, snapshot_audiences, snapshot_inputs


class FixtureGraphTests(unittest.TestCase):
    def test_relocated_fixture_keeps_the_complete_producer_audience_without_other_court_deps(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            (root / 'Cargo.toml').write_text('[workspace]\nmembers=["owner","court"]\n')
            owner, court = root / 'owner', root / 'court'
            owner.mkdir(); court.mkdir()
            (owner / 'Cargo.toml').write_text('[package]\nname="worth-owner"\nversion="0.1.0"\n'
                                              '[dev-dependencies]\nserde_json="1"\nunused-testkit="2"\n')
            (court / 'Cargo.toml').write_text('[package]\nname="worth-court-owner"\nversion="0.1.0"\n'
                                              '[dependencies]\nowner={package="worth-owner",path="../owner",features=["audience"]}\n'
                                              'ambient-backend="3"\n')
            source = court / 'tests/owner_court/sources/worth_owner/ui/denied.rs'
            source.parent.mkdir(parents=True)
            # Relocation preserves the producer's name; the court's alias does
            # not rename imports inside that original producer-owned source.
            source.write_text('fn denied(value: serde_json::Value) { worth_owner::accept(value); } fn main() {}')
            source.with_suffix('.stderr').write_text('error[E0308]: mismatched types')
            row = snapshot_inputs(root / 'Cargo.toml')[0]
            self.assertEqual(set(row['dependencies']), {'serde_json', 'unused-testkit', 'worth-owner'})
            self.assertEqual(row['dependencies']['worth-owner']['features'], ['audience'])
            self.assertEqual(row['dependencies']['serde_json']['version'], '1')

    def test_orphan_snapshot_is_reported_without_inventing_a_compiler_input(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            (root / 'tests').mkdir()
            (root / 'Cargo.toml').write_text('[package]\nname="fixture-court"\nversion="0.1.0"\n[workspace]\n')
            (root / 'tests/retired.stderr').write_text('error')
            orphaned = []
            self.assertEqual(snapshot_inputs(root / 'Cargo.toml', orphaned), [])
            self.assertEqual(len(orphaned), 1)

    def test_unreferenced_declared_dependencies_remain_part_of_the_caller_audience(self):
        # The session's audience-isolation decision retires the import-only
        # dependency claim: suggestions and negative imports require the complete
        # declared normal + dev surface, including unused names.
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            (root / 'tests/ui').mkdir(parents=True)
            (root / 'Cargo.toml').write_text('[package]\nname="fixture-court-runtime"\nversion="0.1.0"\n'
                                           '[workspace]\n[dependencies]\nowner="1"\nunused-testkit="2"\n')
            (root / 'tests/ui/denied.rs').write_text('use owner::Private; fn main() {}')
            (root / 'tests/ui/denied.stderr').write_text('error: private')
            rows = snapshot_inputs(root / 'Cargo.toml')
            self.assertEqual(set(rows[0]['dependencies']), {'owner', 'unused-testkit', 'fixture-court-runtime'})

    def test_normal_and_dev_declarations_union_the_same_dependency_features(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / 'Cargo.toml').write_text('[workspace]\n')
            manifest = root / 'owner.toml'
            manifest.write_text('[dependencies]\nowner={version="1",features=["normal"]}\n'
                                '[dev-dependencies]\nowner={version="1",features=["test"]}\n')
            self.assertEqual(dependency_surface(manifest, root / 'Cargo.toml')['owner']['features'],
                             ['normal', 'test'])

    def test_inherited_paths_and_features_are_resolved_at_their_owner(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            package = root / 'crates/court'
            package.mkdir(parents=True)
            (root / 'Cargo.toml').write_text('[workspace]\n[workspace.dependencies]\n'
                                           'owner={path="crates/owner",features=["a"]}\n')
            (package / 'Cargo.toml').write_text('[package]\nname="court"\nversion="0.1.0"\n'
                                              '[dependencies]\nowner={workspace=true,features=["b"]}')
            result = dependency_surface(package / 'Cargo.toml', root / 'Cargo.toml')
            self.assertEqual(result['owner']['features'], ['a', 'b'])
            self.assertEqual(result['owner']['path'], (root / 'crates/owner').as_posix())

    def test_conflicting_dependency_authority_is_refused_instead_of_aliased(self):
        with self.assertRaises(ValueError):
            merge_dependencies({'owner': {'path': '/first'}}, {'owner': {'path': '/second'}})

    def test_target_normal_and_dev_dependencies_keep_their_original_cargo_condition(self):
        import tomllib
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            manifest = root / 'Cargo.toml'
            manifest.write_text('[workspace]\n[workspace.dependencies]\nbackend={path="backend"}\n'
                                '[target.\'cfg(target_family = "wasm")\'.dependencies]\nbackend.workspace=true\n'
                                '[target.\'cfg(target_family = "wasm")\'.dev-dependencies]\n'
                                'backend={workspace=true,features=["test-authority"]}\n')
            ordinary = dependency_surface(manifest, manifest)
            targets = dependency_targets(manifest, manifest)
            rendered = tomllib.loads(dependency_manifest(ordinary, targets))
            self.assertNotIn('backend', rendered['dependencies'])
            spec = rendered['target']['cfg(target_family = "wasm")']['dependencies']['backend']
            self.assertEqual(spec['features'], ['test-authority'])
            self.assertEqual(spec['path'], (root / 'backend').as_posix())

    def test_configured_groups_keep_declared_dependencies_instead_of_environment_union(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            path = root / 'inputs.json'
            dependency = dict(name='backend', path='backend', features=['authority'], default_features=False)
            path.write_text(json.dumps({'schema': 2, 'environments': [
                {'identity': 'certification', 'profile': 'test', 'packages': [
                    {'audience': 'without', 'dependencies': [], 'sources': ['forbidden.rs']},
                    {'audience': 'with', 'dependencies': [dependency], 'sources': ['allowed.rs']}]}]}))
            row = configured_inputs(root, path)['certification']
            self.assertEqual(len(row['packages']), 2)
            allowed = next(package for package in row['packages'] if 'backend' in package['dependencies'])
            self.assertEqual(allowed['dependencies']['backend']['features'], ['authority'])
            self.assertFalse(allowed['dependencies']['backend']['default-features'])

    def test_explicit_production_snapshot_contract_does_not_borrow_producer_dev_features(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            path = root / 'inputs.json'
            path.write_text(json.dumps({'schema': 2, 'environments': [], 'snapshot_audiences': [{
                'source': 'denied.rs', 'environment': 'production', 'audience': 'ordinary-caller',
                'dependencies': [{'name': 'owner', 'path': 'owner', 'features': [], 'default_features': True}]}]}))
            row = snapshot_audiences(root, path)[root / 'denied.rs']
            self.assertEqual(row['environment'], 'production')
            self.assertEqual(row['dependencies'], {'owner': {'path': (root / 'owner').as_posix()}})

    def test_explicit_dependencies_preserve_versions_and_require_one_source(self):
        root = Path.cwd()
        row = dict(name='external', version='0.4', features=['bounded'], default_features=False)
        self.assertEqual(dependency_records(root, [row]), {
            'external': {'version': '0.4', 'features': ['bounded'], 'default-features': False}})
        for invalid in [dict(row, path='external'), {key: value for key, value in row.items() if key != 'version'}]:
            with self.subTest(row=invalid), self.assertRaisesRegex(ValueError, 'exactly one path or version'):
                dependency_records(root, [invalid])

    def test_semantic_chunk_view_input_keeps_its_complete_check_audience_once(self):
        from compiler_prebuild import environments
        root = Path(__file__).resolve().parents[2]
        workspace = root / 'workspaces/worth-store/Cargo.toml'
        producer = root / 'workspaces/worth-store/crates/worth-store'
        source = (root / 'workspaces/worth-store/crates/worth-store-court-store-replication-physical-record-journeys-physical-work/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/record_chunk_view_exposes_no_pool_authority.rs').resolve()
        rows, _ = environments(root, 'store')
        observed = [(row, package) for row in rows.values() for package in row['packages']
                    for fixture in package['sources'] if fixture == source]
        self.assertEqual(len(observed), 1)
        row, package = observed[0]
        self.assertEqual(row['identity'], 'certification')
        self.assertEqual(row['profile'], 'diagnostic-test')
        self.assertEqual(row['proof_stage'], 'check')
        expected = dependency_surface(producer / 'Cargo.toml', workspace)
        expected['worth-store'] = {'path': producer.as_posix(), 'features': ['certification-test-authority']}
        self.assertEqual(package['dependencies'], expected)
        self.assertEqual(package['target_dependencies'], dependency_targets(producer / 'Cargo.toml', workspace))
        self.assertFalse(source.with_suffix('.stderr').exists())


if __name__ == '__main__':
    unittest.main()
