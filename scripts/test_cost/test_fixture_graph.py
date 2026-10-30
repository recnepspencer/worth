from pathlib import Path
import tempfile
import unittest

from fixture_graph import dependency_surface, merge_dependencies, snapshot_inputs


class FixtureGraphTests(unittest.TestCase):
    def test_orphan_snapshot_is_reported_without_inventing_a_compiler_input(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            (root / 'tests').mkdir()
            (root / 'Cargo.toml').write_text('[package]\nname="fixture-court"\nversion="0.1.0"\n[workspace]\n')
            (root / 'tests/retired.stderr').write_text('error')
            orphaned = []
            self.assertEqual(snapshot_inputs(root / 'Cargo.toml', orphaned), [])
            self.assertEqual(len(orphaned), 1)

    def test_unused_testkit_is_not_a_compiler_fixture_dependency(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            (root / 'tests/ui').mkdir(parents=True)
            (root / 'Cargo.toml').write_text('[package]\nname="fixture-court-runtime"\nversion="0.1.0"\n'
                                           '[workspace]\n[dependencies]\nowner="1"\nunused-testkit="2"\n')
            (root / 'tests/ui/denied.rs').write_text('use owner::Private; fn main() {}')
            (root / 'tests/ui/denied.stderr').write_text('error: private')
            rows = snapshot_inputs(root / 'Cargo.toml')
            self.assertEqual(set(rows[0]['dependencies']), {'owner'})

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


if __name__ == '__main__':
    unittest.main()
