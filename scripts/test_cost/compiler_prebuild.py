"""Build compiler prerequisites before courts; publish raw attributed outcomes."""
import copy
from enum import Enum
import hashlib
import json
import os
from pathlib import Path

from compile_batch import prepare_environment, render_manifest, target_directory
from fixture_graph import configured_inputs, dependency_manifest, merge_dependencies, snapshot_inputs
from workspace import WORKSPACES


class CompilerScope(Enum):
    NONE = 'none'
    UNITS = 'units'
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
        rows = configured_inputs(repository, Path(__file__).with_name('store_compiler_inputs.json'))
    else:
        rows = {'production': {'identity': 'production', 'profile': 'dev', 'dependencies': {}, 'sources': []}}
    for snapshot in snapshots:
        name = 'production' if workspace != 'store' or snapshot['source'].name == 'certification_authority_is_absent.rs' else 'certification'
        row = rows[name]
        row['sources'].append(snapshot['source'])
        merge_dependencies(row['dependencies'], snapshot['dependencies'])
    if workspace == 'store':
        # This fixture intentionally imports a disabled root facade. It cannot
        # borrow a certification graph through the court's own testkit.
        merge_dependencies(rows['production']['dependencies'], {
            'worth-store': {'path': (manifest.parent / 'crates/worth-store').as_posix()}})
        rows['production']['reject_test_features'] = True
    for row in rows.values():
        row['sources'] = sorted(set(row['sources']))
        row['dependency_manifest'] = dependency_manifest(row['dependencies'])
    return rows, orphaned


def unit_inputs(output):
    root = Path(output).resolve() / 'unit-inputs'
    dependency = root / 'dependency'
    (dependency / 'src').mkdir(parents=True, exist_ok=True)
    (dependency / 'Cargo.toml').write_text('[package]\nname="sealed-dependency"\nversion="0.0.0"\nedition="2021"\n[workspace]\n', encoding='utf-8')
    (dependency / 'src/lib.rs').write_text('pub(crate) fn private() {}\n', encoding='utf-8')
    (dependency / 'build.rs').write_text('fn main() { assert_eq!(std::env::var("OPT_LEVEL").unwrap(), "0"); }\n', encoding='utf-8')
    fixtures = root / 'fixtures'
    fixtures.mkdir(parents=True, exist_ok=True)
    for name in ('private_a', 'private_b'):
        (fixtures / f'{name}.rs').write_text('fn main() { sealed_dependency::private(); }\n', encoding='utf-8')
    (fixtures / 'unresolved.rs').write_text('use absent_dependency::private;\nfn main() {}\n', encoding='utf-8')
    return root, {'identity': 'compiler-unit', 'environment_class': 'production',
                  'profile': 'diagnostic-test', 'sources': sorted(fixtures.glob('*.rs')),
                  'dependency_manifest': dependency_manifest({'sealed-dependency': {'path': dependency.as_posix()}})}


def prepare(repository, workspace, output, cargo_runner, scope=CompilerScope.ALL):
    repository, output = Path(repository).resolve(), Path(output).resolve()
    output.mkdir(parents=True, exist_ok=True)
    if scope is CompilerScope.NONE:
        return {}
    manifest = repository / WORKSPACES[workspace]
    rows, orphaned = environments(repository, workspace) if scope is CompilerScope.ALL else ({}, [])
    (output / 'orphan-snapshots.json').write_text(json.dumps(orphaned, indent=2) + '\n', encoding='utf-8')
    environment = {}
    if workspace == 'store':
        root, row = unit_inputs(output)
        rows[row['identity']] = row
        environment['WORTH_PREBUILT_COMPILER_UNIT_ROOT'] = str(root)
    outcomes = {}
    for name, row in rows.items():
        if not row['sources']:
            continue
        observed = prepare_environment(manifest, row, output / name, cargo_runner)
        rendered = render_manifest(row, manifest).replace(repository.as_posix(), '<workspace>')
        identity = hashlib.sha256(rendered.encode()).hexdigest()
        target = target_directory(manifest, cargo_runner)
        for source, result in observed['fixtures'].items():
            key = normalized_source_key(source)
            if key in outcomes:
                raise ValueError(f'fixture appears in two compiler environments: {source}')
            outcomes[key] = {'environment': row.get('environment_class', name),
                             'environment_identity': identity, 'shared_target_root': str(target.resolve()),
                             'fate': result['fate'], 'messages': result['messages']}
    path = output / 'compiler-outcomes.json'
    path.write_text(json.dumps({'schema': 1, 'fixtures': outcomes}, indent=2) + '\n', encoding='utf-8')
    environment['WORTH_PREBUILT_COMPILER_OUTCOMES'] = str(path)
    return environment
