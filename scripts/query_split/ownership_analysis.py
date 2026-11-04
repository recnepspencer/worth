#!/usr/bin/env python3
"""Ownership analysis of one module's top-level children against a draft target stack.

Usage:
    python scripts/query_split/ownership_analysis.py <lib.rs> <stack.json> --out <dir> [--pairs 10]

Plain Python over the source (no cargo). Writes `analysis.json` and `analysis.md`:
1. the stack, each placement justified by the module's purpose (from the stack file) and its
   measured references below and above it, with the greedy layering's rank beside it;
2. every type unit (`ownership_units`) referenced against the stack, classified movable,
   behaviour-coupled or pinned at its destination, and every function, constant and module
   referenced against the stack, with counts;
3. a ruling sheet for the heaviest mutual pairs: references each way, the units crossing each
   way, the options with their cost (nest, move named units, contract trait at the seam) and
   a default;
4. an estimate of the component and the references against the stack left if every default
   ruling is taken, and how many units move by script versus by hand.
"""
import argparse
from collections import Counter, defaultdict
import json
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parent))

from coupling_graph import components, layering  # noqa: E402
from module_coupling import coupling  # noqa: E402
from module_graph import LibraryModel  # noqa: E402
from ownership_units import TYPE_KINDS, Classifier, against_stack, stack_order, units  # noqa: E402

HAND_TRAIT_LIMIT = 12


def item_key(key):
    parts = tuple(key.split('::'))
    return parts[:-1], parts[-1]


def short(key, parent):
    prefix = '::'.join(parent) + '::'
    return key[len(prefix):] if key.startswith(prefix) else key


def module_lines(model, parent, children):
    lines = Counter()
    for names in model.views:
        if names[:len(parent)] == parent and len(names) > len(parent) and names[len(parent)] in children:
            lines[names[len(parent)]] += model.production_lines(names)
    return lines


def justification(stack, edges, order, greedy):
    position = {name: i for i, name in enumerate(order)}
    greedy_rank = {name: len(greedy) - 1 - i for i, name in enumerate(greedy)}
    weights = {edge: len(refs) for edge, refs in edges.items()}
    rows = []
    for layer in stack['layers']:
        for module, purpose in layer['modules']:
            below = sorted(((w, b) for (a, b), w in weights.items() if a == module and position[b] < position[module]),
                           reverse=True)[:3]
            above = sorted(((w, b) for (a, b), w in weights.items() if a == module and position[b] > position[module]),
                           reverse=True)[:3]
            users = sorted(((w, a) for (a, b), w in weights.items() if b == module), reverse=True)[:3]
            drift = greedy_rank[module] - position[module]
            rows.append({'module': module, 'layer': layer['name'], 'rank': position[module],
                         'greedy_rank': greedy_rank[module], 'purpose': purpose,
                         'uses_below': [f'{b} {w}' for w, b in below],
                         'uses_above': [f'{b} {w}' for w, b in above],
                         'used_by': [f'{a} {w}' for w, a in users],
                         'greedy_disagrees': abs(drift) > len(order) // 4})
    return rows


def classify_units(model, parent, order, all_units, edges, kinds):
    position = {name: i for i, name in enumerate(order)}
    classifier = Classifier(model, parent, order, all_units)
    against = against_stack(edges, kinds, order)
    typed, other = [], []
    for key, entry in sorted(against.items()):
        if entry['kind'] not in TYPE_KINDS:
            other.append({'item': short(key, parent), 'kind': entry['kind'], 'owner': entry['owner'],
                          'references': entry['refs'], 'from': dict(sorted(entry['from'].items()))})
            continue
        destination = min(entry['from'], key=lambda m: position[m])
        verdict, detail = classifier.judge(item_key(key), destination)
        row = {'unit': short(key, parent), 'kind': entry['kind'], 'owner': entry['owner'],
               'destination': destination, 'references': entry['refs'],
               'from': dict(sorted(entry['from'].items())), 'class': verdict}
        unit = all_units.get(item_key(key))
        row['impls'] = len(unit.impls) if unit else 0
        if verdict == 'movable':
            row['carries'] = sorted(short('::'.join(k[0] + (k[1],)), parent) for k in detail)
        elif verdict == 'behaviour-coupled':
            by_impl = defaultdict(set)
            for label, node, item, item_kind in detail:
                by_impl[label].add(f'{short(item, parent)} [{item_kind}]')
            row['needs_above'] = {label: sorted(items) for label, items in sorted(by_impl.items())}
            row['blocked_by'] = sorted(short(item, parent) for item in
                                       classifier.roots.get((item_key(key), destination), set()))
            row['direct_blockers'] = sorted({short(item, parent) for _label, _node, item, kind in detail
                                             if '(' not in str(kind)})
        else:
            row['reason'] = detail
        typed.append(row)
    return typed, other


def hubs(all_units, parent, limit=25):
    """Types whose impl blocks spread over the most of the parent's top-level modules."""
    rows = []
    for unit in all_units.values():
        if unit.key[0][:len(parent)] != parent or len(unit.key[0]) == len(parent):
            continue
        hosts = Counter(impl.module[len(parent)] for impl in unit.impls
                        if impl.module[:len(parent)] == parent and len(impl.module) > len(parent))
        foreign = {m: n for m, n in hosts.items() if m != unit.key[0][len(parent)]}
        if foreign:
            rows.append({'unit': short(unit.label, parent), 'owner': unit.key[0][len(parent)],
                         'impl_blocks': len(unit.impls), 'impl_lines': sum(i.lines for i in unit.impls),
                         'modules_hosting_impls': len(hosts), 'elsewhere': dict(sorted(foreign.items()))})
    rows.sort(key=lambda r: (-r['modules_hosting_impls'], -r['impl_blocks'], r['unit']))
    return rows[:limit]


def blockers(typed, field='blocked_by'):
    """Non-type items above a destination that couple units, by how many units they hold back."""
    held = defaultdict(list)
    for unit in typed:
        for item in unit.get(field, []):
            held[item].append(unit['unit'])
    return [{'item': item, 'units_held': len(units), 'references_held':
             sum(u['references'] for u in typed if u['unit'] in set(units))}
            for item, units in sorted(held.items(), key=lambda kv: (-len(kv[1]), kv[0]))]


def pair_sheet(edges, order, typed, other, lines, count):
    position = {name: i for i, name in enumerate(order)}
    weights = {edge: len(refs) for edge, refs in edges.items()}
    mutual = []
    for (a, b), w in weights.items():
        if a < b and weights.get((b, a)):
            mutual.append((w + weights[(b, a)], a, b))
    sheet = []
    for _total, a, b in sorted(mutual, reverse=True)[:count]:
        low, high = (a, b) if position[a] < position[b] else (b, a)
        up_units = [u for u in typed if u['owner'] == high and low in u['from']]
        up_other = [o for o in other if o['owner'] == high and low in o['from']]
        movable = [u for u in up_units if u['class'] == 'movable']
        hand = [u for u in up_units if u['class'] != 'movable'] + up_other
        up_refs = sum(u['from'][low] for u in up_units) + sum(o['from'][low] for o in up_other)
        down_items = Counter(short(k, ()) for _s, k in edges.get((high, low), ()))
        # Nesting the lower module under the higher one makes every other user of the lower
        # module below the higher one reference against the stack.
        exposed = sum(w for (x, y), w in weights.items() if y == low and x != high and position[x] < position[high])
        options = {
            'nest': {'nested': low, 'under': high, 'lines_moved': lines[low],
                     'other_users_left_below': exposed,
                     'cost': f'move {lines[low]} lines of {low} under {high}; {exposed} references from '
                             f'other modules below {high} would then point up'},
            'move_units': {'script': len(movable), 'references': sum(u['from'][low] for u in movable),
                           'units': [u['unit'] for u in movable]},
            'contract_trait': {'seam': f'a contract owned by {low}, implemented in {high}',
                               'items': len(hand), 'references': up_refs - sum(u['from'][low] for u in movable),
                               'units_and_items': [u.get('unit') or u.get('item') for u in hand]},
        }
        if not hand:
            default = f'move the {len(movable)} movable units down by script'
            ruling = 'move'
        elif len(hand) <= HAND_TRAIT_LIMIT or exposed > up_refs:
            default = (f'move the {len(movable)} movable units by script; a contract owned by {low} '
                       f'for the {len(hand)} items that stay behaviour-bound in {high}')
            ruling = 'move+trait'
        else:
            default = f'nest {low} under {high} ({lines[low]} lines); its other users then go through {high}'
            ruling = 'nest'
        sheet.append({'pair': [low, high], 'references': {f'{low} -> {high}': weights.get((low, high), 0),
                                                         f'{high} -> {low}': weights.get((high, low), 0)},
                      'against_stack': {'direction': f'{low} -> {high}', 'references': up_refs,
                                        'movable': len(movable), 'behaviour_coupled':
                                            sum(1 for u in up_units if u['class'] == 'behaviour-coupled'),
                                        'pinned': sum(1 for u in up_units if u['class'] == 'pinned'),
                                        'functions_constants_modules': len(up_other)},
                      'crossing_up': [{'item': u['unit'], 'class': u['class'], 'references': u['from'][low]}
                                      for u in sorted(up_units, key=lambda u: -u['from'][low])] +
                                     [{'item': o['item'], 'class': o['kind'], 'references': o['from'][low]}
                                      for o in sorted(up_other, key=lambda o: -o['from'][low])],
                      'crossing_down': [{'item': k, 'references': n} for k, n in down_items.most_common(12)],
                      'options': options, 'default': default, 'ruling': ruling})
    return sheet


def estimate(edges, kinds, order, typed, other, sheet, parent, hub_rows):
    """Component and references against the stack left after the defaults (and after moving
    every movable unit everywhere besides)."""
    position = {name: i for i, name in enumerate(order)}
    prefix = '::'.join(parent) + '::'

    def remaining(removed, merged):
        rename = {}
        for low, high in merged:
            rename[low] = high
        graph = defaultdict(int)
        against = 0
        for (a, b), refs in edges.items():
            a2, b2 = rename.get(a, a), rename.get(b, b)
            left = [(s, k) for s, k in refs if (a, k) not in removed]
            if a2 == b2 or not left:
                continue
            graph[(a2, b2)] += len(left)
            if position[a2] < position[b2]:
                against += len(left)
        nodes = sorted({rename.get(n, n) for n in order})
        largest = max(len(c) for c in components(nodes, dict(graph)))
        return largest, against

    removed, merged, script, hand = set(), [], 0, 0
    for row in sheet:
        low, high = row['pair']
        if row['ruling'] == 'nest':
            merged.append((low, high))
            continue
        for unit in row['options']['move_units']['units']:
            removed.add((low, prefix + unit))
            script += 1
        for item in row['options']['contract_trait']['units_and_items'] if row['ruling'] == 'move+trait' else []:
            removed.add((low, prefix + item))
            hand += 1
    pairs_only = remaining(removed, merged)
    everywhere = set(removed)
    movable_all = [u for u in typed if u['class'] == 'movable']
    for unit in movable_all:
        for low in unit['from']:
            everywhere.add((low, prefix + unit['unit']))
    all_moved = remaining(everywhere, merged)
    hub_names = {h['unit'] for h in hub_rows[:5]}
    with_hubs = set(everywhere)
    for unit in typed:
        if unit['unit'] in hub_names:
            for low in unit['from']:
                with_hubs.add((low, prefix + unit['unit']))
    hubs_inverted = remaining(with_hubs, merged)
    return {'defaults_only': {'largest_component': pairs_only[0], 'references_against_stack': pairs_only[1],
                              'units_by_script': script, 'items_by_hand': hand, 'nests': len(merged)},
            'defaults_and_every_movable_unit': {'largest_component': all_moved[0],
                                                'references_against_stack': all_moved[1],
                                                'units_by_script': len(movable_all),
                                                'items_by_hand': hand +
                                                sum(1 for u in typed if u['class'] != 'movable'
                                                    and not any((low, prefix + u['unit']) in removed
                                                                for low in u['from'])) +
                                                sum(1 for o in other if not any((low, prefix + o['item']) in removed
                                                                                for low in o['from'])),
                                                'nests': len(merged)},
            'also_five_widest_hubs_behind_contracts': {'hubs': sorted(hub_names),
                                                       'largest_component': hubs_inverted[0],
                                                       'references_against_stack': hubs_inverted[1]}}


def markdown(result):
    parent = result['parent']
    out = [f'# Ownership analysis: `{parent}`', '', '## Draft target stack (bottom to top)', '']
    layer = None
    for row in result['stack']:
        if row['layer'] != layer:
            layer = row['layer']
            out += ['', f'### {layer}', '']
        flag = ' **greedy disagrees** (greedy rank %d)' % row['greedy_rank'] if row['greedy_disagrees'] else ''
        out.append(f"- `{row['module']}` (rank {row['rank']}): {row['purpose']}; uses below "
                   f"{', '.join(row['uses_below']) or 'nothing'}; above {', '.join(row['uses_above']) or 'nothing'}; "
                   f"used by {', '.join(row['used_by']) or 'nothing'}.{flag}")
    s = result['summary']
    out += ['', '## Units referenced against the stack', '',
            f"- {s['units']} units: {s['movable']} movable, {s['behaviour_coupled']} behaviour-coupled, "
            f"{s['pinned']} pinned; {s['unit_references']} references",
            f"- {s['other_items']} functions, constants and modules: {s['other_references']} references", '',
            '| unit | owner -> destination | refs | class | detail |', '|---|---|---|---|---|']
    for u in sorted(result['units'], key=lambda u: (-u['references'], u['unit'])):
        if u['class'] == 'movable':
            detail = f"carries {len(u['carries'])}" if u['carries'] else ''
        elif u['class'] == 'behaviour-coupled':
            roots = u.get('blocked_by', [])
            detail = (f"{len(u['needs_above'])} impl(s) need items above; blocked by {len(roots)}: "
                      + ', '.join(f'`{r.rsplit("::", 1)[-1]}`' for r in roots[:4]) + (' …' if len(roots) > 4 else ''))
        else:
            detail = u['reason']
        out.append(f"| `{u['unit']}` | {u['owner']} -> {u['destination']} | {u['references']} | {u['class']} | {detail} |")
    out += ['', '### Types whose impls live across many modules', '',
            'A unit moves with all its inherent impls, so these types cannot move, or be split into',
            'another crate, until their impls are regrouped or turned into owned extension traits.', '',
            '| type | owner | impl blocks | impl lines | modules hosting impls |', '|---|---|---|---|---|']
    for h in result['hubs']:
        out.append(f"| `{h['unit']}` | {h['owner']} | {h['impl_blocks']} | {h['impl_lines']} | {h['modules_hosting_impls']} |")
    out += ['', '### Direct blockers', '',
            'Non-type items above the destination that a coupled unit\'s own declaration or impls call.', '',
            '| item | units held | their references |', '|---|---|---|']
    for b in result['direct_blockers'][:30]:
        out.append(f"| `{b['item']}` | {b['units_held']} | {b['references_held']} |")
    out += ['', '### What holds the behaviour-coupled units (transitively)', '',
            'Each non-type item above a destination that some coupled unit (directly or through a',
            'coupled type it needs) depends on; inverting the top rows frees the most units.', '',
            '| item | units held | their references |', '|---|---|---|']
    for b in result['blockers'][:20]:
        out.append(f"| `{b['item']}` | {b['units_held']} | {b['references_held']} |")
    out += ['', '### Functions, constants and modules referenced against the stack', '',
            '| item | kind | owner | refs | from |', '|---|---|---|---|---|']
    for o in sorted(result['other'], key=lambda o: (-o['references'], o['item'])):
        out.append(f"| `{o['item']}` | {o['kind']} | {o['owner']} | {o['references']} | "
                   f"{', '.join(f'{k} {v}' for k, v in o['from'].items())} |")
    out += ['', '## Ruling sheet: heaviest mutual pairs', '']
    for row in result['pairs']:
        low, high = row['pair']
        a = row['against_stack']
        out += [f'### `{low}` ↔ `{high}`', '',
                f"- references: " + ', '.join(f'{k} {v}' for k, v in row['references'].items()),
                f"- against the stack ({a['direction']}): {a['references']} references; {a['movable']} movable units, "
                f"{a['behaviour_coupled']} behaviour-coupled, {a['pinned']} pinned, "
                f"{a['functions_constants_modules']} functions/constants/modules",
                '- crossing up: ' + ', '.join(f"`{c['item'].rsplit('::', 1)[-1]}` ({c['class']}, {c['references']})"
                                              for c in row['crossing_up'][:14]) +
                (' …' if len(row['crossing_up']) > 14 else ''),
                '- crossing down (top): ' + ', '.join(f"`{c['item'].rsplit('::', 1)[-1]}` {c['references']}"
                                                      for c in row['crossing_down'][:8]),
                f"- option nest: {row['options']['nest']['cost']}",
                f"- option move units: {row['options']['move_units']['script']} units by script "
                f"({row['options']['move_units']['references']} references)",
                f"- option contract trait: {row['options']['contract_trait']['seam']}, "
                f"{row['options']['contract_trait']['items']} items ({row['options']['contract_trait']['references']} references)",
                f"- **default**: {row['default']}", '']
    e = result['estimate']
    out += ['## Estimate', '',
            f"- defaults for the pairs above: largest component {e['defaults_only']['largest_component']}, "
            f"{e['defaults_only']['references_against_stack']} references against the stack; "
            f"{e['defaults_only']['units_by_script']} units by script, {e['defaults_only']['items_by_hand']} items by hand, "
            f"{e['defaults_only']['nests']} nests",
            f"- defaults plus every movable unit moved: largest component "
            f"{e['defaults_and_every_movable_unit']['largest_component']}, "
            f"{e['defaults_and_every_movable_unit']['references_against_stack']} references against the stack; "
            f"{e['defaults_and_every_movable_unit']['units_by_script']} units by script, "
            f"{e['defaults_and_every_movable_unit']['items_by_hand']} items by hand",
            f"- and the five widest hub types ({', '.join('`' + h.rsplit('::', 1)[-1] + '`' for h in e['also_five_widest_hubs_behind_contracts']['hubs'])}) "
            f"behind contracts: largest component {e['also_five_widest_hubs_behind_contracts']['largest_component']}, "
            f"{e['also_five_widest_hubs_behind_contracts']['references_against_stack']} references against the stack", '']
    return '\n'.join(out) + '\n'


def analyse(lib, stack, repository, pair_count, model=None, all_units=None):
    parent = tuple(stack['parent'].split('::'))
    model = model or LibraryModel(lib, repository)
    if all_units is None:
        all_units, _orphans = units(model)
    children, edges, kinds = coupling(lib, parent, repository, model)
    order, _layer_of = stack_order(stack)
    missing, extra = sorted(set(children) - set(order)), sorted(set(order) - set(children))
    duplicated = sorted(m for m, n in Counter(order).items() if n > 1)
    if missing or extra or duplicated:
        raise SystemExit(f'stack mismatch: missing {missing}, unknown {extra}, duplicated {duplicated}')
    weights = {edge: len(refs) for edge, refs in edges.items()}
    greedy = layering(children, weights)
    typed, other = classify_units(model, parent, order, all_units, edges, kinds)
    held = blockers(typed)
    direct = blockers(typed, 'direct_blockers')
    lines = module_lines(model, parent, children)
    sheet = pair_sheet(edges, order, typed, other, lines, pair_count)
    position = {n: i for i, n in enumerate(order)}
    largest_now = max(len(c) for c in components(children, weights))
    against_now = sum(w for (a, b), w in weights.items() if position[a] < position[b])
    return {'version': 1, 'parent': stack['parent'], 'modules': len(children),
            'now': {'largest_component': largest_now, 'references_against_stack': against_now},
            'stack': justification(stack, edges, order, greedy), 'units': typed, 'other': other,
            'pairs': sheet, 'estimate': estimate(edges, kinds, order, typed, other, sheet, parent,
                                                 hubs(all_units, parent)),
            'blockers': held, 'direct_blockers': direct, 'hubs': hubs(all_units, parent),
            'summary': {'units': len(typed),
                        'movable': sum(1 for u in typed if u['class'] == 'movable'),
                        'behaviour_coupled': sum(1 for u in typed if u['class'] == 'behaviour-coupled'),
                        'pinned': sum(1 for u in typed if u['class'] == 'pinned'),
                        'unit_references': sum(u['references'] for u in typed),
                        'other_items': len(other), 'other_references': sum(o['references'] for o in other),
                        'blocking_items': len(held)}}


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('lib', type=Path)
    parser.add_argument('stacks', type=Path, nargs='+')
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--pairs', type=int, default=10)
    parser.add_argument('--root', type=Path, default=Path.cwd())
    args = parser.parse_args(argv)
    model = LibraryModel(args.lib, args.root.resolve())
    all_units, _orphans = units(model)
    for path in args.stacks:
        stack = json.loads(path.read_text(encoding='utf-8'))
        result = analyse(args.lib, stack, args.root.resolve(), args.pairs, model, all_units)
        target = args.out / stack['parent'].rsplit('::', 1)[-1]
        target.mkdir(parents=True, exist_ok=True)
        (target / 'analysis.json').write_text(json.dumps(result, indent=1, sort_keys=True) + '\n',
                                              encoding='utf-8', newline='\n')
        (target / 'analysis.md').write_text(markdown(result), encoding='utf-8', newline='\n')
        print(json.dumps({'parent': stack['parent'], 'now': result['now'], **result['summary'],
                          'estimate': result['estimate']}, sort_keys=True))
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
