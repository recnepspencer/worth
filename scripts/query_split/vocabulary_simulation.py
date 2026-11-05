#!/usr/bin/env python3
"""Simulate moving each family's refusal and evidence vocabulary to its own bottom module.

Usage:
    python scripts/query_split/vocabulary_simulation.py <lib.rs> <module::path>... --out <dir>

Plain Python over the source (no cargo). Each owner family (a top-level child of the parent)
gets one vocabulary module below every existing module. An item referenced from below its
owner moves to its own family's vocabulary module, whole: a unit is the type with every impl
the unit rule gives it, and it takes along every type and helper it names that sits above
the vocabulary layer, each to that item's own family vocabulary. Traits never move (they are
contracts with behaviour, not vocabulary), and an item that names something that cannot move
stays where it is. A hub (a type with impls in more than one family) and a type whose impls
together exceed `--impl-limit` lines are behaviour, not vocabulary, and stay. The fixpoint
repeats until nothing more moves.

The result lists each vocabulary module with its item count, lines, kinds and source files;
the edges between vocabulary modules (a cycle among them is reported); the largest component
and upward references against a fresh layering; and the seams that still hold the component.
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
from module_graph import LibraryModel  # noqa: E402
from ownership_simulation import HELPER_KINDS, World, short, split  # noqa: E402
from ownership_units import TYPE_KINDS, node_of, resolved_mentions, units  # noqa: E402

MOVABLE_TYPES = ('struct', 'enum', 'type', 'union')
VOCABULARY = 'vocabulary:'


def host_order(root, parent, children, stack):
    """Bottom-to-top order of `parent`'s children by the host supertraits that compile: a module
    whose host extends another's sits above it; application_runtime and application_installation
    sit on top; the stack breaks the ties the host order leaves open."""
    import re
    dc = root / 'workspaces/worth-query/crates/worth-query-execution/src/domain_computation'
    owner_of = {}
    texts = {}
    for path in dc.rglob('host.rs'):
        if 'application_runtime' in path.parts:
            continue
        text = path.read_text(encoding='utf-8')
        m = re.search(r'pub trait (\w+Host)<Schema>:(.*?)\{', text, re.S)
        if not m:
            continue
        rel = path.relative_to(dc).parts[:-1]
        owner_of[m.group(1)] = rel
        texts[m.group(1)] = re.findall(r'(\w+Host)<', m.group(2))
    identity = dc / 'execution_runtime/application_identity.rs'
    owner_of['WorthQueryApplicationIdentityHost'] = ('execution_runtime',)
    owner_of['WorthQueryRuntimeIdentityHost'] = ('primary_graph', 'runtime_host')
    depth = len(parent) - 1          # parent is ('domain_computation', ...)
    def child(rel):
        return rel[depth] if len(rel) > depth and rel[:depth] == parent[1:] else None
    below = defaultdict(set)         # module -> modules it must sit above
    for host, supers in texts.items():
        a = child(owner_of[host])
        for sup in supers:
            if sup not in owner_of:
                continue
            b = child(owner_of[sup])
            if a and b and a != b:
                below[a].add(b)
    rank = {m: i for i, m in enumerate(stack)}
    top = [m for m in ('application_installation', 'application_runtime') if m in children]
    # The known seam: authorization's host names a primary-graph host; the primary graph's hosts
    # extend authorization's, which wins (merges.md section 2 lowers the shared vocabulary).
    if 'authorization' in below and 'primary_graph' in below['authorization']:
        below['authorization'].discard('primary_graph')
    order, placed = [], set()
    remaining = [m for m in children if m not in top]
    while remaining:
        ready = [m for m in remaining if not (below[m] - placed - set(top))]
        if not ready:
            ready = remaining
        pick = min(ready, key=lambda m: (rank.get(m, len(rank)), m))
        order.append(pick)
        placed.add(pick)
        remaining.remove(pick)
    return order + top


class VocabularyWorld(World):
    def __init__(self, model, parent, all_units, helpers, order=None, excluded=frozenset()):
        super().__init__(model, parent, all_units, helpers, helpers_carry_types=True)
        self.vocabularies = set()
        self.excluded = excluded
        if order is not None:
            self.order = order
            self.position = {n: i for i, n in enumerate(order)}
        self.impl_limit = 250

    def owner(self, key):
        module, _name = split(key)
        return node_of(module, self.parent)

    def vocabulary(self, key):
        name = VOCABULARY + self.owner(key)
        if name not in self.vocabularies:
            self.vocabularies.add(name)
            self.position[name] = -1
        return name

    def movable_kind(self, key):
        if split(key)[1] in self.excluded:
            return False
        kind = self.kind(key)
        if not (kind in MOVABLE_TYPES or kind in HELPER_KINDS or (kind is None and split(key)[1].isupper())):
            return False
        unit = self.units.get(split(key))
        if unit is not None:
            # Behaviour, not vocabulary: a hub whose impls spread over several families, or a
            # type whose impls are larger than a vocabulary file could hold.
            families = {node_of(impl.module, self.parent) for impl in unit.impls} - {None}
            if len(families) > 1 or sum(impl.lines for impl in unit.impls) > self.impl_limit:
                return False
        return True

    def judge_vocabulary(self, key, memo, active=frozenset()):
        """(True, carried keys) when `key` and everything it names above the vocabulary layer
        can move to the vocabulary layer; (False, [(blocker, label)]) otherwise."""
        if key in memo:
            return memo[key]
        if key in active:
            return True, set()
        if not self.movable_kind(key):
            memo[key] = (False, [(key, 'not vocabulary: ' + str(self.kind(key)))])
            return memo[key]
        needs = self.needs(key)
        if needs is None:
            result = (True, set()) if split(key)[1].isupper() else (False, [(key, 'no declaration')])
            memo[key] = result
            return result
        active = active | {key}
        carried, blocking = set(), []
        for target, label in needs:
            node = self.node(target)
            if node is None or self.position.get(node, 0) < 0:
                continue
            ok, detail = self.judge_vocabulary(target, memo, active)
            if ok:
                carried |= {target} | detail
            else:
                blocking.append((target, label))
        result = (False, blocking) if blocking else (True, carried)
        memo[key] = result
        return result

    def measure(self):
        graph = self.weights()
        nodes = list(self.children) + sorted(self.vocabularies)
        largest = max(len(c) for c in components(nodes, dict(graph)))
        fresh = list(reversed(layering(nodes, dict(graph))))
        position = {n: i for i, n in enumerate(fresh)}
        against = sum(w for (a, b), w in graph.items() if position[a] < position[b])
        fixed = dict(self.position)
        against_order = sum(w for (a, b), w in graph.items() if fixed.get(a, -1) < fixed.get(b, -1))
        return {'largest_component': largest, 'against_fresh_layering': against,
                'against_host_order': against_order}, fixed


def item_lines(world, key):
    module, name = split(key)
    unit = world.units.get((module, name))
    if unit is not None and unit.declaration is not None:
        return unit.declaration.text.count('\n') + 1 + sum(i.lines for i in unit.impls)
    helper = world.helpers.get(key)
    if helper is not None:
        return helper.text.count('\n') + 1
    return 1


def simulate(world):
    parent = world.parent
    before, _position = world.measure()
    rounds = []
    while True:
        against = world.against()
        memo = {}
        moving = set()
        for key in sorted(against):
            if world.node(key) is not None and world.position.get(world.node(key), 0) < 0:
                continue
            ok, carried = world.judge_vocabulary(key, memo)
            if ok:
                moving |= {key} | carried
        moving = {k for k in moving if world.position.get(world.node(k), 0) >= 0}
        if not moving:
            break
        for key in sorted(moving):
            world.move(key, world.vocabulary(key))
        rounds.append(len(moving))
    after, position = world.measure()
    graph = world.weights()
    # The vocabulary modules: contents and the edges among them.
    modules = defaultdict(lambda: {'items': 0, 'lines': 0, 'kinds': Counter(), 'files': Counter()})
    for key, (_old, new) in world.moved.items():
        row = modules[new]
        row['items'] += 1
        row['lines'] += item_lines(world, key)
        row['kinds'][str(world.kind(key))] += 1
        module, _name = split(key)
        row['files']['::'.join(module[len(parent):])] += 1
    vocab_edges = {f'{a} -> {b}': w for (a, b), w in sorted(graph.items())
                   if a.startswith(VOCABULARY) and b.startswith(VOCABULARY)}
    vocab_cycles = [sorted(c) for c in components(sorted(world.vocabularies),
                                                  {e: w for e, w in graph.items()
                                                   if e[0].startswith(VOCABULARY) and e[1].startswith(VOCABULARY)})
                    if len(c) > 1]
    up_from_vocabulary = {f'{a} -> {b}': w for (a, b), w in sorted(graph.items())
                          if a.startswith(VOCABULARY) and not b.startswith(VOCABULARY)}
    # Seams that still hold the largest component.
    nodes = list(world.children) + sorted(world.vocabularies)
    groups = [g for g in components(nodes, dict(graph)) if len(g) > 1]
    largest = max(groups, key=len) if groups else []
    memo = {}
    seams = []
    for (a, b), w in graph.items():
        if not (a in largest and b in largest and (b, a) in graph and a < b):
            continue
        low, high = (a, b) if position[a] < position[b] else (b, a)
        up = Counter()
        why = Counter()
        for x, source, key in world.refs:
            src = world.source_node(x, source, key)
            if src != low or world.node(key) != high:
                continue
            up[key] += 1
        for key in up:
            ok, detail = world.judge_vocabulary(key, memo)
            for blocker, label in ([] if ok else detail[:2]):
                why[(short(key, parent), short(blocker, parent), label[:70])] += 1
        seams.append({'pair': [low, high], 'against': sum(up.values()), 'items': len(up),
                      'references': {f'{low} -> {high}': graph[(low, high)], f'{high} -> {low}': graph[(high, low)]},
                      'top_items': [{'item': short(k, parent), 'references': n} for k, n in up.most_common(8)],
                      'why': [{'item': i, 'blocked_by': b, 'in': l} for (i, b, l), _n in why.most_common(6)]})
    seams.sort(key=lambda s: -s['against'])
    return {'parent': '::'.join(parent), 'before': before, 'after': after, 'rounds': rounds,
            'moved': len(world.moved),
            'modules': {name: {'items': r['items'], 'lines': r['lines'], 'kinds': dict(r['kinds']),
                               'top_source_modules': dict(r['files'].most_common(12))}
                        for name, r in sorted(modules.items(), key=lambda kv: -kv[1]['lines'])},
            'vocabulary_edges': vocab_edges, 'vocabulary_cycles': vocab_cycles,
            'vocabulary_upward': up_from_vocabulary,
            'largest_component_after': sorted(largest), 'seams': seams,
            'moves': [{'item': short(k, parent), 'kind': world.kind(k), 'from': f, 'to': t}
                      for k, (f, t) in sorted(world.moved.items())]}


def markdown(r):
    b, a = r['before'], r['after']
    lines = [f"# Vocabulary simulation: `{r['parent']}`", '',
             f"Before: component {b['largest_component']}, {b['against_fresh_layering']} references against the layering.",
             f"After: component {a['largest_component']}, {a['against_fresh_layering']} against a fresh layering; "
             f"{r['moved']} items moved over {len(r['rounds'])} rounds {r['rounds']}.", '',
             '## Vocabulary modules', '', '| module | items | lines | kinds | top source modules |', '|---|---|---|---|---|']
    for name, m in r['modules'].items():
        kinds = ', '.join(f'{k} {v}' for k, v in sorted(m['kinds'].items(), key=lambda kv: -kv[1]))
        src = ', '.join(f'`{k}` {v}' for k, v in list(m['top_source_modules'].items())[:5])
        lines.append(f"| `{name}` | {m['items']} | {m['lines']} | {kinds} | {src} |")
    lines += ['', f"Cycles among vocabulary modules: {r['vocabulary_cycles'] or 'none'}", '',
              'Vocabulary modules naming modules above them (should be none): ' +
              (', '.join(f'`{k}` {v}' for k, v in r['vocabulary_upward'].items()) or 'none'), '',
              '## Remaining seams', '']
    for s in r['seams'][:50]:
        lines.append(f"- `{s['pair'][0]}` -> `{s['pair'][1]}`: {s['against']} refs in {s['items']} items: " +
                     ', '.join(f"`{i['item']}`" for i in s['top_items'][:4]))
        for w in s['why'][:3]:
            lines.append(f"  - `{w['item']}` held by `{w['blocked_by']}` ({w['in']})")
    return '\n'.join(lines) + '\n'


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__.split('\n')[0])
    parser.add_argument('lib', type=Path)
    parser.add_argument('parents', nargs='+')
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--root', type=Path, default=Path.cwd())
    parser.add_argument('--host-order', type=Path, nargs='*', default=None,
                        help='stack files breaking ties; layer by the host supertraits that compile')
    parser.add_argument('--exclude-designed', type=Path, nargs='*', default=[],
                        help='design documents whose backticked item names stay for their packets')
    parser.add_argument('--impl-limit', type=int, default=250,
                        help='a type whose impls together exceed this many lines is behaviour and stays')
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
        import re
        excluded = set()
        for doc in args.exclude_designed:
            excluded |= {n for n in re.findall(r'`([A-Za-z_][A-Za-z0-9_]*)(?:[<`(:])', doc.read_text())
                         if n[:1].isupper() or '_' in n}
        order = None
        if args.host_order is not None:
            from coupling_graph import layering as greedy  # noqa: F401
            from module_coupling import coupling
            children, _edges, _kinds = coupling(None, parent, None, model)
            stack = []
            for p in args.host_order:
                data = json.loads(p.read_text())
                if data['parent'] == parent_path:
                    stack = [m for layer in data['layers'] for m, _purpose in layer['modules']]
            order = host_order(args.root, parent, children, stack)
        world = VocabularyWorld(model, parent, all_units, helpers, order, frozenset(excluded))
        world.impl_limit = args.impl_limit
        result = simulate(world)
        target = args.out / parent[-1]
        target.mkdir(parents=True, exist_ok=True)
        (target / 'simulation.json').write_text(json.dumps(result, indent=1, sort_keys=True) + '\n', encoding='utf-8')
        (target / 'simulation.md').write_text(markdown(result), encoding='utf-8')
        summary[parent_path] = {'before': result['before'], 'after': result['after'], 'moved': result['moved'],
                                'modules': {k: (v['items'], v['lines']) for k, v in result['modules'].items()},
                                'vocabulary_cycles': result['vocabulary_cycles'],
                                'vocabulary_upward': len(result['vocabulary_upward']), 'seams': len(result['seams'])}
    print(json.dumps(summary, indent=1, sort_keys=True))
    return 0


if __name__ == '__main__':
    sys.exit(main())
