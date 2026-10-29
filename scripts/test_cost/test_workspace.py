from pathlib import Path
import tempfile
import unittest

from workspace import members, sources


class WorkspaceDiscoveryTests(unittest.TestCase):
    def test_deep_court_source_paths_are_read_on_windows(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            (root / 'Cargo.toml').write_text('[package]\nname="long-court-path"\nversion="0.1.0"')
            import os
            path = root / 'tests' / ('nested/' * 40) / 'court.rs'
            io_path = Path('\\\\?\\' + str(path)) if os.name == 'nt' else path
            io_path.parent.mkdir(parents=True)
            io_path.write_text('fn court() {}')
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


if __name__ == '__main__':
    unittest.main()
