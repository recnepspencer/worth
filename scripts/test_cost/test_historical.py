from pathlib import Path
import tarfile
import unittest

from historical import safe_member


class HistoricalSourceIsolationTests(unittest.TestCase):
    def test_archived_paths_stay_below_the_named_snapshot_directory(self):
        root = Path('.').resolve()
        member = tarfile.TarInfo('crates/court/src/lib.rs')
        self.assertIs(safe_member(root, member), member)
        with self.assertRaises(ValueError):
            safe_member(root, tarfile.TarInfo('../outside.rs'))

    def test_archive_links_cannot_redirect_source_reads_to_another_worktree(self):
        member = tarfile.TarInfo('crates/court/src/link.rs')
        member.type = tarfile.SYMTYPE
        with self.assertRaises(ValueError):
            safe_member(Path('.'), member)


if __name__ == '__main__':
    unittest.main()
