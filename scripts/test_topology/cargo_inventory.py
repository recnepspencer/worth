"""Cargo target inventory and dependency declarations, using offline metadata only."""
import json
from pathlib import Path
import subprocess
import tomllib

from module_tree import read_tree
from rust_source import crate_uses
from frontier_adapter import package_roles


def metadata(manifest):
    command = ['cargo', 'metadata', '--offline', '--format-version', '1',
               '--manifest-path', str(manifest)]
    result = subprocess.run(command, stdout=subprocess.PIPE, text=True, encoding='utf-8')
    if result.returncode:
        raise ValueError(subprocess.list2cmdline(command) + '\nexit code ' + str(result.returncode))
    return json.loads(result.stdout)


def role(package, members):
    if package['id'] not in members:
        return 'external'
    name = package['name']
    if '-court-' in name:
        return 'court'
    if '-testkit' in name:
        return 'testkit'
    if 'fixtures' in Path(package['manifest_path']).parts:
        return 'fixture'
    return 'product'


def inventory(data, root):
    result = []
    members = set(data['workspace_members'])
    roles = package_roles(data)
    for package in sorted(data['packages'], key=lambda p: p['name']):
        if roles.get(package['id']) not in ('product', 'testkit', 'tool'):
            continue
        package_root = Path(package['manifest_path']).resolve().parent
        targets = []
        for target in sorted(package['targets'], key=lambda t: t['name']):
            if 'test' in target['kind']:
                tree = read_tree(Path(target['src_path']))
                targets.append({'cargo': target, 'tree': tree})
        tests = package_root / 'tests'
        files = sorted(p for p in tests.rglob('*') if p.is_file()) if tests.exists() else []
        result.append({'cargo': package, 'root': package_root, 'targets': targets,
                       'role': roles[package['id']],
                       'files': files,
                       'manifest': tomllib.loads(Path(package['manifest_path']).read_text(encoding='utf-8'))})
    return result


def aliases(package):
    return {d.get('rename') or d['name'].replace('-', '_') for d in package['dependencies']
            if d['kind'] != 'build'}


def dependency_rows(package, used, own=False):
    rows = []
    if own:
        rows.append({'alias': package['name'], 'package': package['name'],
                     'path': str(Path(package['manifest_path']).parent), 'features': [],
                     'default_features': True, 'target': None, 'requirement': None})
    for dependency in package['dependencies']:
        alias = dependency.get('rename') or dependency['name']
        if dependency['kind'] == 'build' or alias.replace('-', '_') not in used:
            continue
        rows.append({'alias': alias, 'package': dependency['name'], 'path': dependency.get('path'),
                     'features': sorted(dependency['features']),
                     'default_features': dependency['uses_default_features'],
                     'target': dependency['target'], 'requirement': dependency['req'],
                     'registry': dependency.get('registry'), 'source': dependency.get('source')})
    return rows


def dev_drops(owner):
    # Keep dependencies referenced from *any* src code, examples or benches, including
    # test-mounted files and macro bodies. This conservative scan cannot drop a unit input.
    package = owner['cargo']
    candidates = {d.get('rename') or d['name'] for d in package['dependencies'] if d['kind'] == 'dev'}
    retained = set()
    for folder in ('src', 'examples', 'benches'):
        for source in sorted((owner['root'] / folder).rglob('*.rs')):
            used = crate_uses(source.read_text(encoding='utf-8'), {n.replace('-', '_') for n in candidates})
            retained.update(n for n in candidates if n.replace('-', '_') in used)
    return sorted(candidates - retained)
