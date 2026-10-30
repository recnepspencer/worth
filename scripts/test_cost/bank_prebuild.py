"""Observe Bank's dependency metadata before courts start."""
from enum import Enum
import json
from pathlib import Path


class MetadataFailureKind(Enum):
    CARGO_FAILED = 'cargo-failed'
    INVALID_METADATA = 'invalid-metadata'
    FOREIGN_WORKSPACE = 'foreign-workspace'


class BankMetadataPrebuildFailure(RuntimeError):
    def __init__(self, kind, context):
        self.kind = kind
        self.context = context
        super().__init__(f'{kind.value}: {context}')


def prepare(repository, output_directory, cargo_runner):
    """Return the environment for a court consuming the exact Cargo observation.

    cargo_runner(command, cwd, log_path) retains command evidence and returns a
    CompletedProcess with stdout, stderr and returncode. No court may fall back
    to invoking Cargo if this prerequisite is absent.
    """
    repository = Path(repository).resolve()
    output_directory = Path(output_directory).resolve()
    workspace = repository / 'workspaces/worth-query-bank-world'
    command = ['cargo', 'metadata', '--format-version', '1', '--no-deps',
               '--manifest-path', str(workspace / 'Cargo.toml')]
    output_directory.mkdir(parents=True, exist_ok=True)
    output = cargo_runner(command, repository, output_directory / 'bank-metadata.log')
    if output.returncode:
        raise BankMetadataPrebuildFailure(MetadataFailureKind.CARGO_FAILED,
                                          f'exit {output.returncode}: {output.stderr}')
    raw = output.stdout.encode('utf-8') if isinstance(output.stdout, str) else output.stdout
    try:
        metadata = json.loads(raw)
        if not isinstance(metadata, dict) or not isinstance(metadata.get('packages'), list):
            raise ValueError('Cargo metadata must contain its packages array')
        observed_workspace = Path(metadata['workspace_root']).resolve()
    except (KeyError, TypeError, ValueError, UnicodeDecodeError) as error:
        raise BankMetadataPrebuildFailure(MetadataFailureKind.INVALID_METADATA, str(error)) from error
    if observed_workspace != workspace.resolve():
        raise BankMetadataPrebuildFailure(MetadataFailureKind.FOREIGN_WORKSPACE,
                                          f'expected {workspace}, observed {observed_workspace}')
    destination = output_directory / 'bank-metadata.json'
    destination.write_bytes(raw)
    return {'WORTH_PREBUILT_BANK_METADATA': str(destination)}
