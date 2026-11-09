from pathlib import Path
import subprocess
import tempfile
import unittest

from snapshot_warnings import committed_warnings, warning_lines


class CommittedSnapshotWarningTests(unittest.TestCase):
    def test_committed_compiler_snapshots_contain_only_the_protected_refusals(self):
        repository = Path(__file__).resolve().parents[2]
        self.assertEqual(committed_warnings(repository), [],
                         'Remove the fixture lint and re-record through the owning recorder')

    def test_diagnostic_lines_are_distinct_from_a_quoted_source_literal(self):
        self.assertEqual(warning_lines('warning: unused import\nwarning[lint]: other\n'), [1, 2])
        self.assertEqual(warning_lines('error[E0451]: private\n  | "warning: source text";'), [])

    def test_a_tracked_snapshot_exposes_a_stray_warning(self):
        with tempfile.TemporaryDirectory() as temporary:
            repository = Path(temporary)
            subprocess.run(['git', 'init', '-q', temporary], check=True)
            snapshot = repository / 'private_field.stderr'
            snapshot.write_text('error[E0451]: private field\n', encoding='utf-8')
            subprocess.run(['git', 'add', snapshot.name], cwd=repository, check=True)
            self.assertEqual(committed_warnings(repository), [])
            snapshot.write_text('error[E0451]: private field\nwarning: unused import\n',
                                encoding='utf-8')
            self.assertEqual(committed_warnings(repository), [
                {'snapshot': snapshot.name, 'lines': [2]},
            ])


if __name__ == '__main__':
    unittest.main()
