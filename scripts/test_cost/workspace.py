"""Resolve a workspace's owned source without traversing build directories."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tomllib

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'test_topology'))
from filesystem_path import filesystem_path
from module_tree import read_tree
from rust_items import items


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


def sources(manifest):
    result = []
    for package in members(manifest):
        name = package['name']
        root = package['manifest'].parent
        support = any(s in name for s in ('-court-', '-testkit', '-test-support', '-certification'))
        declaration = tomllib.loads(package['manifest'].read_text(encoding='utf-8'))
        roots = []
        for family, folder in [('lib', 'src'), ('bin', 'src'), ('test', 'tests'),
                               ('bench', 'benches'), ('example', 'examples')]:
            explicit = declaration.get(family, [])
            explicit = [explicit] if isinstance(explicit, dict) else explicit
            for target in explicit:
                if target.get('path'):
                    roots.append((root / target['path'], support or family not in ('lib', 'bin')))
            if family in ('lib', 'bin'):
                default = root / folder / ('lib.rs' if family == 'lib' else 'main.rs')
                if filesystem_path(default).is_file():
                    roots.append((default, support))
            elif declaration['package'].get('auto' + family + 's', True):
                roots += [(path, True) for path in (root / folder).glob('*.rs')]
                roots += [(path, True) for path in (root / folder).glob('*/main.rs')]
        spans = {}

        def visit(node, testing):
            testing = testing or bool(node.declaration and 'test' in node.declaration.attributes)
            if testing:
                spans.setdefault(node.source, []).append((node.offset, node.offset + len(node.text)))
            else:
                for item in items(node.text):
                    header = node.text[item.start:item.body[0] if item.body else item.end]
                    if item.test or ('cfg' in header and 'test' in header):
                        spans.setdefault(node.source, []).append((node.offset + item.start, node.offset + item.end))
            for child in node.children:
                visit(child, testing)

        for path, testing in sorted(set(roots)):
            visit(read_tree(path), testing)
        for path, selected in sorted(spans.items()):
            text = filesystem_path(path).read_text(encoding='utf-8-sig')
            masked = ['\n' if character == '\n' else ' ' for character in text]
            for start, end in selected:
                masked[start:end] = text[start:end]
            result.append((package, path, ''.join(masked)))
    return result


def metadata(manifest, destination):
    command = ['cargo', 'metadata', '--format-version', '1', '--no-deps',
               '--manifest-path', str(Path(manifest).resolve())]
    value = subprocess.check_output(command, text=True, encoding='utf-8')
    json.loads(value)
    Path(destination).write_text(value, encoding='utf-8')
    return json.loads(value)
