#!/usr/bin/env python3
"""Measure each court/testkit with compiler JSON and cargo llvm-lines file mode."""
import argparse
import hashlib
import json
from pathlib import Path
import sys
import tomllib

from cargo_capture import CargoRunner
from logs import llvm_lines
from run_lane import child_environment
from workspace import WORKSPACES, members, repository_root
from filesystem_path import filesystem_path


def measurement_name(identity):
    return identity if len(identity) <= 80 else identity[:48] + '-' + hashlib.sha256(identity.encode()).hexdigest()[:24]


def targets(manifest):
    rows = []
    for package in members(manifest):
        name = package['name']
        if not any(value in name for value in ('-court-', '-testkit', '-test-support')):
            continue
        data = tomllib.loads(package['manifest'].read_text(encoding='utf-8'))
        for target in data.get('test', []):
            rows.append(dict(package=name, target=target['name'], selector=['--test', target['name']]))
        root = package['manifest'].parent
        if filesystem_path(root / data.get('lib', {}).get('path', 'src/lib.rs')).is_file():
            rows.append(dict(package=name, target='lib', selector=['--lib']))
    return rows


def measure(manifest, output, runner, profile='dev', selection=None):
    manifest, output = Path(manifest).resolve(), Path(output).resolve()
    output.mkdir(parents=True, exist_ok=True)
    measured = {}
    for target in targets(manifest):
        if selection and target['package'] not in selection:
            continue
        identity = target['package'] + '-' + target['target']
        filename = measurement_name(identity)
        ir = output / (filename + '.ll')
        command = ['cargo', 'rustc', '--manifest-path', str(manifest), '--locked',
                   '-p', target['package'], *target['selector'], '--profile', profile,
                   '--', '--emit=llvm-ir=' + str(ir) + ',link']
        compiled = runner(command, manifest.parent, output / (filename + '-compiler.log'))
        if compiled.returncode or not filesystem_path(ir).is_file():
            raise ValueError(f'compiler did not produce IR for {identity}')
        # File mode never invokes a nested Cargo process: every compiler call
        # above already retained its full --message-format=json observation.
        observed = runner(['cargo', 'llvm-lines', '--files', str(ir)], manifest.parent,
                          output / (filename + '-llvm-lines.log'))
        if observed.returncode:
            raise ValueError(f'cargo llvm-lines failed for {identity}')
        rows = llvm_lines(observed.stdout)
        measured[identity] = {'compiler_command': command, 'profile': profile, 'rows': rows}
        (output / 'measurements.json').write_text(json.dumps(measured, indent=2) + '\n', encoding='utf-8')
    return measured


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--workspace', choices=WORKSPACES, required=True)
    parser.add_argument('--cargo-profile', default='dev')
    parser.add_argument('--package', action='append')
    args = parser.parse_args()
    root = repository_root(Path.cwd())
    measure(root / WORKSPACES[args.workspace], root / 'target/test-cost' / args.workspace / 'llvm',
            CargoRunner(root, child_environment()), args.cargo_profile, args.package)


if __name__ == '__main__':
    main()
