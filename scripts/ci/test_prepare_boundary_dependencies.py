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
        (self.config/'domain-values.toml').write_text('workspace_manifests=["domain/Cargo.toml"]\n')
        self.expected = ['Cargo.toml', 'allowed/Cargo.toml', 'context/Cargo.toml', 'contracts/Cargo.toml',
                         'courts/Cargo.toml', 'denied/Cargo.toml', 'domain/Cargo.toml', 'target/Cargo.toml']
        for relative in self.expected:
            manifest = self.root/relative; manifest.parent.mkdir(parents=True, exist_ok=True)
            manifest.write_text('[workspace]\n')
            manifest.with_name('Cargo.lock').write_text('version=4\n')

    def test_every_existing_boundary_workspace_source_is_used_once(self):
        self.assertEqual(manifests(self.root), self.expected)

    def test_domain_registry_union_deduplicates_other_sources(self):
        (self.config/'domain-values.toml').write_text(
            'workspace_manifests=["domain/Cargo.toml", "courts/Cargo.toml", '
            '"domain/Cargo.toml", "Cargo.toml"]\n')
        self.assertEqual(manifests(self.root), self.expected)

    def test_optional_domain_inventory_can_be_absent(self):
        (self.config/'domain-values.toml').write_text('binding=[]\n')
        self.assertEqual(manifests(self.root),
                         [path for path in self.expected if path != 'domain/Cargo.toml'])

    def test_invalid_domain_inventory_refuses_before_any_fetch(self):
        for value in ('1', '[1]', '["domain/lib.rs"]',
                      '["missing/Cargo.toml"]', '["../outside/Cargo.toml"]'):
            with self.subTest(value=value):
                (self.config/'domain-values.toml').write_text('workspace_manifests=' + value)
                invoke = Mock()
                with self.assertRaises(ValueError):
                    prepare(self.root, invoke=invoke)
                invoke.assert_not_called()

    def test_missing_or_malformed_domain_registry_refuses_before_any_fetch(self):
        registry = self.config/'domain-values.toml'
        registry.unlink()
        invoke = Mock()
        with self.assertRaises(OSError):
            prepare(self.root, invoke=invoke)
        invoke.assert_not_called()
        registry.write_text('workspace_manifests=[')
        with self.assertRaises(ValueError):
            prepare(self.root, invoke=invoke)
        invoke.assert_not_called()

    def test_domain_manifest_and_lock_are_both_required_before_any_fetch(self):
        manifest = self.root/'domain/Cargo.toml'
        for missing in (manifest, manifest.with_name('Cargo.lock')):
            with self.subTest(missing=missing.name):
                original = missing.read_text()
                missing.unlink()
                invoke = Mock()
                with self.assertRaisesRegex(ValueError, 'manifest and lock'):
                    prepare(self.root, invoke=invoke)
                invoke.assert_not_called()
                missing.write_text(original)

    def test_actual_ci_inventory_contains_every_configured_domain_workspace(self):
        import tomllib
        repository = Path(__file__).resolve().parents[2]
        config = repository/'tools/boundary-check/config/domain-values.toml'
        domain = tomllib.loads(config.read_text(encoding='utf-8'))['workspace_manifests']
        selected = manifests(repository)
        self.assertTrue(domain)
        self.assertTrue(set(domain).issubset(selected))
        self.assertEqual(len(selected), len(set(selected)))

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
