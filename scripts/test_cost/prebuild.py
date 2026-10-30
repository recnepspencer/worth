"""Workspace prerequisite ownership shared by root, Query, Bank and Store gates."""
import argparse
import json
import os
from pathlib import Path
import sys

from cargo_capture import CargoRunner
from workspace import WORKSPACES, repository_root


def prepare(repository, workspace, output, runner=None, include_proof=True, compiler_scope=None, profile='dev',
            native_build=None):
    repository, output = Path(repository).resolve(), Path(output).resolve()
    output.mkdir(parents=True, exist_ok=True)
    runner = runner or CargoRunner(repository)
    if workspace == 'store':
        from store_prebuild import require_native
        native = require_native(repository, native_build)
    if workspace in ('root', 'store'):
        from compiler_prebuild import CompilerScope, prepare as compiler
        compiler_environment = compiler(repository, workspace, output / 'compiler', runner,
                                        compiler_scope or CompilerScope.ALL)
    else:
        compiler_environment = {}
    if workspace == 'root':
        from proof_prebuild import prepare as proof
        environment = {'WORTH_PREBUILT_PROOF': str(proof(repository, output / 'proof', runner))} if include_proof else {}
    elif workspace == 'bank':
        from bank_prebuild import prepare as bank
        environment = bank(repository, output / 'bank', runner)
    elif workspace == 'store':
        from store_prebuild import prepare as store
        environment = store(repository, output / 'store', runner, profile, native=native)
    elif workspace == 'query':
        environment = {}
    else:
        raise ValueError(f'ungoverned workspace: {workspace}')
    environment.update(compiler_environment)
    (output / 'environment.json').write_text(json.dumps(environment, indent=2) + '\n', encoding='utf-8')
    return environment


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--workspace', choices=WORKSPACES, required=True)
    parser.add_argument('--output', type=Path)
    parser.add_argument('--native-build', type=Path, help='native build directory; required for Store')
    args = parser.parse_args()
    root = repository_root(Path.cwd())
    output = args.output or root / 'target/test-cost' / args.workspace / 'prebuilt'
    environment = prepare(root, args.workspace, output, native_build=args.native_build)
    print(json.dumps(environment, indent=2))


if __name__ == '__main__':
    main()
