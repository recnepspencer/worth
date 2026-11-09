from pathlib import Path
import subprocess
import tempfile
import unittest

from snapshot_warnings import (committed_warnings, load_snapshot_warning_manifest,
                               snapshot_warning_policy_violations,
                               snapshot_warning_violations, warning_lines)


class CommittedSnapshotWarningTests(unittest.TestCase):
    def test_committed_compiler_snapshots_contain_only_the_protected_refusals(self):
        repository = Path(__file__).resolve().parents[2]
        residuals = load_snapshot_warning_manifest(Path(__file__).with_name('snapshot_warning_residuals.json'))
        policy = load_snapshot_warning_manifest(Path(__file__).with_name('snapshot_warning_policy.json'))
        self.assertEqual(snapshot_warning_policy_violations(residuals, policy), [],
                         'Residual changes require a reviewed set and an open filed issue')
        self.assertEqual(snapshot_warning_violations(repository, residuals), [],
                         'Re-record unfiled lint diagnostics; remove stale or missing residuals')

    def test_residual_policy_requires_exact_paths_and_open_filed_issue_tags(self):
        policy = {'pinned_snapshots': ['known.stderr'], 'open_filed_issues': ['#117']}
        self.assertEqual(snapshot_warning_policy_violations({'known.stderr': '#117'}, policy), [])
        cases = [({'known.stderr': '#999'}, 'unfiled-snapshot-warning-issue'),
                 ({'known.stderr': '#117', 'extra.stderr': '#117'},
                  'unapproved-snapshot-warning-residual'),
                 ({}, 'missing-approved-snapshot-warning-residual')]
        for residuals, kind in cases:
            with self.subTest(residuals=residuals):
                self.assertEqual([v['kind'] for v in
                                  snapshot_warning_policy_violations(residuals, policy)], [kind])

    def test_residual_manifest_refuses_duplicate_keys_even_when_the_last_tag_is_valid(self):
        with tempfile.TemporaryDirectory() as temporary:
            manifest = Path(temporary) / 'residuals.json'
            manifest.write_text('{"known.stderr":"#117"}', encoding='utf-8')
            self.assertEqual(load_snapshot_warning_manifest(manifest), {'known.stderr': '#117'})
            manifest.write_text('{"known.stderr":"#999","known.stderr":"#117"}',
                                encoding='utf-8')
            with self.assertRaisesRegex(ValueError, 'duplicate-snapshot-warning-key: known.stderr'):
                load_snapshot_warning_manifest(manifest)

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
            self.assertEqual(snapshot_warning_violations(repository, {}), [
                {'snapshot': snapshot.name, 'lines': [2], 'kind': 'unfiled-snapshot-warning'},
            ])

    def test_a_filed_warning_must_remain_tracked_present_and_warning_bearing(self):
        with tempfile.TemporaryDirectory() as temporary:
            repository = Path(temporary)
            subprocess.run(['git', 'init', '-q', temporary], check=True)
            snapshot = repository / 'private_field.stderr'
            snapshot.write_text('error[E0451]: private field\nwarning: unused import\n',
                                encoding='utf-8')
            subprocess.run(['git', 'add', snapshot.name], cwd=repository, check=True)
            residuals = {snapshot.name: '#117'}
            self.assertEqual(snapshot_warning_violations(repository, residuals), [])
            subprocess.run(['git', 'rm', '--cached', '-q', snapshot.name], cwd=repository,
                           check=True)
            self.assertEqual(snapshot_warning_violations(repository, residuals), [
                {'kind': 'stale-snapshot-warning-residual',
                 'snapshot': snapshot.name, 'issue': '#117'},
            ])
            subprocess.run(['git', 'add', snapshot.name], cwd=repository, check=True)
            snapshot.write_text('error[E0451]: private field\n', encoding='utf-8')
            self.assertEqual(snapshot_warning_violations(repository, residuals), [
                {'kind': 'stale-snapshot-warning-residual',
                 'snapshot': snapshot.name, 'issue': '#117'},
            ])
            snapshot.unlink()
            self.assertEqual(snapshot_warning_violations(repository, residuals), [
                {'kind': 'missing-snapshot-warning-residual',
                 'snapshot': snapshot.name, 'issue': '#117'},
            ])


if __name__ == '__main__':
    unittest.main()
