"""Build compiler prerequisites before courts; publish raw attributed outcomes."""
from enum import Enum
import hashlib
import json
import os
from pathlib import Path

from compile_batch import prepare_environment, render_manifests, target_directory
from compiler_audiences import group, sources as fixture_sources
from fixture_graph import configured_inputs, snapshot_audiences, snapshot_inputs
from workspace import WORKSPACES
from diagnostic_flags import FLAGS as DIAGNOSTIC_FLAGS


class CompilerScope(Enum):
    NONE = 'none'
    UNITS = 'units'
    FIXTURES = 'fixtures'
    ALL = 'all'


def normalized_source_key(path):
    value = str(Path(path).resolve()).replace('\\', '/')
    if value.startswith('//?/UNC/'):
        value = '//' + value[8:]
    elif value.startswith('//?/'):
        value = value[4:]
    return value.lower() if os.name == 'nt' else value


def environments(repository, workspace):
    repository = Path(repository).resolve()
    manifest = repository / WORKSPACES[workspace]
    orphaned = []
    snapshots = snapshot_inputs(manifest, orphaned)
    if workspace == 'store':
        registry = Path(__file__).with_name('store_compiler_inputs.json')
        rows = configured_inputs(repository, registry)
        contracts = snapshot_audiences(repository, registry)
    else:
        rows = {'production': {'identity': 'production', 'profile': 'test', 'packages': []}}
        contracts = {}
    records = {name: declared_audiences(row['packages']) for name, row in rows.items()}
    for snapshot in snapshots:
        contract = contracts.get(snapshot['source'])
        name = contract['environment'] if contract else 'certification' if workspace == 'store' else 'production'
        records[name].append({'dependencies': contract['dependencies'] if contract else snapshot['dependencies'],
                              'target_dependencies': contract.get('target_dependencies', {}) if contract else snapshot['target_dependencies'],
                              'sources': [snapshot['source']],
                              'audience': contract['audience'] if contract else snapshot['package']})
    absent_contracts = set(contracts) - {snapshot['source'] for snapshot in snapshots}
    if absent_contracts:
        raise ValueError(f'compiler audience contracts have no snapshot input: {sorted(map(str, absent_contracts))}')
    if 'production' in rows:
        rows['production']['reject_test_features'] = True
    for name, row in rows.items():
        row['packages'] = group(records[name])
    return rows, orphaned


def declared_audiences(packages):
    return [{'dependencies': package['dependencies'], 'sources': package['sources'],
             'target_dependencies': package['target_dependencies'],
             'audience': ', '.join(package['audiences'])} for package in packages]


def recipe_identity(repository, manifest, environment):
    manifests = {path: text.replace(Path(repository).as_posix(), '<workspace>')
                 for path, text in render_manifests(environment, manifest).items()}
    recipe = json.dumps({'manifests': manifests, 'profile': environment['profile'],
                         'diagnostic_flags': DIAGNOSTIC_FLAGS}, sort_keys=True)
    return hashlib.sha256(recipe.encode()).hexdigest()


def unit_inputs(output):
    root = Path(output).resolve() / 'unit-inputs'
    dependency = root / 'dependency'
    (dependency / 'src').mkdir(parents=True, exist_ok=True)
    (dependency / 'Cargo.toml').write_text('[package]\nname="sealed-dependency"\nversion="0.0.0"\nedition="2021"\n[workspace]\n', encoding='utf-8')
    (dependency / 'src/lib.rs').write_text('pub(crate) fn private() {}\n', encoding='utf-8')
    (dependency / 'build.rs').write_text(
        'fn main() { assert_eq!(std::env::var("OPT_LEVEL").unwrap(), "0"); '
        'let flags=std::env::var("CARGO_ENCODED_RUSTFLAGS").unwrap(); '
        'assert!(flags.contains("--verbose")); assert!(flags.contains("dead_code")); }\n', encoding='utf-8')
    fixtures = root / 'fixtures'
    fixtures.mkdir(parents=True, exist_ok=True)
    for name in ('private_a', 'private_b'):
        (fixtures / f'{name}.rs').write_text('fn main() { sealed_dependency::private(); }\n', encoding='utf-8')
    (fixtures / 'unresolved.rs').write_text('use absent_dependency::private;\nfn main() {}\n', encoding='utf-8')
    return root, {'identity': 'compiler-unit', 'environment_class': 'production',
                  'reject_test_features': True,
                  'profile': 'diagnostic-test', 'packages': group([{
                      'dependencies': {'sealed-dependency': {'path': dependency.as_posix()}},
                      'sources': sorted(fixtures.glob('*.rs')), 'audience': 'compiler-reader-unit-inputs'}])}


def prepare(repository, workspace, output, cargo_runner, scope=CompilerScope.ALL):
    repository, output = Path(repository).resolve(), Path(output).resolve()
    output.mkdir(parents=True, exist_ok=True)
    if scope is CompilerScope.NONE:
        return {}
    manifest = repository / WORKSPACES[workspace]
    rows, orphaned = environments(repository, workspace) if scope in (CompilerScope.ALL, CompilerScope.FIXTURES) else ({}, [])
    (output / 'orphan-snapshots.json').write_text(json.dumps(orphaned, indent=2) + '\n', encoding='utf-8')
    environment = {}
    if workspace == 'store' and scope in (CompilerScope.ALL, CompilerScope.UNITS):
        root, row = unit_inputs(output)
        if scope is CompilerScope.ALL:
            production = rows['production']
            if production['profile'] != row['profile']:
                raise ValueError('compiler-unit and production fixtures declare different compiler profiles')
            production['packages'] = group(declared_audiences(production['packages'] + row['packages']))
        else:
            rows[row['identity']] = row
        environment['WORTH_PREBUILT_COMPILER_UNIT_ROOT'] = str(root)
    outcomes = {}
    seen = set()
    for row in rows.values():
        for source in fixture_sources(row):
            key = normalized_source_key(source)
            if key in seen:
                raise ValueError(f'fixture appears in two compiler environments: {source}')
            seen.add(key)
    for name, row in rows.items():
        if not fixture_sources(row):
            continue
        observed = prepare_environment(manifest, row, output / name, cargo_runner)
        identity = recipe_identity(repository, manifest, row)
        target = target_directory(manifest, cargo_runner)
        for source, result in observed['fixtures'].items():
            key = normalized_source_key(source)
            if key in outcomes:
                raise ValueError(f'fixture appears in two compiler environments: {source}')
            outcomes[key] = {'environment': row.get('environment_class', name),
                             'environment_identity': identity, 'shared_target_root': str(target.resolve()),
                             'audience_identity': result['audience_identity'],
                             'fate': result['fate'], 'messages': result['messages']}
    path = output / 'compiler-outcomes.json'
    path.write_text(json.dumps({'schema': 1, 'fixtures': outcomes}, indent=2) + '\n', encoding='utf-8')
    environment['WORTH_PREBUILT_COMPILER_OUTCOMES'] = str(path)
    return environment
