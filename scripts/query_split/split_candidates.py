#!/usr/bin/env python3
"""Derive candidate crate partitions of worth-query-execution from its module graph.

Usage:
    python scripts/query_split/split_candidates.py --out <dir>

Writes one candidate JSON per partition, all derived from structure alone:
- `today`: the single crate;
- `vocabulary-below`: every module that cannot reach the largest cycle, as one lower crate;
- `vocabulary-below-facade-above`: the same, plus the modules only the facade side reaches;
- `layered-<k>-hypothetical`: the greedy feedback-minimizing order cut into k line-balanced
  crates, scored as if the upward references were removed first (Stage 2's outcome).
"""
import argparse
import json
from collections import defaultdict
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parent))

from coupling_graph import components, layering  # noqa: E402
from churn_replay import EXPANDED, LIB  # noqa: E402
from module_graph import LibraryModel, flattened_nodes, label, node_graph  # noqa: E402

DEFAULT = 'worth-query-execution'


def reach(start, graph):
    seen, stack = set(start), list(start)
    while stack:
        for nxt in graph[stack.pop()]:
            if nxt not in seen:
                seen.add(nxt)
                stack.append(nxt)
    return seen


def candidates(model):
    nodes = flattened_nodes(model, EXPANDED)
    edges, _kinds, lines = node_graph(model, nodes)
    weights = {(a, b): len(r) for (a, b), r in edges.items() if r}
    succ, pred = defaultdict(set), defaultdict(set)
    for a, b in weights:
        succ[a].add(b)
        pred[b].add(a)
    largest = max(components(nodes, weights), key=len)
    reaches_cycle = reach(largest, pred)
    below = sorted(n for n in nodes if n not in reaches_cycle and n != ())
    above = sorted(reach(largest, pred) - set(largest) - {()})
    result = [{'name': 'today', 'description': 'The single crate as it stands.',
               'default': DEFAULT, 'crates': {}},
              {'name': 'vocabulary-below',
               'description': 'Modules that cannot reach the largest cycle form one lower crate.',
               'default': DEFAULT, 'crates': {DEFAULT + '-vocabulary': [label(n) for n in below]}},
              {'name': 'vocabulary-below-facade-above',
               'description': 'The vocabulary crate below, and the modules above the cycle on top.',
               'default': DEFAULT,
               'crates': {DEFAULT + '-vocabulary': [label(n) for n in below],
                          DEFAULT + '-core': [label(n) for n in sorted(largest)]}}]
    order = [n for n in layering(nodes, weights) if n != ()]
    for k in (3, 5, 8):
        groups = balanced(order, lines, k)
        names = [f'{DEFAULT}-layer-{i}' for i in range(k)]
        crates = {name: [label(n) for n in group] for name, group in zip(names, groups)}
        result.append({'name': f'layered-{k}-hypothetical',
                       'description': f'The greedy order of {len(order)} module nodes cut into {k} '
                                      'line-balanced crates, top first.',
                       'default': names[0], 'crates': crates, 'hypothetical_order': names})
    return result, {'nodes': len(nodes), 'largest_cycle': len(largest), 'below': len(below),
                    'above': len(above)}


def balanced(order, lines, k):
    """Contiguous groups of a top-to-bottom order, each near total/k production lines."""
    total = sum(lines.get(n, 0) for n in order)
    groups, current, size = [], [], 0
    for index, node in enumerate(order):
        current.append(node)
        size += lines.get(node, 0)
        remaining_groups = k - len(groups) - 1
        if remaining_groups and size >= total / k and len(order) - index - 1 >= remaining_groups:
            groups.append(current)
            current, size = [], 0
    groups.append(current)
    while len(groups) < k:
        groups.append([])
    return groups


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--root', type=Path, default=Path.cwd())
    args = parser.parse_args(argv)
    model = LibraryModel(args.root / LIB, args.root)
    result, shape = candidates(model)
    args.out.mkdir(parents=True, exist_ok=True)
    for candidate in result:
        (args.out / (candidate['name'] + '.json')).write_text(
            json.dumps(candidate, indent=1, sort_keys=True) + '\n', encoding='utf-8', newline='\n')
    print(json.dumps(shape, sort_keys=True))
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
