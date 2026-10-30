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
from diagnostic_flags import arguments as diagnostic_arguments
from compiler_audiences import source_identity, sources as fixture_sources
from compiler_features import violations as production_feature_violations
from fixture_graph import dependency_manifest


class FixtureFate(Enum):
    COMPILED = 'compiled'
    DENIED = 'denied'
    UNOBSERVED = 'unobserved'


class BatchCompilationFailure(ValueError):
    pass


def bin_name(source):
    return 'fixture_' + hashlib.sha256(source_identity(source).replace('\\', '/').encode()).hexdigest()[:24]


def declared_targets(sources):
    names, seen = {}, set()
    for source in sources:
        key = source_identity(source)
        if key in seen:
            raise BatchCompilationFailure(f'duplicate fixture source: {source}')
        seen.add(key)
        name = bin_name(source)
        if name in names:
            raise BatchCompilationFailure(f'generated fixture target name collision: {names[name]} and {source}')
        names[name] = str(Path(source).resolve())
    return names


def cargo_target(event):
    target = event.get('target')
    kinds = {'bin', 'lib', 'rlib', 'dylib', 'cdylib', 'staticlib', 'proc-macro',
             'custom-build', 'example', 'test', 'bench'}
    if (not isinstance(target, dict) or not isinstance(target.get('name'), str) or
            not target['name'] or target['name'].strip() != target['name'] or
            not isinstance(target.get('kind'), list) or not target['kind'] or
            any(not isinstance(kind, str) or kind not in kinds for kind in target['kind']) or
            not isinstance(target.get('src_path'), str) or not target['src_path'] or
            '\x00' in target['src_path'] or not Path(target['src_path']).is_absolute()):
        raise BatchCompilationFailure(f'compiler observation has malformed target metadata: {target}')
    try:
        return target['name'], target['kind'], source_identity(target['src_path'])
    except (OSError, ValueError) as error:
        raise BatchCompilationFailure(f'compiler target source cannot be resolved: {target}') from error


def render_manifests(environment, workspace_manifest):
    try:
        declared_targets(fixture_sources(environment))
    except ValueError as error:
        raise BatchCompilationFailure(str(error)) from error
    profile = tomllib.loads(Path(workspace_manifest).read_text(encoding='utf-8')).get('profile', {})
    batch = hashlib.sha256(environment['identity'].encode()).hexdigest()[:12]
    manifests, members, names = {}, [], set()
    for package in environment['packages']:
        member = 'packages/' + package['identity']
        members.append(member)
        name = f'compiler-fixtures-{batch}-{package["identity"][:24]}'
        if name in names:
            raise BatchCompilationFailure(f'compiler audience package name collision: {name}')
        names.add(name)
        lines = [f'[package]\nname={json.dumps(name)}\nversion="0.0.0"\nedition="2024"\nautobins=false\n',
                 dependency_manifest(package['dependencies'], package['target_dependencies'])]
        for source in package['sources']:
            source = str(Path(source).resolve()).replace('\\', '/')
            lines += [f'[[bin]]\nname={json.dumps(bin_name(source))}\npath={json.dumps(source)}\n']
        manifests[member + '/Cargo.toml'] = '\n'.join(lines)
    lines = ['[workspace]\nresolver="3"\nmembers=' + json.dumps(members) + '\n']
    for name, value in profile.items():
        lines += [f'[profile.{json.dumps(name)}]\n' +
                  '\n'.join(f'{json.dumps(k)}={toml_value(v)}' for k, v in value.items()) + '\n']
    manifests['Cargo.toml'] = '\n'.join(lines)
    return dict(sorted(manifests.items()))


def attribute(events, sources):
    names = declared_targets(sources)
    result = {source: {'fate': FixtureFate.UNOBSERVED.value, 'messages': [], 'artifacts': []}
              for source in names.values()}
    dependency_errors = []
    for event in events:
        if event.get('reason') not in ('compiler-message', 'compiler-artifact'):
            continue
        target, kinds, source = cargo_target(event)
        if event['reason'] == 'compiler-message' and (
                not isinstance(event.get('message'), dict) or not isinstance(event['message'].get('level'), str)):
            raise BatchCompilationFailure('compiler observation has malformed diagnostic metadata')
        if target not in names or kinds != ['bin'] or source != source_identity(names[target]):
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
    manifests = render_manifests(environment, workspace_manifest)
    for relative, text in manifests.items():
        path = output / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding='utf-8')
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
        forbidden = production_feature_violations(workspace_manifest, json.loads(resolved.stdout))
        if forbidden:
            raise BatchCompilationFailure(f'production compiler graph enables test authority: {forbidden}')
    target = target_directory(workspace_manifest, cargo_runner)
    command = ['cargo', 'check', '--workspace', '--bins', '--keep-going', '--offline', '--locked',
               '--manifest-path', str(output / 'Cargo.toml'),
               '--target-dir', str(target), '--config', 'build.incremental=false',
               '--profile', environment['profile']]
    with diagnostic_arguments(cargo_runner) as flags:
        observed = cargo_runner(command + flags, workspace_manifest.parent, output / 'check.log')
    if observed.returncode not in (0, 101):
        raise BatchCompilationFailure(f'compiler session failed to execute: {observed.returncode}')
    outcomes = attribute(list(json_objects(observed.stdout)), fixture_sources(environment))
    for package in environment['packages']:
        for source in package['sources']:
            outcomes[str(Path(source).resolve())]['audience_identity'] = package['identity']
    value = {'schema': 1, 'environment': environment['identity'], 'profile': environment['profile'],
             'manifests': manifests,
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
