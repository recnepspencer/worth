#!/usr/bin/env python3
"""Simulate moving helper functions down beside the types they classify, to a fixpoint.

Usage:
    python scripts/query_split/ownership_simulation.py <lib.rs> <module::path>... --out <dir>

Plain Python over the source (no cargo), against each parent's current greedy layering:
1. every non-type item (function, constant, static) at the root of a behaviour-coupled unit's
   blockers is judged beside the types it classifies: at the lowest destination among the
   units it holds. It could move when everything it names sits at or below that destination
   (strict), or, with `--helpers-carry-types`, when the types it names can move there with it;
   a helper never carries another helper;
2. every such helper and every unit that becomes movable as a result moves, repeatedly, until
   nothing else can move. A moved item's references come from its new module, and the
   references it makes leave its old one;
3. the result reports the largest component, the references against the current layering and
   against a fresh layering, the moves taken, and the seams that still hold the component.
   Each seam gets the same nest-or-contract default as the ownership ruling sheet.
Macro-made items (`thread_local!` observation counters) have no declaration to read; they
carry no dependency of their own and move with the code that uses them.
"""
import argparse
from collections import Counter, defaultdict
import json
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parent))
sys.path.insert(0, str(Path(__file__).resolve().parent.parent / 'test_topology'))

from coupling_graph import components, layering  # noqa: E402
from item_mentions import declarations  # noqa: E402
from module_coupling import coupling  # noqa: E402
from module_graph import LibraryModel  # noqa: E402
from ownership_units import TYPE_KINDS, node_of, resolved_mentions, units  # noqa: E402

HELPER_KINDS = ('fn', 'const', 'static')
HAND_TRAIT_LIMIT = 12


def split(key):
    parts = tuple(key.split('::'))
    return parts[:-1], parts[-1]


class World:
    """Item locations, item dependencies and the reference list of one parent."""

    def __init__(self, model, parent, all_units, helpers, helpers_carry_types=False, stack=None):
        self.helpers_carry_types = helpers_carry_types
        self.max_carry = 10 ** 9
        self.bidirectional = False
        self.up_first = False
        self.not_destinations = set()
        self.model, self.parent = model, parent
        self.children, edges, self.kinds = coupling(None, parent, None, model)
        if stack is not None:
            self.order = [m for layer in stack['layers'] for m, _purpose in layer['modules']]
            missing = set(self.children) ^ set(self.order)
            if missing:
                raise SystemExit(f'stack mismatch for {parent}: {sorted(missing)}')
        else:
            self.order = list(reversed(layering(self.children, {e: len(r) for e, r in edges.items()})))
        self.position = {n: i for i, n in enumerate(self.order)}
        self.refs = [(a, source, key) for (a, _b), refs in edges.items() for source, key in refs]
        self.units, self.helpers = all_units, helpers
        self.location = {}
        self.roots = {}
        self.moved = {}
        self.moved_from_source = {}

    def kind(self, key):
        found = self.kinds.get(key)
        if found is not None:
            return found
        if key in self.helpers:
            return self.helpers[key].keyword
        unit = self.units.get(split(key))
        return unit.keyword if unit is not None else None

    def node(self, key):
        if key in self.location:
            return self.location[key]
        module, _name = split(key)
        return node_of(module, self.parent)

    def needs(self, key):
        """[(item key, impl label)] the item names: a unit's declaration and impls (an impl of a
        crate trait that sits with a higher trait is left out), or a helper's body."""
        module, name = split(key)
        unit = self.units.get((module, name))
        result = []
        if unit is not None:
            sources = [('declaration', getattr(unit, 'declaration_mentions', []))]
            own_node = self.node(key)
            for impl in unit.impls:
                trait_key = impl.trait[1] if impl.trait and impl.trait[0] == 'item' else None
                if trait_key is not None and trait_key != (module, name) and impl.self_key == (module, name):
                    trait_node = node_of(trait_key[0], self.parent)
                    if trait_node is not None and own_node is not None and \
                            self.position.get(trait_node, -1) > self.position.get(own_node, -1):
                        continue
                sources.append((impl.header, impl.mentions))
            for label, mentions in sources:
                for target, item, _kind in mentions:
                    if (target, item) != (module, name):
                        result.append(('::'.join(target + (item,)), label))
            return result
        helper = self.helpers.get(key)
        if helper is not None:
            return [('::'.join(t + (i,)), 'body') for t, i, _k in helper.resolved if (t, i) != (module, name)]
        return None

    def source_node(self, a, source, key):
        return self.moved_from_source.get((source, key), a)

    def against(self):
        """{item key: {referencing node: refs}} referenced from below its current node."""
        result = defaultdict(Counter)
        for a, source, key in self.refs:
            a = self.source_node(a, source, key)
            if a in self.not_destinations:
                continue
            b = self.node(key)
            if b is None or a == b:
                continue
            if self.position[a] < self.position[b]:
                result[key][a] += 1
        return result

    def judge(self, key, destination, memo, active=frozenset(), comove_types=True):
        """(True, co-moved keys) or (False, [(blocking item, label)]) for moving `key` to
        `destination` whole. A unit carries the types it needs (as the ownership analysis
        does); a helper carries nothing. Either carries the macro-made statics it names (the
        `thread_local!` observation counters), which have no declaration and no dependency."""
        memo_key = (key, destination, comove_types)
        if memo_key in memo:
            return memo[memo_key]
        if key in active:
            return True, set()
        needs = self.needs(key)
        if needs is None:
            result = (True, set()) if split(key)[1].isupper() else (False, [(key, 'no declaration')])
            memo[memo_key] = result
            return result
        limit = self.position[destination]
        active = active | {key}
        carried, blocking, roots = set(), [], set()
        for target, label in needs:
            node = self.node(target)
            if node is None or self.position.get(node, -1) <= limit:
                continue
            kind = self.kind(target)
            if kind is None and split(target)[1].isupper():
                carried.add(target)
                continue
            if kind in TYPE_KINDS and comove_types:
                ok, detail = self.judge(target, destination, memo, active, comove_types)
                if ok:
                    carried |= {target} | detail
                    continue
                roots |= self.roots.get((target, destination), set())
            else:
                roots.add(target)
            blocking.append((target, label))
        if blocking:
            self.roots[(key, destination)] = roots
        result = (False, blocking) if blocking else (True, carried)
        memo[memo_key] = result
        return result

    def users(self):
        """{item key: {items whose declaration or impls name it}} (units and helpers)."""
        if getattr(self, '_users', None) is None:
            found = defaultdict(set)
            keys = ['::'.join(k[0] + (k[1],)) for k in self.units] + list(self.helpers)
            for key in keys:
                for target, _label in self.needs(key) or []:
                    found[target].add(key)
            self._users = found
        return self._users

    def lift_target(self, key):
        """The highest module above `key`'s that `key` needs, or None when it needs none above."""
        here = self.node(key)
        if here is None:
            return None
        best = None
        for target, _label in self.needs(key) or []:
            node = self.node(target)
            if node is None or self.position[node] <= self.position[here]:
                continue
            if best is None or self.position[node] > self.position[best]:
                best = node
        return best

    def move(self, key, destination):
        old = self.node(key)
        self.location[key] = destination
        self.moved[key] = (old, destination)
        needed = {t for t, _l in (self.needs(key) or [])}
        files = self.files_of(key)
        for a, source, target in self.refs:
            if target in needed and source in files and self.source_node(a, source, target) == old:
                self.moved_from_source[(source, target)] = destination

    def files_of(self, key):
        module, name = split(key)
        files = set()
        unit = self.units.get((module, name))
        if unit is not None:
            files.add(str(self.model.index[unit.declaration.module].source))
            for impl in unit.impls:
                files.add(str(self.model.index[impl.module].source))
        elif key in self.helpers:
            files.add(str(self.model.index[self.helpers[key].module].source))
        return files

    def weights(self):
        graph = Counter()
        for a, source, key in self.refs:
            a = self.source_node(a, source, key)
            b = self.node(key)
            if b is None or a == b:
                continue
            graph[(a, b)] += 1
        return graph

    def measure(self):
        graph = self.weights()
        largest = max(len(c) for c in components(self.children, dict(graph)))
        against_current = sum(w for (a, b), w in graph.items() if self.position[a] < self.position[b])
        fresh = list(reversed(layering(self.children, dict(graph))))
        fresh_position = {n: i for i, n in enumerate(fresh)}
        against_fresh = sum(w for (a, b), w in graph.items() if fresh_position[a] < fresh_position[b])
        return {'largest_component': largest, 'against_current_layering': against_current,
                'against_fresh_layering': against_fresh}


def short(key, parent):
    prefix = '::'.join(parent) + '::'
    return key[len(prefix):] if key.startswith(prefix) else key


def simulate(world):
    parent = world.parent
    before = world.measure()
    # 1. Helpers blocking coupled units, judged where they would go.
    against = world.against()
    memo = {}
    held = defaultdict(set)
    unit_destination = {}
    for key, froms in against.items():
        if world.kind(key) not in TYPE_KINDS:
            continue
        destination = min(froms, key=lambda m: world.position[m])
        ok, _detail = world.judge(key, destination, memo)
        if not ok:
            unit_destination[key] = destination
            for root in world.roots.get((key, destination), set()):
                held[root].add(key)
    helpers = []
    for key, units_held in sorted(held.items(), key=lambda kv: (-len(kv[1]), kv[0])):
        if world.kind(key) not in HELPER_KINDS:
            continue
        # Beside the types it classifies: the lowest destination among the units it holds.
        destination = min((unit_destination[u] for u in units_held), key=lambda m: world.position[m])
        ok, detail = world.judge(key, destination, memo, comove_types=world.helpers_carry_types)
        row = {'helper': short(key, parent), 'kind': world.kind(key), 'owner': world.node(key),
               'destination': destination, 'units_blocked': len(units_held),
               'references_held': sum(sum(against[u].values()) for u in units_held),
               'verdict': 'could move' if ok else 'blocked'}
        if not ok:
            row['blocked_by'] = sorted({short(b, parent) + ' (' + str(world.kind(b)) + ')' for b, _l in detail})[:8]
        helpers.append(row)
    # 2. Fixpoint: move every helper that can go beside the units it holds, then every unit
    #    referenced from below that can then move whole; repeat until nothing moves.
    rounds = []
    while True:
        against = world.against()
        memo = {}
        held, unit_destination, unit_moves = defaultdict(set), {}, []
        for key, froms in sorted(against.items()):
            if world.kind(key) not in TYPE_KINDS:
                continue
            destination = min(froms, key=lambda m: world.position[m])
            ok, carried = world.judge(key, destination, memo)
            if ok and len(carried) + 1 > world.max_carry:
                ok = False
                world.roots[(key, destination)] = set()
            if ok:
                unit_moves.append((key, destination, carried))
            else:
                unit_destination[key] = destination
                for root in world.roots.get((key, destination), set()):
                    held[root].add(key)
        helper_moves = []
        for key, units_held in sorted(held.items()):
            if world.kind(key) not in HELPER_KINDS:
                continue
            destination = min((unit_destination[u] for u in units_held), key=lambda m: world.position[m])
            if world.position[destination] >= world.position[world.node(key)]:
                continue
            ok, carried = world.judge(key, destination, memo, comove_types=world.helpers_carry_types)
            if ok and len(carried) + 1 <= world.max_carry:
                helper_moves.append((key, destination, carried))
        up_moves = []
        if world.bidirectional:
            # Lift the user instead: an item below that names an item above it, and that nothing
            # below its new home names, goes up to the highest module it needs.
            users = world.users()
            planned = set() if world.up_first else {k for k, _d, c in helper_moves + unit_moves for k in {k} | c}
            for key, froms in sorted(against.items()):
                if key in planned or key in unit_destination and False:
                    continue
                if world.kind(key) not in TYPE_KINDS and world.kind(key) not in HELPER_KINDS:
                    pass
                for user in sorted(users.get(key, ())):
                    if user in planned or user in world.moved:
                        continue
                    here = world.node(user)
                    if here is None or here not in froms:
                        continue
                    up = world.lift_target(user)
                    if up is None:
                        continue
                    if any(world.node(u) is not None and world.position[world.node(u)] < world.position[up]
                           and u not in planned for u in users.get(user, ())):
                        continue
                    if any(self_ref for self_ref in []):
                        continue
                    up_moves.append((user, up, set()))
                    planned.add(user)
        if world.bidirectional and world.up_first and up_moves:
            lifted = {u for u, _d, _c in up_moves}
            lifted_for = set()
            users = world.users()
            for key in against:
                if users.get(key, set()) & lifted:
                    lifted_for.add(key)
            unit_moves = [m for m in unit_moves if m[0] not in lifted_for]
            helper_moves = [m for m in helper_moves if m[0] not in lifted_for]
        if not unit_moves and not helper_moves and not up_moves:
            break
        moved_now = set()
        for key, destination, carried in helper_moves + unit_moves + up_moves:
            for item in {key} | carried:
                if item not in moved_now and world.node(item) != destination:
                    world.move(item, destination)
                    moved_now.add(item)
        if not moved_now:
            break
        rounds.append({'helpers': sum(1 for k in moved_now if world.kind(k) in HELPER_KINDS),
                       'units': sum(1 for k in moved_now if world.kind(k) in TYPE_KINDS),
                       'statics': sum(1 for k in moved_now if world.kind(k) is None)})
    after = world.measure()
    # 3. What still holds the component.
    graph = world.weights()
    groups = [g for g in components(world.children, dict(graph)) if len(g) > 1]
    largest = max(groups, key=len) if groups else []
    against = world.against()
    memo = {}
    seams = []
    pairs = set()
    for (a, b), w in graph.items():
        if a in largest and b in largest and (b, a) in graph and a < b:
            pairs.add((a, b))
    for a, b in pairs:
        low, high = (a, b) if world.position[a] < world.position[b] else (b, a)
        up = Counter()
        reasons = Counter()
        for key, froms in against.items():
            if world.node(key) != high or low not in froms:
                continue
            up[key] += froms[low]
            kind = world.kind(key)
            if kind in TYPE_KINDS or kind in HELPER_KINDS:
                destination = min(froms, key=lambda m: world.position[m])
                ok, detail = world.judge(key, destination, memo, comove_types=kind in TYPE_KINDS or world.helpers_carry_types)
                for blocker, label in ([] if ok else detail):
                    reasons[(short(key, parent), short(blocker, parent), label[:80])] += 1
            else:
                reasons[(short(key, parent), '(' + str(kind) + ': not movable)', '')] += 1
        if not up:
            continue
        up_refs = sum(up.values())
        items = len(up)
        exposed = sum(w for (x, y), w in graph.items() if y == low and x != high and
                      world.position[x] < world.position[high])
        default = 'contract' if items <= HAND_TRAIT_LIMIT or exposed > up_refs else 'nest or merge'
        seams.append({'pair': [low, high], 'references': {f'{low} -> {high}': graph[(low, high)],
                                                         f'{high} -> {low}': graph[(high, low)]},
                      'against': up_refs, 'items': items, 'default': default,
                      'top_items': [{'item': short(k, parent), 'references': n} for k, n in up.most_common(8)],
                      'why': [{'item': i, 'blocked_by': b, 'in': l} for (i, b, l), _n in reasons.most_common(8)]})
    seams.sort(key=lambda s: -s['against'])
    return {'parent': '::'.join(parent), 'modules': len(world.children), 'before': before, 'after': after,
            'helpers_judged': helpers,
            'helpers_could_move': sum(1 for h in helpers if h['verdict'] == 'could move'),
            'rounds': rounds,
            'moved': {'helpers': sum(1 for k in world.moved if world.kind(k) in HELPER_KINDS),
                      'units': sum(1 for k in world.moved if world.kind(k) in TYPE_KINDS)},
            'moves': sorted({'item': short(k, parent), 'kind': world.kind(k), 'from': f, 'to': t}
                            for k, (f, t) in world.moved.items()) if False else
            [{'item': short(k, parent), 'kind': world.kind(k), 'from': f, 'to': t}
             for k, (f, t) in sorted(world.moved.items())],
            'largest_component_after': sorted(largest),
            'seams': seams,
            'seam_defaults': dict(Counter(s['default'] for s in seams))}


def markdown(result):
    b, a = result['before'], result['after']
    lines = [f"# Helper-move simulation: `{result['parent']}` ({result['modules']} modules)", '',
             f"Before: component {b['largest_component']}, {b['against_current_layering']} references against the "
             f"current layering.",
             f"After: component {a['largest_component']}, {a['against_current_layering']} against the current layering, "
             f"{a['against_fresh_layering']} against a fresh layering of the result.",
             f"Moved: {result['moved']['helpers']} helpers and {result['moved']['units']} units over "
             f"{len(result['rounds'])} rounds. Of {len(result['helpers_judged'])} helpers blocking coupled units, "
             f"{result['helpers_could_move']} could move at once.", '',
             f"Remaining seams in the largest component: {len(result['seams'])} "
             f"({', '.join(f'{v} {k}' for k, v in sorted(result['seam_defaults'].items()))}).", '',
             '## Helpers blocking coupled units', '', '| helper | owner | destination | units blocked | verdict |',
             '|---|---|---|---|---|']
    for h in result['helpers_judged'][:60]:
        verdict = h['verdict'] + (': ' + ', '.join(h.get('blocked_by', [])[:3]) if h.get('blocked_by') else '')
        lines.append(f"| `{h['helper']}` | {h['owner']} | {h.get('destination', '')} | {h['units_blocked']} | {verdict} |")
    lines += ['', '## Remaining seams', '']
    for s in result['seams'][:40]:
        low, high = s['pair']
        lines.append(f"### `{low}` ↔ `{high}`: {s['against']} references up in {s['items']} items, default "
                     f"**{s['default']}**")
        lines.append('')
        lines.append('Items: ' + ', '.join(f"`{i['item']}` ({i['references']})" for i in s['top_items']))
        lines.append('')
        for w in s['why'][:6]:
            lines.append(f"- `{w['item']}` held by `{w['blocked_by']}`" + (f" in `{w['in']}`" if w['in'] else ''))
        lines.append('')
    return '\n'.join(lines) + '\n'


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__.split('\n')[0])
    parser.add_argument('lib', type=Path)
    parser.add_argument('parents', nargs='+')
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--root', type=Path, default=Path.cwd())
    parser.add_argument('--stacks', type=Path, nargs='*', default=[],
                        help='stack files (ownership_analysis format) to use instead of the greedy layering')
    parser.add_argument('--not-destination', nargs='*', default=[],
                        help='modules whose references never pull a unit down (treated as seams instead)')
    parser.add_argument('--up-first', action='store_true',
                        help='with --bidirectional, prefer lifting users over moving items down')
    parser.add_argument('--bidirectional', action='store_true',
                        help='also lift a user above the item it names when nothing below names the user')
    parser.add_argument('--max-carry', type=int, default=10 ** 9,
                        help='refuse a move that would carry more than this many items with it')
    parser.add_argument('--helpers-carry-types', action='store_true',
                        help='a helper may take the types it names with it (never another helper)')
    args = parser.parse_args(argv)
    model = LibraryModel(args.lib, args.root)
    all_units, _orphans = units(model)
    helpers = {}
    for declaration in declarations(model, (), HELPER_KINDS):
        declaration.resolved = resolved_mentions(model, declaration.module, declaration.mentions)
        helpers['::'.join(declaration.module + (declaration.name,))] = declaration
    summary = {}
    for parent_path in args.parents:
        parent = tuple(parent_path.split('::'))
        stacks = {json.loads(p.read_text())['parent']: json.loads(p.read_text()) for p in args.stacks}
        world = World(model, parent, all_units, helpers, args.helpers_carry_types, stacks.get(parent_path))
        world.max_carry = args.max_carry
        world.bidirectional = args.bidirectional
        world.up_first = args.up_first
        world.not_destinations = set(args.not_destination)
        result = simulate(world)
        target = args.out / parent[-1]
        target.mkdir(parents=True, exist_ok=True)
        (target / 'simulation.json').write_text(json.dumps(result, indent=1, sort_keys=True) + '\n', encoding='utf-8')
        (target / 'simulation.md').write_text(markdown(result), encoding='utf-8')
        summary[parent_path] = {'before': result['before'], 'after': result['after'], 'moved': result['moved'],
                                'helpers_judged': len(result['helpers_judged']),
                                'helpers_could_move': result['helpers_could_move'],
                                'seams': len(result['seams']), 'seam_defaults': result['seam_defaults']}
    print(json.dumps(summary, indent=1, sort_keys=True))
    return 0


if __name__ == '__main__':
    sys.exit(main())
