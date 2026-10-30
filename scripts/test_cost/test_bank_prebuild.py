import json
from pathlib import Path
import subprocess
import tempfile
import unittest

from bank_prebuild import BankMetadataPrebuildFailure, MetadataFailureKind, prepare


class BankMetadataPrebuildTests(unittest.TestCase):
    def test_preserves_raw_metadata_and_declares_exact_workspace_command(self):
        with tempfile.TemporaryDirectory() as temporary:
            repository = Path(temporary).resolve()
            workspace = repository / 'workspaces/worth-query-bank-world'
            raw = json.dumps({'packages': [], 'workspace_root': str(workspace)}, indent=3).encode()
            calls = []

            def cargo_runner(command, cwd, log_path):
                calls.append((command, cwd, log_path))
                return subprocess.CompletedProcess(command, 0, raw, b'')

            output = repository / 'target/test-cost/bank'
            environment = prepare(repository, output, cargo_runner)
            self.assertEqual(Path(environment['WORTH_PREBUILT_BANK_METADATA']).read_bytes(), raw)
            self.assertEqual(calls, [(['cargo', 'metadata', '--format-version', '1', '--no-deps',
                                      '--manifest-path', str(workspace / 'Cargo.toml')],
                                     repository, output / 'bank-metadata.log')])

    def test_failed_metadata_command_refuses_before_publishing_an_observation(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            def cargo_runner(command, cwd, log_path):
                return subprocess.CompletedProcess(command, 101, b'{}', b'broken manifest')
            with self.assertRaises(BankMetadataPrebuildFailure) as failure:
                prepare(root, root / 'out', cargo_runner)
            self.assertEqual(failure.exception.kind, MetadataFailureKind.CARGO_FAILED)
            self.assertFalse((root / 'out/bank-metadata.json').exists())

    def test_invalid_json_and_foreign_workspace_are_distinct_refusals(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            for raw, kind in [(b'{', MetadataFailureKind.INVALID_METADATA),
                              (b'{}', MetadataFailureKind.INVALID_METADATA),
                              (json.dumps({'packages': [], 'workspace_root': str(root)}),
                               MetadataFailureKind.FOREIGN_WORKSPACE)]:
                def cargo_runner(command, cwd, log_path):
                    return subprocess.CompletedProcess(command, 0, raw, b'')
                with self.assertRaises(BankMetadataPrebuildFailure) as failure:
                    prepare(root, root / 'out', cargo_runner)
                self.assertEqual(failure.exception.kind, kind)


if __name__ == '__main__':
    unittest.main()
