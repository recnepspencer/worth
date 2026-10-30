"""Reach follows Cargo membership and inputs rather than a second package map."""
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

from cargo_reach import changed_paths, consumer_documents, local_package_dirs, workspace_hits


class CargoReachTests(unittest.TestCase):
    def test_metadata_keeps_transitive_local_dependencies_and_member_roles(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            packages = [{'id': name, 'source': source,
                         'manifest_path': str(root/directory/'Cargo.toml')}
                        for name, directory, source in [('owner', 'workspace/owner', None),
                                                       ('dep', 'crates/dep', None),
                                                       ('registry', 'unused', 'registry+url')]]
            with patch('cargo_reach.run', return_value=json.dumps(
                    {'packages': packages, 'workspace_members': ['owner']})) as call:
                self.assertEqual(local_package_dirs(root, 'workspace/Cargo.toml'),
                                 [('workspace/owner', True), ('crates/dep', False)])
                self.assertIn('--locked', call.call_args.args[0])
                self.assertIn('--offline', call.call_args.args[0])
                local_package_dirs(root, 'workspace/Cargo.toml', offline=False)
                self.assertNotIn('--offline', call.call_args.args[0])

    def test_membership_and_inherited_inputs_include_deleted_files(self):
        directories = [('workspaces/consumer/owner', True), ('crates/shared', False)]
        paths = ['crates/shared/src/deleted.rs', 'crates/shared/tests/unrelated.rs',
                 'workspaces/consumer/owner/tests/claim.rs', 'crates/shared/AGENT_CONTEXT.md',
                 'Cargo.toml', 'Cargo.lock', '.cargo/config.toml', 'rust-toolchain.toml',
                 'workspaces/consumer/Cargo.toml', 'workspaces/consumer/.cargo/config',
                 'Cargo.toml.backup', 'plans/unrelated.md']
        self.assertEqual(workspace_hits(paths, 'workspaces/consumer/Cargo.toml', directories),
                         [path for path in paths if path not in
                          ('crates/shared/tests/unrelated.rs', 'crates/shared/AGENT_CONTEXT.md',
                           'Cargo.toml.backup', 'plans/unrelated.md')])

    def test_configured_documents_come_from_nested_real_configuration(self):
        with tempfile.TemporaryDirectory() as temporary:
            config = Path(temporary)/'tools/boundary-check/config/consumer-docs.toml'
            config.parent.mkdir(parents=True)
            config.write_text('documents=["docs/selected.md"]\n[[nested]]\npath="other.md"\n')
            self.assertEqual(consumer_documents(temporary), {'docs/selected.md', 'other.md'})

    def test_outside_local_package_refuses_a_partial_graph(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            graph = {'workspace_members': [], 'packages': [{'id': 'escape', 'source': None,
                      'manifest_path': str(root.parent/'outside/Cargo.toml')}]}
            with patch('cargo_reach.run', return_value=json.dumps(graph)):
                with self.assertRaisesRegex(ValueError, 'outside the repository'):
                    local_package_dirs(root, 'Cargo.toml')

    def test_git_diff_keeps_renamed_and_deleted_owners_and_rejects_unknown_base(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            def git(*args):
                return subprocess.check_output(['git', *args], cwd=root, text=True).strip()
            git('init', '-q'); git('config', 'user.email', 'fixture@example.invalid')
            git('config', 'user.name', 'Fixture')
            (root/'old.rs').write_text('fn main() {}\n')
            (root/'deleted.rs').write_text('fn deleted() {}\n')
            git('add', '.'); git('commit', '-qm', 'before'); base = git('rev-parse', 'HEAD')
            (root/'old.rs').rename(root/'new.rs'); (root/'deleted.rs').unlink()
            git('add', '-A'); git('commit', '-qm', 'after')
            self.assertEqual(changed_paths(root, base, 'HEAD'), ['deleted.rs', 'new.rs', 'old.rs'])
            with self.assertRaises(subprocess.CalledProcessError):
                changed_paths(root, 'nonexistent-base', 'HEAD')
            with self.assertRaisesRegex(ValueError, 'comparison base'):
                changed_paths(root, '0'*40, 'HEAD')


if __name__ == '__main__':
    unittest.main()
