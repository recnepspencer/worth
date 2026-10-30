#!/usr/bin/env python3
"""Measure each court/testkit with compiler JSON and cargo llvm-lines file mode."""
import argparse
import hashlib
import json
from pathlib import Path
import sys
import tomllib

from cargo_capture import CargoRunner
from compiler_flags import arguments as compiler_arguments
from llvm_artifacts import attributed
from logs import llvm_lines
from run_lane import child_environment
from workspace import WORKSPACES, members, repository_root
from filesystem_path import filesystem_path


FLAGS = ['--emit=llvm-ir', '-Cno-prepopulate-passes', '-Cpasses=name-anon-globals', '-Ccodegen-units=1']


def measurement_name(identity):
    return identity if len(identity) <= 80 else identity[:48] + '-' + hashlib.sha256(identity.encode()).hexdigest()[:24]


def publish(output, measured):
    """Attach measured IR to the existing census, preserving every other observation."""
    from census import markdown
    path = Path(output) / 'census.json'
    result = json.loads(path.read_text(encoding='utf-8'))
    result.setdefault('llvm_lines', {}).update({name: value['rows'] for name, value in measured.items()})
    result.setdefault('llvm_measurements', {}).update({name: {key: value for key, value in row.items() if key != 'rows'}
                                                     for name, row in measured.items()})
    path.write_text(json.dumps(result, indent=2) + '\n', encoding='utf-8')
    path.with_suffix('.md').write_text(markdown(result), encoding='utf-8')


def targets(manifest):
    rows = []
    for package in members(manifest):
        name = package['name']
        if not any(value in name for value in ('-court-', '-testkit', '-test-support')):
            continue
        data = tomllib.loads(package['manifest'].read_text(encoding='utf-8'))
        root = package['manifest'].parent
        library = data.get('lib', {})
        source = root / library.get('path', 'src/lib.rs')
        if 'lib' in data or data['package'].get('autolib', True) and filesystem_path(source).is_file():
            if not filesystem_path(source).is_file():
                raise ValueError(f'declared library source is missing: {source}')
            rows.append(dict(package=name, target='lib', family='lib', manifest=str(package['manifest']),
                             cargo_target=library.get('name', name.replace('-', '_')),
                             source=str(source.resolve()), selector=['--lib']))
        declared = data.get('test', [])
        test_sources = {}
        for target in declared:
            candidates = ([root / target['path']] if 'path' in target else
                          [root / 'tests' / (target['name'] + '.rs'),
                           root / 'tests' / target['name'] / 'main.rs'])
            existing = [path for path in candidates if filesystem_path(path).is_file()]
            if len(existing) != 1:
                raise ValueError(f'declared test has no unique source: {name}/{target["name"]}')
            test_sources[target['name']] = existing[0]
        if data['package'].get('autotests', True):
            for source in sorted((root / 'tests').glob('*.rs')) + sorted((root / 'tests').glob('*/main.rs')):
                target_name = source.stem if source.name != 'main.rs' else source.parent.name
                if source not in test_sources.values():
                    test_sources.setdefault(target_name, source)
        for target_name, source in sorted(test_sources.items()):
            rows.append(dict(package=name, target=target_name, family='test',
                             manifest=str(package['manifest']), cargo_target=target_name,
                             source=str(source.resolve()), selector=['--test', target_name]))
    return rows


def measure(manifest, output, runner, profile='dev', selection=None):
    manifest, output = Path(manifest).resolve(), Path(output).resolve()
    output.mkdir(parents=True, exist_ok=True)
    wanted = targets(manifest)
    if selection:
        unknown = set(selection) - {row['package'] for row in wanted}
        if unknown:
            raise ValueError('unknown court/testkit package selection: ' + ', '.join(sorted(unknown)))
        wanted = [row for row in wanted if row['package'] in selection]
    if not wanted:
        raise ValueError('workspace has no selected court/testkit targets to measure')
    # This measurement-only graph keeps workspace feature unification and does
    # not execute any tests, including scheduled-only ignored tests.
    with compiler_arguments(runner, FLAGS, environment={'CARGO_INCREMENTAL': '0'}) as flags:
        command = ['cargo', 'test', '--workspace', '--all-targets', '--no-run', '--manifest-path', str(manifest),
                   '--locked', '--profile', profile, '--message-format=json',
                   '--config', 'build.incremental=false', *flags]
        environment = {name: runner.environment[name] for name in
                       ('CARGO_INCREMENTAL', 'RUSTFLAGS', 'CARGO_ENCODED_RUSTFLAGS')
                       if name in runner.environment}
        compiled = runner(command, manifest.parent, output / 'workspace-compiler.log')
    compiler_log = str(runner.logs[-1])
    if compiled.returncode:
        raise ValueError('workspace LLVM measurement build failed: ' + compiler_log)
    artifacts = attributed(compiled.stdout, wanted)
    measured = {}
    for identity, artifact in sorted(artifacts.items()):
        filename = measurement_name(identity)
        observed = runner(['cargo', 'llvm-lines', '--files', artifact['ir']['path']], manifest.parent,
                          output / (filename + '-llvm-lines.log'))
        if observed.returncode:
            raise ValueError(f'cargo llvm-lines failed for {identity}')
        rows = llvm_lines(observed.stdout)
        measured[identity] = {**artifact, 'compiler_command': command, 'cargo_profile': profile,
                             'compiler_environment': environment, 'rows': rows,
                             'compiler_log': compiler_log, 'llvm_lines_log': str(runner.logs[-1])}
        (output / 'measurements.json').write_text(json.dumps(measured, indent=2) + '\n', encoding='utf-8')
    return measured


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--workspace', choices=WORKSPACES, required=True)
    parser.add_argument('--cargo-profile', default='dev')
    parser.add_argument('--package', action='append')
    args = parser.parse_args()
    root = repository_root(Path.cwd())
    output = root / 'target/test-cost' / args.workspace
    measured = measure(root / WORKSPACES[args.workspace], output / 'llvm',
                       CargoRunner(root, child_environment()), args.cargo_profile, args.package)
    publish(output, measured)


if __name__ == '__main__':
    main()
