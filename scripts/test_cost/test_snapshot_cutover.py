from pathlib import Path
import tempfile
import unittest

from snapshot_cutover import apply, plan


class SnapshotOwnershipCutoverTests(unittest.TestCase):
    def test_active_fixture_keeps_its_exact_snapshot_bytes_under_one_owner(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            manifest = root / 'Cargo.toml'
            manifest.write_text('[workspace]\nmembers=["crates/*"]\n')
            kit, court = root / 'crates/kit', root / 'crates/court'
            for directory, name in [(kit, 'worth-testkit-server'), (court, 'worth-court-server')]:
                directory.mkdir(parents=True)
                (directory / 'Cargo.toml').write_text(f'[package]\nname="{name}"\nversion="0.0.0"\n')
            with (court / 'Cargo.toml').open('a') as stream:
                stream.write('[[test]]\nname="court"\npath="tests/court.rs"\n')
            snapshot = kit / 'assets/ui/sealed.stderr'
            source = court / 'tests/court/sources/worth_server/ui/sealed.rs'
            snapshot.parent.mkdir(parents=True)
            source.parent.mkdir(parents=True)
            snapshot.write_bytes(b'error: sealed\r\n  --> tests/ui/sealed.rs:1:2\r\n')
            source.write_text('fn main() {}')
            expected = snapshot.read_bytes()
            moves = plan(manifest, root)
            self.assertEqual(moves, [(snapshot, source.with_suffix('.stderr'))])
            apply(moves, root)
            self.assertEqual(source.with_suffix('.stderr').read_bytes(), expected)
            self.assertFalse(snapshot.exists())
            self.assertEqual(plan(manifest, root), [])


if __name__ == '__main__':
    unittest.main()
