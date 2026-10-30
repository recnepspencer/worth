"""One Cargo check session per declared compiler-fixture environment."""
from enum import Enum
import hashlib
import json
import os
from pathlib import Path
import shutil
import sys
import tomllib

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'test_topology'))
from manifest_edit import toml_value

from logs import json_objects


class FixtureFate(Enum):
    COMPILED = 'compiled'
    DENIED = 'denied'
    UNOBSERVED = 'unobserved'


class BatchCompilationFailure(ValueError):
    pass


def bin_name(source):
    return 'fixture_' + hashlib.sha256(str(source).replace('\\', '/').encode()).hexdigest()[:24]


def render_manifest(environment, workspace_manifest):
    dependencies = environment['dependency_manifest']
    if '[dependencies]' not in dependencies:
        raise BatchCompilationFailure('fixture environment must declare dependencies')
    profile = tomllib.loads(Path(workspace_manifest).read_text(encoding='utf-8'))['profile']
    name = 'compiler-fixtures-' + hashlib.sha256(environment['identity'].encode()).hexdigest()[:20]
    lines = [f'[package]\nname={json.dumps(name)}\nversion="0.0.0"\nedition="2024"\nautobins=false\n',
             '[workspace]\nresolver="3"\n', dependencies]
    seen = set()
    for source in environment['sources']:
        source = str(Path(source).resolve()).replace('\\', '/')
        name = bin_name(source)
        if name in seen:
            raise BatchCompilationFailure(f'duplicate fixture source: {source}')
        seen.add(name)
        lines += [f'[[bin]]\nname={json.dumps(name)}\npath={json.dumps(source)}\n']
    for name, value in profile.items():
        lines += [f'[profile.{json.dumps(name)}]\n' +
                  '\n'.join(f'{json.dumps(k)}={toml_value(v)}' for k, v in value.items()) + '\n']
    return '\n'.join(lines)


def attribute(events, sources):
    names = {bin_name(str(Path(source).resolve()).replace('\\', '/')): str(Path(source).resolve())
             for source in sources}
    result = {source: {'fate': FixtureFate.UNOBSERVED.value, 'messages': [], 'artifacts': []}
              for source in names.values()}
    dependency_errors = []
    for event in events:
        if event.get('reason') not in ('compiler-message', 'compiler-artifact'):
            continue
        target = event['target']['name']
        if target not in names:
            if event.get('message', {}).get('level') == 'error':
                dependency_errors.append(event)
            continue
        outcome = result[names[target]]
        if event['reason'] == 'compiler-message':
            outcome['messages'].append(event['message'])
            if event['message']['level'] == 'error':
                outcome['fate'] = FixtureFate.DENIED.value
        else:
            outcome['artifacts'].append(event)
            if outcome['fate'] != FixtureFate.DENIED.value:
                outcome['fate'] = FixtureFate.COMPILED.value
    if dependency_errors:
        raise BatchCompilationFailure('fixture dependency failed before the claimed boundary: ' +
                                      json.dumps(dependency_errors))
    missing = [source for source, outcome in result.items() if outcome['fate'] == FixtureFate.UNOBSERVED.value]
    if missing:
        raise BatchCompilationFailure(f'Cargo did not compile every declared fixture: {missing}')
    return result


def prepare_environment(workspace_manifest, environment, output, cargo_runner):
    workspace_manifest = Path(workspace_manifest).resolve()
    output = Path(output).resolve()
    output.mkdir(parents=True, exist_ok=True)
    (output / 'Cargo.toml').write_text(render_manifest(environment, workspace_manifest), encoding='utf-8')
    # Resolve the generated package's own lock entry offline, starting from the
    # workspace's exact versions. No compiler is invoked by metadata.
    shutil.copyfile(workspace_manifest.with_name('Cargo.lock'), output / 'Cargo.lock')
    resolved = cargo_runner(['cargo', 'metadata', '--offline', '--format-version', '1',
                             '--manifest-path', str(output / 'Cargo.toml')],
                            workspace_manifest.parent, output / 'metadata.log')
    if resolved.returncode:
        raise BatchCompilationFailure(f'could not resolve compiler environment {environment["identity"]}')
    verify_locked_dependencies(workspace_manifest.with_name('Cargo.lock'),
                               json.loads(resolved.stdout))
    if environment.get('reject_test_features'):
        metadata = json.loads(resolved.stdout)
        forbidden = [(node['id'], feature) for node in metadata['resolve']['nodes']
                     for feature in node['features']
                     if 'certification' in feature or 'for-test' in feature or 'for_test' in feature]
        if forbidden:
            raise BatchCompilationFailure(f'production compiler graph enables test authority: {forbidden}')
    target = target_directory(workspace_manifest, cargo_runner)
    command = ['cargo', 'check', '--bins', '--keep-going', '--offline', '--locked',
               '--manifest-path', str(output / 'Cargo.toml'),
               '--target-dir', str(target), '--config', 'build.incremental=false',
               '--profile', environment['profile']]
    observed = cargo_runner(command, workspace_manifest.parent, output / 'check.log')
    if observed.returncode not in (0, 101):
        raise BatchCompilationFailure(f'compiler session failed to execute: {observed.returncode}')
    outcomes = attribute(list(json_objects(observed.stdout)), environment['sources'])
    value = {'schema': 1, 'environment': environment['identity'], 'profile': environment['profile'],
             'dependency_manifest': environment['dependency_manifest'],
             'compiler_exit': observed.returncode, 'fixtures': outcomes}
    (output / 'outcomes.json').write_text(json.dumps(value, indent=2) + '\n', encoding='utf-8')
    return value


def verify_locked_dependencies(lockfile, metadata):
    locked = tomllib.loads(Path(lockfile).read_text(encoding='utf-8'))
    versions = {(row['name'], row['version'], row.get('source')) for row in locked.get('package', [])}
    changed = [(row['name'], row['version'], row['source']) for row in metadata['packages']
               if row.get('source') is not None and
               (row['name'], row['version'], row['source']) not in versions]
    if changed:
        raise BatchCompilationFailure(f'fixture resolution escaped workspace Cargo.lock: {changed}')


def target_directory(workspace_manifest, cargo_runner):
    environment = getattr(cargo_runner, 'environment', None) or os.environ
    workspace_manifest = Path(workspace_manifest).resolve()
    target = Path(environment.get('CARGO_TARGET_DIR', str(workspace_manifest.parent / 'target')))
    return (target if target.is_absolute() else workspace_manifest.parent / target).resolve()
