from pathlib import Path
import tempfile
import unittest

from workspace import members, sources


class WorkspaceDiscoveryTests(unittest.TestCase):
    def test_missing_declared_member_cannot_silently_shrink_the_census(self):
        with tempfile.TemporaryDirectory() as temporary:
            manifest = Path(temporary) / 'Cargo.toml'
            manifest.write_text('[workspace]\nmembers=["missing-court"]\n')
            with self.assertRaisesRegex(ValueError, 'declared workspace member is missing'):
                members(manifest)

    def test_private_unit_test_modules_are_scanned_without_production_loops(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            (root / 'Cargo.toml').write_text('[package]\nname="product"\nversion="0.1.0"')
            (root / 'src').mkdir()
            (root / 'src/lib.rs').write_text('fn ordinary() { loop {} }\n#[cfg(test)] mod tests;')
            (root / 'src/tests.rs').write_text('#[test] fn waits() { thread::sleep(delay); }')
            values = sources(root / 'Cargo.toml')
            text = '\n'.join(value for _, _, value in values)
            self.assertIn('thread::sleep', text)
            self.assertNotIn('loop {}', text)

    def test_deep_court_source_paths_are_read_on_windows(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            (root / 'Cargo.toml').write_text('[package]\nname="long-court-path"\nversion="0.1.0"')
            import os
            path = root / 'tests' / ('nested/' * 40) / 'court.rs'
            io_path = Path('\\\\?\\' + str(path)) if os.name == 'nt' else path
            io_path.parent.mkdir(parents=True)
            io_path.write_text('fn court() {}')
            with (root / 'Cargo.toml').open('a') as manifest:
                manifest.write('\n[[test]]\nname="deep"\npath="tests/' + 'nested/' * 40 + 'court.rs"\n')
            try:
                self.assertEqual(len(sources(root / 'Cargo.toml')), 1)
            finally:
                io_path.unlink()
                parent = io_path.parent
                while parent.name == 'nested':
                    parent.rmdir()
                    parent = parent.parent

    def test_source_scan_obeys_members_and_excludes_fixtures_and_targets(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / 'Cargo.toml').write_text('[workspace]\nmembers=["crates/*"]\nexclude=["crates/excluded"]')
            for name in ['a-court-runtime', 'excluded']:
                package = root / 'crates' / name
                package.mkdir(parents=True)
                (package / 'Cargo.toml').write_text(f'[package]\nname="{name}"\nversion="0.1.0"')
                for source in ['tests/suite.rs', 'tests/ui/no.rs', 'target/hidden.rs']:
                    path = package / source
                    path.parent.mkdir(parents=True, exist_ok=True)
                    path.write_text('fn main() {}')
            self.assertEqual([m['name'] for m in members(root / 'Cargo.toml')], ['a-court-runtime'])
            self.assertEqual([p.name for _, p, _ in sources(root / 'Cargo.toml')], ['suite.rs'])

    def test_mounted_ui_harness_is_scanned_while_unmounted_ui_fixture_is_not(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            (root / 'Cargo.toml').write_text('[package]\nname="a-court-runtime"\nversion="0.1.0"')
            (root / 'tests/ui').mkdir(parents=True)
            (root / 'tests/court.rs').write_text('#[path="ui/mod.rs"] mod ui;')
            (root / 'tests/ui/mod.rs').write_text('#[test] fn boundary() { trybuild::TestCases::new(); }')
            (root / 'tests/ui/fixture.rs').write_text('fn main() {}')
            found = {path.relative_to(root).as_posix() for _, path, _ in sources(root / 'Cargo.toml')}
            self.assertIn('tests/ui/mod.rs', found)
            self.assertNotIn('tests/ui/fixture.rs', found)


if __name__ == '__main__':
    unittest.main()
