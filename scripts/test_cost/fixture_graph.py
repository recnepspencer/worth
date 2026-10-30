"""Derive compiler-fixture dependencies from their real Cargo audience surface."""
import copy
import json
from pathlib import Path
import tomllib

from workspace import members
from filesystem_path import filesystem_path
from manifest_edit import toml_value
from compiler_audiences import group


def dependency_surface(manifest, workspace_manifest):
    data = tomllib.loads(Path(manifest).read_text(encoding='utf-8'))
    workspace = tomllib.loads(Path(workspace_manifest).read_text(encoding='utf-8'))['workspace']
    return resolved_dependencies(data, workspace, manifest, workspace_manifest)


def dependency_targets(manifest, workspace_manifest):
    data = tomllib.loads(Path(manifest).read_text(encoding='utf-8'))
    workspace = tomllib.loads(Path(workspace_manifest).read_text(encoding='utf-8'))['workspace']
    return {target: resolved_dependencies(tables, workspace, manifest, workspace_manifest)
            for target, tables in data.get('target', {}).items()
            if tables.get('dependencies') or tables.get('dev-dependencies')}


def resolved_dependencies(data, workspace, manifest, workspace_manifest):
    result = {}
    for family in ('dependencies', 'dev-dependencies'):
        for alias, raw in data.get(family, {}).items():
            spec = {'version': raw} if isinstance(raw, str) else copy.deepcopy(raw)
            base = Path(manifest).parent
            if spec.pop('workspace', False):
                inherited = workspace['dependencies'][alias]
                inherited = {'version': inherited} if isinstance(inherited, str) else copy.deepcopy(inherited)
                features = sorted(set(inherited.get('features', [])) | set(spec.get('features', [])))
                inherited.update(spec)
                if features:
                    inherited['features'] = features
                spec, base = inherited, Path(workspace_manifest).parent
            if 'path' in spec:
                spec['path'] = str((base / spec['path']).resolve()).replace('\\', '/')
            spec.pop('optional', None)
            if not spec.get('features'):
                spec.pop('features', None)
            merge_dependencies(result, {alias: spec})
    return result


def merge_dependencies(destination, incoming):
    for name, value in incoming.items():
        value = copy.deepcopy(value)
        if name not in destination:
            destination[name] = value
            continue
        existing = destination[name]
        for field in ('path', 'package', 'git', 'registry'):
            if existing.get(field) != value.get(field):
                raise ValueError(f'compiler fixture dependency collision: {name}: {field}')
        if existing.get('version', '').lstrip('^') != value.get('version', '').lstrip('^'):
            raise ValueError(f'compiler fixture version collision: {name}')
        features = sorted(set(existing.get('features', [])) | set(value.get('features', [])))
        if features:
            existing['features'] = features
        if existing.get('default-features', True) or value.get('default-features', True):
            existing.pop('default-features', None)


def snapshot_inputs(workspace_manifest, orphaned=None):
    """Find real snapshots, then follow each program's actual module imports."""
    rows = []
    packages = members(workspace_manifest)
    owners = {package['name'].replace('-', '_'): package for package in packages}
    surfaces = {name: dependency_surface(package['manifest'], workspace_manifest)
                for name, package in owners.items()}
    targets = {name: dependency_targets(package['manifest'], workspace_manifest)
               for name, package in owners.items()}
    for package in packages:
        root = package['manifest'].parent
        surface = surfaces[package['name'].replace('-', '_')]
        for family in ('tests', 'assets'):
            directory = filesystem_path(root / family)
            for snapshot in sorted(directory.rglob('*.stderr')):
                io_source = snapshot.with_suffix('.rs')
                if not io_source.is_file():
                    if orphaned is not None:
                        orphaned.append(str(snapshot))
                    continue
                source = root / family / io_source.relative_to(directory)
                relative = source.relative_to(root).parts
                producer = package
                if len(relative) >= 4 and relative[0] == 'tests' and relative[2] == 'sources':
                    if relative[3] not in owners:
                        raise ValueError(f'fixture producer is not a workspace package: {source}')
                    producer = owners[relative[3]]
                elif family == 'assets' and package['name'].startswith('worth-testkit-'):
                    name = 'worth_' + package['name'].removeprefix('worth-testkit-').replace('-', '_')
                    if name not in owners:
                        raise ValueError(f'asset fixture producer is not a workspace package: {source}')
                    producer = owners[name]
                # Absence claims and diagnostic suggestions use the complete
                # declared normal + dev audience, never an import-name subset.
                dependencies = copy.deepcopy(surfaces[producer['name'].replace('-', '_')])
                own = next((spec for alias, spec in surface.items()
                            if spec.get('package', alias) == producer['name']), None)
                dependencies[producer['name']] = copy.deepcopy(own) if own else {
                    'path': producer['manifest'].parent.resolve().as_posix()}
                rows.append({'source': source.resolve(), 'package': producer['name'],
                             'dependencies': dependencies,
                             'target_dependencies': copy.deepcopy(targets[producer['name'].replace('-', '_')])})
    return rows


def dependency_manifest(dependencies, target_dependencies=None):
    text = '[dependencies]\n' + ''.join(f'{json.dumps(name)}={toml_value(spec)}\n'
                                      for name, spec in sorted(dependencies.items()))
    for target, table in sorted((target_dependencies or {}).items()):
        text += f'\n[target.{json.dumps(target)}.dependencies]\n'
        text += ''.join(f'{json.dumps(name)}={toml_value(spec)}\n' for name, spec in sorted(table.items()))
    return text


def configured_inputs(repository, path):
    config = json.loads(Path(path).read_text(encoding='utf-8'))
    if config['schema'] != 2:
        raise ValueError('unsupported compiler input registry')
    result = {}
    for environment in config['environments']:
        records = [{'dependencies': dependency_records(repository, package['dependencies']),
                    'target_dependencies': {target: dependency_records(repository, dependencies)
                                            for target, dependencies in package.get('target_dependencies', {}).items()},
                    'sources': [repository / source for source in package['sources']],
                    'audience': package['audience']} for package in environment['packages']]
        result[environment['identity']] = {'identity': environment['identity'],
                                          'profile': environment['profile'],
                                          'packages': group(records)}
    return result


def dependency_records(repository, rows):
    dependencies = {}
    for row in rows:
        if ('path' in row) == ('version' in row):
            raise ValueError(f"compiler dependency requires exactly one path or version: {row['name']}")
        spec = ({'path': str((repository / row['path']).resolve()).replace('\\', '/')}
                if 'path' in row else {'version': row['version']})
        if row['features']:
            spec['features'] = row['features']
        if not row['default_features']:
            spec['default-features'] = False
        dependencies[row['name']] = spec
    return dependencies


def snapshot_audiences(repository, path):
    config = json.loads(Path(path).read_text(encoding='utf-8'))
    if config['schema'] != 2:
        raise ValueError('unsupported compiler input registry')
    result = {}
    for row in config.get('snapshot_audiences', []):
        source = (repository / row['source']).resolve()
        if source in result:
            raise ValueError(f'duplicate snapshot audience contract: {source}')
        result[source] = {'environment': row['environment'], 'audience': row['audience'],
                          'dependencies': dependency_records(repository, row['dependencies'])}
    return result
