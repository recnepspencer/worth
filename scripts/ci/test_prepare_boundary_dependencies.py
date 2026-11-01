"""Boundary preparation reads configured workspaces and preserves Cargo failures."""
import contextlib
import io
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import Mock

from prepare_boundary_dependencies import manifests, prepare


class BoundaryDependencyPreparationTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.config = self.root/'tools/boundary-check/config'
        self.config.mkdir(parents=True)
        (self.config/'road1.toml').write_text(
            '[[subworkspaces]]\npath="contracts"\n[[context_workspaces]]\npath="context"\n'
            '[[dependency_denials]]\nworkspace_manifest="denied/Cargo.toml"\n'
            '[[source_dependency_allowlists]]\nworkspace_manifest="allowed/Cargo.toml"\n'
            '[[dependency_target_allowlists]]\nworkspace_manifest="target/Cargo.toml"\n')
        (self.config/'test-workspaces.txt').write_text('# inventory\n. | root\ncourts | stem\n')
        self.expected = ['Cargo.toml', 'allowed/Cargo.toml', 'context/Cargo.toml', 'contracts/Cargo.toml',
                         'courts/Cargo.toml', 'denied/Cargo.toml', 'target/Cargo.toml']
        for relative in self.expected:
            manifest = self.root/relative; manifest.parent.mkdir(parents=True, exist_ok=True)
            manifest.write_text('[workspace]\n')
            manifest.with_name('Cargo.lock').write_text('version=4\n')

    def test_every_existing_boundary_workspace_source_is_used_once(self):
        self.assertEqual(manifests(self.root), self.expected)

    def test_fetches_are_locked_and_a_cargo_failure_stops_with_original_code(self):
        invoke = Mock(side_effect=[subprocess.CompletedProcess([], 0), subprocess.CompletedProcess([], 101)])
        with contextlib.redirect_stdout(io.StringIO()):
            self.assertEqual(prepare(self.root, invoke=invoke), 101)
        self.assertEqual(invoke.call_count, 2)
        self.assertEqual(invoke.call_args_list[0].args[0],
                         ['cargo', 'fetch', '--locked', '--manifest-path', 'Cargo.toml'])
        self.assertEqual(invoke.call_args_list[0].kwargs, {'cwd': self.root})

    def test_dry_run_never_invokes_cargo(self):
        invoke = Mock(side_effect=AssertionError('unexpected Cargo'))
        with contextlib.redirect_stdout(io.StringIO()) as output:
            self.assertEqual(prepare(self.root, dry_run=True, invoke=invoke), 0)
        self.assertEqual(len(output.getvalue().splitlines()), len(self.expected))
        invoke.assert_not_called()

    def test_missing_lock_or_outside_workspace_fails_before_fetching(self):
        (self.root/'Cargo.lock').unlink()
        invoke = Mock()
        with self.assertRaisesRegex(ValueError, 'manifest and lock'):
            prepare(self.root, invoke=invoke)
        invoke.assert_not_called()
        (self.root/'Cargo.lock').write_text('version=4\n')
        with (self.config/'road1.toml').open('a') as stream:
            stream.write('[[subworkspaces]]\npath="../outside"\n')
        with self.assertRaisesRegex(ValueError, 'manifest and lock'):
            prepare(self.root, invoke=invoke)
        invoke.assert_not_called()


if __name__ == '__main__':
    unittest.main()
