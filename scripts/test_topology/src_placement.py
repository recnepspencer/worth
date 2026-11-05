"""Assign moving module scopes to testkit and court crates under the v3 naming and ceiling.

Testkit parts are grouped by strongly connected component (a testkit crate may depend on
another only one way), then packed under the ceiling by area. Court parts are grouped by
their family's product frontier (named Query products plus those of the testkits they
read), then packed under the ceiling by whole module areas, descending only oversized ones.

`SRC_PINS_DUMP=<dir>` writes the final placement (`courts.json`, `testkits.json`: scope path
`a/b/c` -> crate, and `frontiers.json`). `SRC_PINS=<dir>` reads such a placement back: a scope
keeps its pinned crate, so a later tree ports into an earlier plan's crates; a new scope joins
the crate of its longest pinned prefix (a court only of its own frontier), and packs as usual
only when none exists.
"""
from collections import defaultdict
import json
import os
from pathlib import Path

from court_packing import area_suffix, first_fit_decreasing, whole_areas
from court_partition import CEILING
from frontier_adapter import surface
from rust_source import crate_uses, tokens

STEM = 'worth-query'


def pins(kind):
    path = os.environ.get('SRC_PINS')
    if not path:
        return None
    return {'modules': json.loads((Path(path) / f'{kind}.json').read_text(encoding='utf-8')),
            'frontiers': json.loads((Path(path) / 'frontiers.json').read_text(encoding='utf-8'))}


def dump(kind, crates, frontiers=None):
    path = os.environ.get('SRC_PINS_DUMP')
    if not path:
        return
    Path(path).mkdir(parents=True, exist_ok=True)
    placed = {'/'.join(n): crate for crate, members in crates.items() for n in members}
    (Path(path) / f'{kind}.json').write_text(json.dumps(placed, indent=1, sort_keys=True), encoding='utf-8')
    if frontiers is not None:
        (Path(path) / 'frontiers.json').write_text(json.dumps(frontiers, indent=1, sort_keys=True), encoding='utf-8')


def pinned_crate(names, pinned):
    """(crate, depth) of the longest pinned module prefix of `names`, or (None, 0)."""
    for depth in range(len(names), 0, -1):
        crate = pinned.get('/'.join(names[:depth]))
        if crate:
            return crate, depth
    return None, 0


def repin(default, unit_names, pinned, kind, near_fits):
    """{crate: [names]} from `default` with each scope moved to its pinned crate when that crate
    is of `kind`; an unpinned scope joins the crate `near_fits` allows that shares its longest
    pinned prefix, else keeps its default crate."""
    home = {n: crate for crate, members in default.items() for n in members}
    result = defaultdict(list)
    for names in unit_names:
        crate, _ = pinned_crate(names, pinned)
        if crate is None or not kind(crate):
            near = [(sum(1 for a, b in zip(names, other) if a == b), c) for other, c in
                    ((tuple(k.split('/')), c) for k, c in pinned.items()) if kind(c) and near_fits(c, names)]
            best = max(near, default=(0, None))
            crate = best[1] if best[0] > 0 else home[names]
        result[crate].append(names)
    return {crate: sorted(members) for crate, members in result.items()}


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
    door_owner = {method: names for names in support for door in model.records[names].doors
                  for method in door.methods}
    for names in support:
        for _reference, origin in model.records[names].uses:
            if origin and origin[0] in members and origin[0] != names:
                edges[names].add(origin[0])
        # Calling a door method imports its trait: a dependency on the door's owner.
        stream = tokens(model.records[names].node.own_text())
        for i in range(len(stream) - 1):
            owner = door_owner.get(stream[i + 1].value) if stream[i].value in ('.', ':') else None
            if owner is not None and owner != names:
                edges[names].add(owner)
    # Mounted foreign source reads its parent and siblings through `super::`: one crate holds them.
    from src_court_plan import foreign_includes
    # Foreign source reaches its siblings through `super::` too: a parent with mounted foreign
    # children keeps all its support children.
    hosts = {names[:-1] for names in support
             if model.records[names].foreign or foreign_includes(model.records[names])}
    for names in support:
        if names[:-1] in hosts and names[:-1] in members:
            edges[names].add(names[:-1])
            edges[names[:-1]].add(names)
    from src_ownership import impl_couplings
    for owner, holders in impl_couplings(model).items():
        for holder in holders & members if owner in members else ():
            edges[owner].add(holder)
            edges[holder].add(owner)
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
    """Support components packed so testkit crates form a DAG.

    A component's need is the frontier of everything it and its support dependencies name,
    so a component that uses a `decl` component carries `decl` too and groups depend one way.
    Inside a group, components fill crates contiguously in dependency order (dependencies
    first, ties by area), so a crate only depends on crates filled before it.
    """
    support, edges = support_graph(model, kinds)
    groups = components(support, edges)
    group_of = {n: g for g in groups for n in g}
    depends = {g: {group_of[t] for n in g for t in edges[n]} - {g} for g in groups}
    named = {}

    def closure(g):
        if g not in named:
            named[g] = set().union(*(products_of(n) for n in g))
            for d in sorted(depends[g]):
                named[g] |= closure(d)
        return named[g]

    by_need = defaultdict(list)
    for g in groups:
        need = tuple(sorted(set(frontier_of_products(closure(g))) - {'worth-query-execution'}))
        by_need[need].append(g)
    crates = {}
    for need, members in sorted(by_need.items(), key=lambda kv: len(kv[0])):
        order = topological(members, depends)
        pieces, current, size = [], [], 0
        for g in order:
            lines = sum(part_lines(model, n, kinds[n], 'testkit') for n in g)
            if current and size + lines > ceiling:
                pieces.append(current)
                current, size = [], 0
            current.append({'owner_area': '/'.join(g[0]), 'packing_lines': lines, 'members': g})
            size += lines
        if current:
            pieces.append(current)
        for piece in pieces:
            name = unique(testkit_name(need, piece, len(pieces) > 1), crates)
            crates[name] = sorted(n for unit in piece for n in unit['members'])
    spec = pins('testkits')
    if spec:
        default = dict(crates)
        units = sorted(n for members in default.values() for n in members)
        regrouped = repin(default, units, spec['modules'], lambda crate: crate.startswith(STEM + '-testkit-'),
                          lambda crate, names: True)
        crates = dict(sorted(regrouped.items()))
    dump('testkits', crates)
    return crates


def topological(groups, depends):
    """Dependencies first; among ready components, the smallest area path first."""
    members = set(groups)
    waiting = {g: {d for d in depends[g] if d in members} for g in groups}
    ready = sorted((g for g in groups if not waiting[g]), key=lambda g: '/'.join(g[0]))
    order = []
    while ready:
        g = ready.pop(0)
        order.append(g)
        for other in sorted(members):
            if g in waiting[other]:
                waiting[other].discard(g)
                if not waiting[other]:
                    ready.append(other)
                    ready.sort(key=lambda x: '/'.join(x[0]))
    if len(order) != len(groups):
        raise ValueError('support components are cyclic after condensation')
    return order


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
    spec = pins('courts')
    if spec:
        frontier_of_unit = {u['names']: list(f) for f, units in by_frontier.items() for u in units}
        default = {name: court['members'] for name, court in courts.items()}

        def fits(crate, names):
            return spec['frontiers'].get(crate) == frontier_of_unit[names]

        regrouped = repin(default, sorted(frontier_of_unit), spec['modules'],
                          lambda crate: crate.startswith(STEM + '-court-'), fits)
        courts = {crate: {'frontier': frontier_of_unit[members[0]], 'members': members}
                  for crate, members in sorted(regrouped.items())}
    dump('courts', {name: court['members'] for name, court in courts.items()},
         {name: court['frontier'] for name, court in courts.items()})
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
