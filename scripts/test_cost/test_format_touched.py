from pathlib import Path
import unittest
import tempfile

from format_touched import changed_sources, package_for
from filesystem_path import filesystem_path


class FormatOwnershipTests(unittest.TestCase):
    def test_windows_long_paths_do_not_hide_affected_crates_from_formatting(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            relative = Path(*(['court_domain'] * 25)) / 'proof.rs'
            source = filesystem_path(root / relative)
            source.parent.mkdir(parents=True)
            source.write_text('fn main() {}')
            try:
                self.assertEqual(changed_sources(root, [str(relative)]), [root / relative])
            finally:
                source.unlink()
                parent = source.parent
                while parent.name == 'court_domain':
                    parent.rmdir()
                    parent = parent.parent

    def test_fixture_source_uses_its_nearest_owning_manifest(self):
        root = Path('.').resolve()
        packages = [dict(name='outer', manifest=root / 'Cargo.toml'),
                    dict(name='court', manifest=root / 'crates/court/Cargo.toml')]
        self.assertEqual(package_for(root / 'crates/court/tests/source.rs', packages)['name'], 'court')
        self.assertIsNone(package_for(root.parent / 'elsewhere/source.rs', packages))


if __name__ == '__main__':
    unittest.main()
