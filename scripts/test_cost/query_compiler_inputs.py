"""Prebuild the real consumer workspace's foreign-schema compiler contract."""
import hashlib
import json
from pathlib import Path

from compile_batch import BatchCompilationFailure, verify_locked_dependencies
from compiler_audiences import surface_identity
from compiler_prebuild import normalized_source_key
from fixture_graph import dependency_surface, dependency_targets
from logs import json_objects


CONSUMER = 'workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry'
PACKAGES = ('topology_entry', 'parameter_entry', 'consumer_root', 'hostile_foreign_schema')


def prepare(repository, output, runner):
    repository, output = Path(repository), Path(output)
    output.mkdir(parents=True, exist_ok=True)
    root = repository / CONSUMER
    manifest = root / 'Cargo.toml'
    metadata = runner(['cargo', 'metadata', '--offline', '--format-version', '1',
                       '--manifest-path', str(manifest)], repository, output / 'metadata.log')
    if metadata.returncode:
        raise BatchCompilationFailure('consumer compiler metadata failed')
    resolved = json.loads(metadata.stdout)
    verify_locked_dependencies(manifest.with_name('Cargo.lock'), resolved)
    owners = {package['manifest_path']: package for package in resolved['packages']}
    targets, audiences = {}, {}
    for member in PACKAGES:
        package = owners[str(root / member / 'Cargo.toml')]
        target = next(t for t in package['targets'] if t['kind'] in (['lib'], ['bin']))
        source = normalized_source_key(target['src_path'])
        targets[source] = (package['id'], target)
        audiences[source] = surface_identity(dependency_surface(root / member / 'Cargo.toml', manifest),
                                            dependency_targets(root / member / 'Cargo.toml', manifest))
    command = ['cargo', 'check', '--offline', '--locked', '--keep-going',
               '--manifest-path', str(manifest)]
    for member in PACKAGES:
        command += ['-p', owners[str(root / member / 'Cargo.toml')]['name']]
    observed = runner(command, repository, output / 'check.log')
    if observed.returncode not in (0, 101):
        raise BatchCompilationFailure(f'consumer compiler session did not execute: {observed.returncode}')
    events = list(json_objects(observed.stdout))
    for event in events:
        if event.get('reason') == 'compiler-message':
            resolve_locations(event['message'], root)
    outcomes = attributed(events, targets)
    identity = hashlib.sha256(json.dumps({'targets': targets, 'audiences': audiences}, sort_keys=True).encode()).hexdigest()
    for source, outcome in outcomes.items():
        outcome.update(environment='production', environment_identity=identity,
                       audience_identity=audiences[source], shared_target_root=resolved['target_directory'],
                       proof_stage='check')
    return outcomes


def attributed(events, targets):
    outcomes = {source: {'fate': 'unobserved', 'messages': []} for source in targets}
    for event in events:
        if event.get('reason') not in ('compiler-artifact', 'compiler-message'):
            continue
        target = event['target']
        source = normalized_source_key(target['src_path'])
        expected = targets.get(source)
        exact = expected and event['package_id'] == expected[0] and target == expected[1]
        if not exact:
            if event.get('message', {}).get('level') == 'error':
                raise BatchCompilationFailure('consumer dependency failed before the schema contract: ' + json.dumps(event))
            continue
        outcome = outcomes[source]
        if event['reason'] == 'compiler-message':
            outcome['messages'].append(event['message'])
            if event['message']['level'] == 'error':
                outcome['fate'] = 'denied'
        elif outcome['fate'] != 'denied':
            outcome['fate'] = 'compiled'
    if any(row['fate'] == 'unobserved' for row in outcomes.values()):
        raise BatchCompilationFailure('consumer compiler did not observe every declared counterpart')
    return outcomes


def resolve_locations(value, workspace):
    """rustc's relative spans name sources relative to its Cargo workspace."""
    if isinstance(value, dict):
        source = value.get('file_name')
        if isinstance(source, str) and not Path(source).is_absolute():
            candidate = workspace / source
            if candidate.is_file():
                value['file_name'] = str(candidate.resolve())
        for child in value.values():
            resolve_locations(child, workspace)
    elif isinstance(value, list):
        for child in value:
            resolve_locations(child, workspace)
