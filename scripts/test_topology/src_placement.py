"""Assign moving module scopes to testkit and court crates under the v3 naming and ceiling.

Testkit parts are grouped by strongly connected component (a testkit crate may depend on
another only one way), then packed under the ceiling by area. Court parts are grouped by
their family's product frontier (named Query products plus those of the testkits they
read), then packed under the ceiling by whole module areas, descending only oversized ones.
"""
from collections import defaultdict

from court_packing import area_suffix, first_fit_decreasing, whole_areas
from court_partition import CEILING
from frontier_adapter import surface
from rust_source import crate_uses

STEM = 'worth-query'


def part_lines(model, names, disposition, part):
    measured = getattr(model, 'measured', None)
    if measured and (names, part) in measured:
        return measured[(names, part)]
    record = model.records[names]
    text = record.node.own_text()
    total = text.count('\n') + 1
    if disposition != 'split':
        return total
    tests = sum(text.count('\n', i.start, i.end) + 1 for i in record.items if i.test)
    return tests if part == 'court' else max(total - tests, 1)


def support_graph(model, kinds):
    """Edges between testkit parts: a support scope reading another support scope."""
    support = sorted(n for n, k in kinds.items() if k in ('testkit', 'split'))
    members = set(support)
    edges = defaultdict(set)
    for names in support:
        for _reference, origin in model.records[names].uses:
            if origin and origin[0] in members and origin[0] != names:
                edges[names].add(origin[0])
    return support, edges


def components(nodes, edges):
    index, low, stack, on, result, counter = {}, {}, [], set(), [], [0]

    def visit(root):
        work = [(root, iter(sorted(edges[root])))]
        index[root] = low[root] = counter[0]
        counter[0] += 1
        stack.append(root)
        on.add(root)
        while work:
            node, successors = work[-1]
            for nxt in successors:
                if nxt not in index:
                    index[nxt] = low[nxt] = counter[0]
                    counter[0] += 1
                    stack.append(nxt)
                    on.add(nxt)
                    work.append((nxt, iter(sorted(edges[nxt]))))
                    break
                if nxt in on:
                    low[node] = min(low[node], index[nxt])
            else:
                work.pop()
                if work:
                    low[work[-1][0]] = min(low[work[-1][0]], low[node])
                if low[node] == index[node]:
                    group = []
                    while True:
                        member = stack.pop()
                        on.discard(member)
                        group.append(member)
                        if member == node:
                            break
                    result.append(tuple(sorted(group)))
    for node in sorted(nodes):
        if node not in index:
            visit(node)
    return result


def pack(units, ceiling):
    """units: dicts with 'owner_area' and 'packing_lines'; returns lists of units per crate."""
    areas = whole_areas(units, ceiling)
    return [[u for area in piece['areas'] for u in area['units']]
            for piece in first_fit_decreasing(areas, ceiling)]


def place_testkits(model, kinds, products_of, frontier_of_products, ceiling=CEILING):
    support, edges = support_graph(model, kinds)
    groups = components(support, edges)
    by_need = defaultdict(list)
    for group in groups:
        need = tuple(sorted(set(frontier_of_products(set().union(*(products_of(n) for n in group))))
                            - {'worth-query-execution'}))
        lines = sum(part_lines(model, n, kinds[n], 'testkit') for n in group)
        by_need[need].append({'owner_area': '/'.join(group[0]), 'packing_lines': lines, 'members': group})
    crates = {}
    for need, units in sorted(by_need.items()):
        pieces = pack(units, ceiling)
        for index, piece in enumerate(pieces):

            name = unique(testkit_name(need, piece, len(pieces) > 1), crates)
            crates[name] = sorted(n for unit in piece for n in unit['members'])
    return crates


def unique(name, taken):
    """Two pieces led by the same area keep distinct names by their order."""
    candidate, index = name, 2
    while candidate in taken:
        candidate, index = f'{name}-{index}', index + 1
    return candidate


def testkit_name(need, piece, split):
    parts = ['execution'] + [n.removeprefix(STEM + '-') for n in need]
    if split:
        parts.append(area_label(piece, 'execution'))
    return STEM + '-testkit-' + '-'.join(parts)


def place_courts(model, kinds, frontier_of, ceiling=CEILING):
    """Families grouped by frontier, then court parts packed by area under the ceiling."""
    by_frontier = defaultdict(list)
    for family in sorted(model.moving):
        names = [n for n in model.records if model.records[n].family == family
                 and kinds[n] in ('court', 'local', 'split')]
        if not names:
            continue
        frontier = tuple(frontier_of(family, names))
        for n in names:
            by_frontier[frontier].append({'owner_area': '/'.join(n), 'names': n,
                                          'packing_lines': part_lines(model, n, kinds[n], 'court')})
    courts = {}
    for frontier, units in sorted(by_frontier.items()):
        base = STEM + '-court-' + surface(STEM, list(frontier))
        pieces = pack(units, ceiling)
        for index, piece in enumerate(sorted(pieces, key=lambda p: min(u['owner_area'] for u in p))):
            name = base if index == 0 else unique(base + '-' + area_label(piece, base), courts)
            courts[name] = {'frontier': list(frontier), 'members': sorted(u['names'] for u in piece)}
    return courts


def area_label(piece, base):
    """The piece's largest top area, named by court_packing's suffix rule."""
    weights = defaultdict(int)
    for unit in piece:
        words = [s for s in unit['owner_area'].split('/') if s not in ('domain_computation', 'tests', 'mod')]
        weights[tuple(words[:2])] += unit['packing_lines']
    lead = max(sorted(weights), key=lambda k: weights[k])
    return area_suffix(lead, base.removeprefix(STEM + '-court-'), ('execution',))


def named_products(model, names, aliases):
    """Query products a scope names, by crate alias (workspace members only)."""
    used = crate_uses(model.records[names].node.own_text(), set(aliases))
    return {aliases[a] for a in used}
