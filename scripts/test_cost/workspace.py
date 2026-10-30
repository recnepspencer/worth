"""Resolve a workspace's owned source without traversing build directories."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tomllib

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'test_topology'))
from filesystem_path import filesystem_path
from source_roots import roots as cargo_source_roots
from source_documents import documents


WORKSPACES = {'root': 'Cargo.toml', 'query': 'workspaces/worth-query/Cargo.toml',
              'bank': 'workspaces/worth-query-bank-world/Cargo.toml',
              'store': 'workspaces/worth-store/Cargo.toml'}


def repository_root(path):
    for parent in (path.resolve(), *path.resolve().parents):
        if (parent / '.git').exists():
            return parent
    raise ValueError(f'no repository above {path}')


def members(manifest):
    manifest = Path(manifest).resolve()
    data = tomllib.loads(manifest.read_text(encoding='utf-8'))
    paths = {manifest} if 'package' in data else set()
    workspace = data.get('workspace', {})
    excluded = {p.resolve() for pattern in workspace.get('exclude', [])
                for p in manifest.parent.glob(pattern)}
    for pattern in workspace.get('members', []):
        directories = list(manifest.parent.glob(pattern))
        if not directories and not any(character in pattern for character in '*?['):
            raise ValueError(f'declared workspace member is missing: {pattern}')
        for directory in directories:
            path = directory / 'Cargo.toml'
            if path.is_file() and directory.resolve() not in excluded:
                paths.add(path.resolve())
    result = []
    for path in sorted(paths):
        package = tomllib.loads(path.read_text(encoding='utf-8')).get('package')
        if package:
            result.append({'name': package['name'], 'manifest': path})
    if not result:
        raise ValueError(f'workspace has no packages: {manifest}')
    return result


def sources(manifest, packages=None):
    result = []
    declaration = tomllib.loads(Path(manifest).read_text(encoding='utf-8'))
    inherited_edition = declaration.get('workspace', {}).get('package', {}).get('edition')
    for package in members(manifest):
        name = package['name']
        if packages is not None and name not in packages:
            continue
        root = package['manifest'].parent
        support = any(s in name for s in ('-court-', '-testkit', '-test-support', '-certification'))
        declaration = tomllib.loads(package['manifest'].read_text(encoding='utf-8'))
        roots = [(row['source'], support or row['family'] not in ('lib', 'bin'))
                 for row in cargo_source_roots(root, declaration, inherited_edition)]
        result.extend(documents(package, sorted(set(roots))))
    return result


def metadata(manifest, destination):
    command = ['cargo', 'metadata', '--format-version', '1', '--no-deps',
               '--manifest-path', str(Path(manifest).resolve())]
    value = subprocess.check_output(command, text=True, encoding='utf-8')
    json.loads(value)
    Path(destination).write_text(value, encoding='utf-8')
    return json.loads(value)
