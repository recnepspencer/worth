"""Derive compiler-fixture dependencies from their real Cargo audience surface."""
import copy
import json
from pathlib import Path
import tomllib

from workspace import members
from filesystem_path import filesystem_path
from manifest_edit import toml_value
from module_tree import read_tree
from rust_source import crate_uses


def dependency_surface(manifest, workspace_manifest):
    data = tomllib.loads(Path(manifest).read_text(encoding='utf-8'))
    workspace = tomllib.loads(Path(workspace_manifest).read_text(encoding='utf-8'))['workspace']
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
            result[alias] = spec
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
    surfaces = {package['name'].replace('-', '_'): dependency_surface(package['manifest'], workspace_manifest)
                for package in packages}
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
                # A moved fixture retains its producer's declared dependencies.
                # The court's explicit audience configuration takes precedence.
                fixture_surface = surface
                if len(relative) >= 4 and relative[0] == 'tests' and relative[2] == 'sources':
                    fixture_surface = dict(surfaces.get(relative[3], {}), **surface)
                tree = read_tree(source)
                text = '\n'.join(node.own_text() for node in tree.walk())
                names = crate_uses(text, {alias.replace('-', '_') for alias in fixture_surface})
                needed = {alias: spec for alias, spec in fixture_surface.items() if alias.replace('-', '_') in names}
                rows.append({'source': source.resolve(), 'package': package['name'], 'dependencies': needed})
    return rows


def dependency_manifest(dependencies):
    return '[dependencies]\n' + ''.join(f'{json.dumps(name)}={toml_value(spec)}\n'
                                       for name, spec in sorted(dependencies.items()))


def configured_inputs(repository, path):
    config = json.loads(Path(path).read_text(encoding='utf-8'))
    if config['schema'] != 1:
        raise ValueError('unsupported compiler input registry')
    result = {}
    for environment in config['environments']:
        dependencies = {}
        for row in environment['dependencies']:
            spec = {'path': str((repository / row['path']).resolve()).replace('\\', '/')}
            if row['features']:
                spec['features'] = row['features']
            if not row['default_features']:
                spec['default-features'] = False
            dependencies[row['name']] = spec
        result[environment['identity']] = {'identity': environment['identity'],
                                          'profile': environment['profile'],
                                          'dependencies': dependencies,
                                          'sources': [repository / p for p in environment['sources']]}
    return result
