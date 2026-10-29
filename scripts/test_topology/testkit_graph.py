"""Preserve support module ancestry while compiling each shared file once."""
from collections import defaultdict
import os

from source_rewrite import file_alias


def prepare(kits, assignments, owners, ceiling):
    owner_by_name = {o['cargo']['name']: o for o in owners}
    # Split shared support along real directories, not numbered line buckets.
    for name, kit in list(kits.items()):
        rust = [p for p in kit['files'] if p.suffix == '.rs']
        if sum(len(p.read_text(encoding='utf-8').splitlines()) for p in rust) <= ceiling:
            continue
        del kits[name]
        areas = {}
        for path in kit['files']:
            destination = assignments[path]['destination']
            suffix = destination.relative_to(next(p for p in destination.parents if p.name == name))
            areas[path] = tuple(p for p in suffix.with_suffix('').parts[1:]
                                if p not in ('fixtures', 'support'))
        groups = []
        def partition(paths, depth):
            lines = sum(len(p.read_bytes().splitlines()) + 2 for p in paths if p.suffix == '.rs')
            prefix = tuple(os.path.commonprefix([list(areas[p]) for p in paths]))
            if lines <= ceiling or len(paths) == 1:
                groups.append((prefix, paths))
                return
            buckets = defaultdict(list)
            for path in paths:
                buckets[areas[path][:depth]].append(path)
            if len(buckets) == 1:
                if all(len(areas[p]) <= depth for p in paths):
                    groups.append((prefix, paths))
                else:
                    partition(paths, depth + 1)
            else:
                for area, population in sorted(buckets.items()):
                    partition(population, depth + 1)
        partition(kit['files'], 1)
        for area, paths in groups:
            child_name = name + '-' + '-'.join(area).replace('_', '-')
            child = {'name': child_name, 'files': paths, 'nodes': [], 'readers': set()}
            for path in paths:
                assignment = assignments[path]
                old_root = next(p for p in assignment['destination'].parents if p.name == name)
                new_root = old_root.with_name(child_name)
                assignment['destination'] = new_root / assignment['destination'].relative_to(old_root)
                assignment['owner'] = child_name
                child['readers'].update(assignment['readers'])
            child['nodes'] = [n for n in kit['nodes'] if n.source in paths]
            kits[child_name] = child
    aliases = {}
    for name, kit in sorted(kits.items()):
        nodes = {node.source: node for node in kit['nodes']}
        incoming = {child.source for node in nodes.values() for child in node.children
                    if child.source in nodes and child.source != node.source}
        roots = [nodes[p] for p in sorted(set(nodes) - incoming)]
        kit['roots'] = []

        def mount(node, route):
            if node.source in aliases:
                return
            aliases[node.source] = route
            for child in node.children:
                if child.source in nodes and child.source != node.source:
                    mount(child, route + '::' + child.names[-1])

        for node in roots:
            if node.source in aliases:
                continue
            owner = owner_by_name[assignments[node.source]['package']]
            route = file_alias(node.source, owner)
            kit['roots'].append((node, route))
            mount(node, route)
        if set(nodes) - set(aliases):
            raise ValueError('shared module graph has no owning root: ' + name)
    return aliases
