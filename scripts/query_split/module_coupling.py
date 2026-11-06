#!/usr/bin/env python3
"""Coupling between a module's top-level children, with re-exports resolved to definitions.

Usage:
    python scripts/query_split/module_coupling.py report <lib.rs> <module::path> --out <dir>
    python scripts/query_split/module_coupling.py baseline <lib.rs> <module::path> --out <layering.json>
    python scripts/query_split/module_coupling.py check <lib.rs> <layering.json> [<layering.json> ...]

`baseline` freezes the current layering, its upward edges and the inherent impls that sit outside
their type's top-level child (a crate split refuses those). `check` is the ratchet:
it refuses a new upward edge, an unplaced new module, or a baseline edge that no longer
exists (shrink the baseline in the same change), so the cycle can only get smaller.
Each layering file names its own `parent`, so one run checks several modules (for example
`domain_computation` and `domain_computation::primary_graph`) against one parsed library.
A layering whose `upward` list is empty is strict: any upward edge is refused.

Only production references count: test-gated families are excluded on both ends.
Each reference is a distinct (source file, defining module, item) triple, so a
re-export at the parent root is attributed to the child that defines the item.
"""
import argparse
from collections import Counter, defaultdict
import json
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parent))

from coupling_graph import components, layering, upward_edges  # noqa: E402
from module_graph import LibraryModel, node_graph  # noqa: E402
from ownership_units import stray_inherent_impls  # noqa: E402

TYPE_KINDS = ('struct', 'enum', 'trait', 'type', 'union')


def coupling(lib_source, parent, repository, model=None):
    """Children of `parent`, their reference edges and item kinds; the parent root is excluded."""
    model = model or LibraryModel(lib_source, repository)
    children = sorted(c.names for c in model.index[parent].children
                      if not model.surface.in_family(c.names))
    edges, kinds, _lines = node_graph(model, children + [parent])
    named = {}
    for (a, b), refs in edges.items():
        if a != parent and b != parent:
            named[(a[-1], b[-1])] = refs
    return [c[-1] for c in children], named, kinds


def strays(model, parent):
    """Inherent impls sitting in another top-level child than their type, as sorted labels."""
    return sorted({f"{'::'.join(module)} -> {'::'.join(key[0])}::{key[1]}"
                   for module, key, _header in stray_inherent_impls(model, parent)})


def report(lib_source, parent, repository):
    model = LibraryModel(lib_source, repository)
    children, edges, kinds = coupling(lib_source, parent, repository, model)
    weights = {edge: len(refs) for edge, refs in edges.items()}
    groups = components(children, weights)
    cyclic = [g for g in groups if len(g) > 1]
    order = layering(children, weights)
    upward = upward_edges(order, weights)
    upward_rows = []
    for (low, high) in upward:
        refs = sorted(edges[(low, high)])
        items = Counter(item for _source, item in refs)
        upward_rows.append({'from': low, 'to': high, 'references': len(refs),
                            'items': [{'item': item, 'kind': kinds[item], 'references': n}
                                      for item, n in sorted(items.items())]})
    upward_rows.sort(key=lambda r: (-r['references'], r['from'], r['to']))
    by_kind = Counter()
    for row in upward_rows:
        for item in row['items']:
            by_kind['type' if item['kind'] in TYPE_KINDS else 'function' if item['kind'] == 'fn'
                    else item['kind']] += item['references']
    total = sum(weights.values())
    within = sum(w for (a, b), w in weights.items() if any(a in g and b in g for g in cyclic))
    return {'version': 1, 'parent': '::'.join(parent), 'modules': len(children),
            'acyclic_modules': sum(1 for g in groups if len(g) == 1),
            'largest_component': max((len(g) for g in groups), default=0),
            'components': [sorted(g) for g in cyclic],
            'references': total, 'references_within_components': within,
            'upward_edges': len(upward_rows),
            'upward_references': sum(r['references'] for r in upward_rows),
            'upward_references_by_kind': dict(sorted(by_kind.items())),
            'layering_top_to_bottom': order, 'upward': upward_rows,
            'stray_impls': strays(model, parent),
            'edges': [{'from': a, 'to': b, 'references': w} for (a, b), w in sorted(weights.items())]}


def markdown(result):
    lines = ['# Module coupling: `' + result['parent'] + '`', '',
             f"- top-level modules: {result['modules']}",
             f"- acyclic modules: {result['acyclic_modules']}",
             f"- largest strongly connected component: {result['largest_component']}",
             f"- cross-module references: {result['references']} "
             f"({result['references_within_components']} inside components)",
             f"- upward edges against the layering: {result['upward_edges']} carrying "
             f"{result['upward_references']} references {result['upward_references_by_kind']}", '',
             '## Layering, top to bottom', '', ' > '.join(result['layering_top_to_bottom']), '',
             '## Upward edges', '']
    for row in result['upward']:
        lines.append(f"- `{row['from']} -> {row['to']}` ({row['references']}): " +
                     ', '.join(f"`{i['item'].rsplit('::', 1)[-1]}` {i['kind']}" for i in row['items']))
    return '\n'.join(lines) + '\n'


def freeze(result):
    return {'version': 1, 'parent': result['parent'], 'order': result['layering_top_to_bottom'],
            'upward': sorted([r['from'], r['to']] for r in result['upward']),
            'stray_impls': result['stray_impls']}


def ratchet(layering, children, edges, stray=()):
    """Refusals for one tree against a frozen layering; empty means the rule passes. `stray`
    lists inherent impls outside their type's top-level child (a crate split refuses them);
    only those frozen in "stray_impls" pass, and a frozen one that is gone must be removed."""
    position = {name: i for i, name in enumerate(layering['order'])}
    allowed = {tuple(edge) for edge in layering['upward']}
    refusals = [f'module {name} is not placed in the layering; add it to "order"'
                for name in children if name not in position]
    present = set()
    for (low, high), refs in sorted(edges.items()):
        if low in position and high in position and position[low] > position[high] and refs:
            present.add((low, high))
            if (low, high) not in allowed:
                example = sorted(refs)[0]
                refusals.append(f'new upward edge {low} -> {high} ({len(refs)} references, '
                                f'first: {example[1]} in {example[0]})')
    refusals += [f'baseline edge {a} -> {b} no longer exists; remove it from "upward"'
                 for a, b in sorted(allowed - present)]
    frozen = set(layering.get('stray_impls', []))
    refusals += [f'inherent impl outside its type\'s module: {label}; move it home, or make it an '
                 f'extension trait owned where it sits, or a free function'
                 for label in sorted(set(stray) - frozen)]
    refusals += [f'baseline stray impl {label} no longer exists; remove it from "stray_impls"'
                 for label in sorted(frozen - set(stray))]
    return refusals


def check_all(lib, layering_files, root):
    """Ratchet every layering file against one parsed library; 1 if any refuses, 2 if unreadable."""
    layerings = []
    for path in layering_files:
        if not path.is_file():
            print(f'layering file {path.as_posix()} does not exist; regenerate it with '
                  f'module_coupling.py baseline', file=sys.stderr)
            return 2
        layerings.append((path, json.loads(path.read_text(encoding='utf-8'))))
    model, failed, many = LibraryModel(lib, root), False, len(layerings) > 1
    for path, layering in layerings:
        parent = tuple(layering['parent'].split('::'))
        children, edges, _kinds = coupling(lib, parent, root, model)
        refusals = ratchet(layering, children, edges, strays(model, parent))
        for refusal in refusals:
            print(f"[{layering['parent']}] {refusal}" if many else refusal, file=sys.stderr)
        summary = {'refusals': len(refusals), 'upward_edges': len(layering['upward'])}
        if many:
            summary = {'layering': path.as_posix(), 'parent': layering['parent'], **summary}
        print(json.dumps(summary))
        failed = failed or bool(refusals)
    return 1 if failed else 0


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = parser.add_subparsers(dest='command', required=True)
    run = sub.add_parser('report')
    run.add_argument('lib', type=Path)
    run.add_argument('module')
    run.add_argument('--out', type=Path, required=True)
    run.add_argument('--root', type=Path, default=Path.cwd())
    frozen = sub.add_parser('baseline')
    frozen.add_argument('lib', type=Path)
    frozen.add_argument('module')
    frozen.add_argument('--out', type=Path, required=True)
    frozen.add_argument('--root', type=Path, default=Path.cwd())
    checking = sub.add_parser('check')
    checking.add_argument('lib', type=Path)
    checking.add_argument('layering', type=Path, nargs='+')
    checking.add_argument('--root', type=Path, default=Path.cwd())
    args = parser.parse_args(argv)
    if args.command == 'check':
        return check_all(args.lib, args.layering, args.root.resolve())
    parent = tuple(args.module.split('::'))
    result = report(args.lib, parent, args.root.resolve())
    if args.command == 'baseline':
        args.out.parent.mkdir(parents=True, exist_ok=True)
        args.out.write_text(json.dumps(freeze(result), indent=1, sort_keys=True) + '\n',
                            encoding='utf-8', newline='\n')
        print(json.dumps({'upward_edges': len(result['upward'])}))
        return 0
    args.out.mkdir(parents=True, exist_ok=True)
    (args.out / 'coupling.json').write_text(json.dumps(result, indent=1, sort_keys=True) + '\n',
                                            encoding='utf-8', newline='\n')
    (args.out / 'coupling.md').write_text(markdown(result), encoding='utf-8', newline='\n')
    print(json.dumps({k: result[k] for k in ('modules', 'acyclic_modules', 'largest_component',
                                             'upward_edges', 'upward_references')}, sort_keys=True))
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
