"""Resolve a workspace's owned source without traversing build directories."""
import json
import os
from pathlib import Path
import subprocess
import tomllib


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
        for directory in manifest.parent.glob(pattern):
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


def sources(manifest):
    result = []
    for package in members(manifest):
        name = package['name']
        root = package['manifest'].parent
        support = any(s in name for s in ('-court-', '-testkit', '-test-support', '-certification'))
        for family in ('src', 'tests', 'benches', 'examples'):
            directory = root / family
            io_directory = Path('\\\\?\\' + str(directory)) if os.name == 'nt' else directory
            for io_path in sorted(io_directory.rglob('*.rs')):
                path = directory / io_path.relative_to(io_directory)
                text = io_path.read_text(encoding='utf-8-sig')
                # Fixtures are compiler inputs, not programs which run Cargo.
                fixture = any(part in ('sources', 'ui', 'fixtures', 'compile_fail_proofs')
                              for part in path.relative_to(root).parts)
                testing = support or family != 'src' or 'cfg(test)' in text or 'test' in path.stem
                if testing and not fixture:
                    result.append((package, path, text))
    return result


def metadata(manifest, destination):
    command = ['cargo', 'metadata', '--format-version', '1', '--no-deps',
               '--manifest-path', str(Path(manifest).resolve())]
    value = subprocess.check_output(command, text=True, encoding='utf-8')
    json.loads(value)
    Path(destination).write_text(value, encoding='utf-8')
    return json.loads(value)
