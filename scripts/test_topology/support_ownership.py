"""Assign every moved source/data file one physical owner from its court readers."""
from collections import defaultdict
import glob
from pathlib import Path
import re

from module_tree import literal_references, source_files, test_names
from rust_source import code
from court_partition import has_tests


def referenced_files(pattern):
    if '*' in str(pattern):
        files = [Path(p).resolve() for p in glob.glob(str(pattern), recursive=True) if Path(p).is_file()]
        files += [p.with_suffix('.stderr') for p in files if p.with_suffix('.stderr').is_file()]
        return sorted(set(files))
    if pattern.is_dir():
        return sorted(p for p in pattern.rglob('*') if p.is_file())
    return [pattern] if pattern.is_file() else []


def footprint(unit, compilation_sources):
    nodes = [unit['node']] if unit['own_only'] else list(unit['node'].walk())
    files = {n.source for n in nodes}
    for node in nodes:
        for reference in literal_references(node, unit['owner']['root']):
            files.update(p for p in referenced_files(reference['path'])
                         if p.suffix == '.rs' and p not in compilation_sources)
    return files


def relevant_support(unit):
    text = '\n'.join(n.text for n in unit['node'].walk())
    ancestors = unit['ancestors']
    roots = []
    for ancestor in ancestors + ([unit['node']] if unit['own_only'] else []):
        text += '\n' + ancestor.own_text()
        for sibling in ancestor.children:
            if sibling in ancestors or sibling is unit['node']:
                continue
            if has_tests(sibling):
                continue
            # Shared root mounts often route through crate aliases. Preserve siblings
            # when explicitly named; parent glob imports conservatively retain them.
            if re.search(r'\b' + re.escape(sibling.names[-1]) + r'\b', code(text)) or 'use crate :: *' in code(text):
                roots.append(sibling)
    return roots


def allocate(units, owners, stem, workspace, repository):
    readers, source_readers, records, support_nodes = defaultdict(set), defaultdict(set), {}, {}
    for unit in units:
        owner = unit['owner']
        nodes = list(unit['node'].walk()) if not unit['own_only'] else [unit['node']]
        unit['support'] = relevant_support(unit)
        for support in unit['support']:
            nodes.extend(support.walk())
        for node in nodes:
            path = node.source
            if not path.is_relative_to(owner['root'] / 'tests'):
                continue  # Examples and src are mounted in place, never moved.
            readers[path].add(unit['court'])
            source_readers[path].add(unit['court'])
            support_nodes[path] = node
            records[path] = {'kind': 'rust', 'package': owner['cargo']['name']}
            for reference in literal_references(node, owner['root']):
                pattern = reference['path']
                assets = referenced_files(pattern)
                for asset in assets:
                    if asset.is_relative_to(owner['root'] / 'tests'):
                        readers[asset].add(unit['court'])
                        records.setdefault(asset, {'kind': 'asset', 'package': owner['cargo']['name']})
    # Unreferenced fixture siblings (stderr, golden files, nested Cargo projects)
    # share their closest referenced directory's readers. All tests/ paths leave.
    for owner in owners:
        population = {u['court'] for u in units if u['owner'] is owner}
        if not population:
            continue
        tests = owner['root'] / 'tests'
        for path in owner['files']:
            if path in records:
                continue
            parent = path.parent
            matched = set()
            while parent.is_relative_to(tests) and not matched:
                matched = set().union(*(rs for p, rs in readers.items() if p.parent == parent))
                parent = parent.parent
            readers[path].update(matched or population)
            records[path] = {'kind': 'asset', 'package': owner['cargo']['name'],
                             'unreferenced': True}
    assignments, kits = {}, {}
    owner_lookup = {o['cargo']['name']: o for o in owners}
    for path in sorted(records):
        record = records[path]
        # A source-code file's data observers do not become compilation owners.
        # Readiness inventories often inspect other courts' source as data.
        courts = sorted(source_readers[path] if path in source_readers else readers[path])
        if not courts:
            continue
        owner = owner_lookup[record['package']]
        relative = path.relative_to(owner['root'] / 'tests')
        if len(courts) == 1:
            name = courts[0]
            base = workspace / 'crates' / name / 'tests' / name.replace('-', '_')
            destination = base / 'sources' / record['package'].replace('-', '_') / relative
        else:
            # A family per product keeps unrelated fixture authority separate.
            family = record['package'].removeprefix(stem + '-').removeprefix('worth-')
            name = stem + '-testkit-' + family
            kits.setdefault(name, {'name': name, 'readers': set(), 'files': [], 'nodes': []})
            kits[name]['readers'].update(courts)
            kits[name]['files'].append(path)
            is_source = record['kind'] == 'rust'
            destination = workspace / 'crates' / name / ('src' if is_source else 'assets') / relative
            if is_source:
                kits[name]['nodes'].append(support_nodes[path])
        assignments[path] = {'destination': destination, 'owner': name, 'readers': courts, **record}
    return assignments, kits
