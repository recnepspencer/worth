#!/usr/bin/env python3
"""Map one hub type's impl blocks: which fields each method touches, how, and whom it calls.

Usage:
    python scripts/query_split/hub_map.py <lib.rs> <module::path::Type> [--expand FIELD ...]
        [--share 0.5] [--parent module::path] --out <dir>

For every production impl block of the type (test doors and test-gated blocks excluded), each
method is read once:
- `self.f` and, for an `--expand`ed path `f` (or `f.g`), `self.f.g` are field paths; each occurrence is a
  write (assigned), a mutable borrow (`&mut`), an interior mutation (followed by `.lock()`,
  `.borrow_mut()`, an atomic store or a collection mutator) or a read;
- `self.m(` and `Self::m(` are calls to the type's own methods; `self.f.m(` on an expanded
  field is a call on that field's type, listed as the pseudo-field `f.m()`.
Each method sits in the top-level module (under `--parent`, by default the primary graph, else
under `domain_computation`) whose file holds its impl block; inside the type's own top-level
module, in that module's child, so the home module's own clusters show.

The field-to-module affinity counts, per field, the methods of each module touching it. A
field whose dominant module holds at least `--share` of its methods belongs to that module's
candidate subsystem; the rest are shared. A method joins the subsystem whose fields it touches
most; its calls into another subsystem's methods are the narrow traits a split would need.
Writes `hub-<Type>.json` and `hub-<Type>.md`.
"""
import argparse
from collections import Counter, defaultdict
import json
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parent))
sys.path.insert(0, str(Path(__file__).resolve().parent.parent / 'test_topology'))

from item_mentions import lookup  # noqa: E402
from module_graph import LibraryModel  # noqa: E402
from ownership_units import header_parts  # noqa: E402
from rust_source import pairs, tokens  # noqa: E402
from src_families import test_gated  # noqa: E402
from src_scope import scope_items  # noqa: E402

MUTATORS = {'lock', 'borrow_mut', 'get_mut', 'insert', 'push', 'push_back', 'pop', 'remove',
            'fetch_add', 'fetch_sub', 'fetch_max', 'fetch_update', 'store', 'swap', 'write',
            'take', 'replace', 'clear', 'entry', 'extend', 'retain', 'set', 'get_or_init',
            'compare_exchange', 'drain', 'try_lock', 'append', 'truncate', 'iter_mut',
            'values_mut', 'as_mut'}
ASSIGN = {'=', '+=', '-=', '*=', '/=', '|=', '&=', '^=', '<<=', '>>=', '%='}
DOOR = 'court_doors_for_test'


def bucket(module, parent, home=None):
    """The top-level module under `parent` (else under domain_computation, else the crate);
    inside the type's own top-level module (`home`), its child, so the home splits too."""
    if module[:len(parent)] == parent and len(module) > len(parent):
        top = module[len(parent)]
        if top == home and len(module) > len(parent) + 1:
            return top + '::' + module[len(parent) + 1]
        return top
    root = ('domain_computation',)
    if module[:1] == root and len(module) > 1:
        return 'dc::' + module[1]
    return 'crate::' + (module[0] if module else '')


def joined(stream, k):
    """Compound operators arrive as single punctuation tokens; rejoin `+=`, `==` and friends."""
    value = stream[k].value
    if k + 1 < len(stream) and stream[k + 1].start == stream[k].end and stream[k + 1].value == '=':
        return value + '='
    return value


def is_call(stream, k):
    """A call follows token k: `(`, or a turbofish `::<`."""
    if k >= len(stream):
        return False
    if stream[k].value == '(':
        return True
    return stream[k].value == ':' and k + 1 < len(stream) and stream[k + 1].value == ':'


def method_facts(body, expand):
    """Fields touched (with mode), own calls and expanded-field calls in one method body."""
    stream = tokens(body)
    touched, calls, field_calls = defaultdict(set), set(), set()
    for k, token in enumerate(stream):
        if token.kind == 'ident' and token.value == 'Self' and k + 3 < len(stream) \
                and stream[k + 1].value == ':' and stream[k + 2].value == ':' \
                and stream[k + 3].kind == 'ident':
            if is_call(stream, k + 4):
                calls.add(stream[k + 3].value)
            continue
        if not (token.kind == 'ident' and token.value == 'self'):
            continue
        if k + 2 >= len(stream) or stream[k + 1].value != '.' or stream[k + 2].kind != 'ident':
            continue
        name, end = stream[k + 2].value, k + 2
        if is_call(stream, end + 1):
            calls.add(name)
            continue
        path, called = name, False
        while path in expand and end + 2 < len(stream) and stream[end + 1].value == '.' \
                and stream[end + 2].kind == 'ident':
            sub = stream[end + 2].value
            if is_call(stream, end + 3):
                field_calls.add(path + '.' + sub)
                touched[path + '.' + sub + '()'].add('read')
                called = True
                break
            path, end = path + '.' + sub, end + 2
        if called:
            continue
        # the rest of the place expression: `.field` steps, indexing
        tail = end + 1
        while tail + 1 < len(stream) and stream[tail].value == '.' and stream[tail + 1].kind == 'ident' \
                and not is_call(stream, tail + 2):
            tail += 2
        before = stream[k - 1].value if k >= 1 else ''
        before2 = stream[k - 2].value if k >= 2 else ''
        after = joined(stream, tail) if tail < len(stream) else ''
        if before == 'mut' and before2 == '&':
            touched[path].add('mut_borrow')
        elif after in ASSIGN and not (after == '=' and tail + 1 < len(stream) and stream[tail + 1].value == '='):
            touched[path].add('write')
        elif after == '.' and tail + 1 < len(stream) and stream[tail + 1].value in MUTATORS:
            touched[path].add('mutate')
        else:
            touched[path].add('read')
    return touched, calls, field_calls


def receiver(signature):
    stream = tokens(signature)
    values = [t.value for t in stream]
    if 'self' not in values:
        return 'associated'
    k = values.index('self')
    if k >= 2 and values[k - 1] == 'mut' and values[k - 2] == '&':
        return '&mut self'
    if k >= 1 and values[k - 1] == '&':
        return '&self'
    return 'self'


def hub_methods(model, key, expand, parent):
    methods, blocks = [], Counter()
    home = key[0][len(parent)] if key[0][:len(parent)] == parent and len(key[0]) > len(parent) else None
    for names in sorted(model.views):
        if DOOR in names or 'tests' in names:
            continue
        node = model.index[names]
        text = node.text
        file_text = None
        for attributes, _visibility, keyword, _name, start, end in scope_items(text):
            if keyword != 'impl' or test_gated(attributes):
                continue
            body = text[start:end]
            generics, trait_path, self_path, _header = header_parts(body)
            if not self_path or (self_path[0] in generics and len(self_path) == 1):
                continue
            target = lookup(model, names, self_path)
            if target is None or target[0] != 'item' or target[1] != key:
                continue
            trait = '::'.join(trait_path) if trait_path else None
            module = bucket(names, parent, home)
            blocks[module] += 1
            stream = tokens(body)
            matching = pairs(stream)
            opening = next(i for i, t in enumerate(stream) if t.value == '{')
            inner_start, inner_end = stream[opening].end, stream[matching[opening]].start
            inner = body[inner_start:inner_end]
            for attrs, _vis, kw, name, s, e in scope_items(inner):
                if kw != 'fn' or test_gated(attrs):
                    continue
                item = inner[s:e]
                brace = item.find('{')
                signature = item if brace < 0 else item[:brace]
                touched, calls, field_calls = method_facts(item if brace < 0 else item[brace:], expand)
                if file_text is None:
                    file_text = node.source.read_text(encoding='utf-8')
                line = file_text[:node.offset + start + inner_start + s].count('\n') + 1
                methods.append({
                    'name': name, 'module': module, 'source_module': '::'.join(names),
                    'file': str(node.source), 'trait': trait, 'receiver': receiver(signature),
                    'lines': item.count('\n') + 1,
                    'fields': {f: sorted(modes) for f, modes in sorted(touched.items())},
                    'calls': sorted(calls), 'field_calls': sorted(field_calls), 'line': line,
                })
    return methods, blocks


def analyse(methods, share):
    defined = defaultdict(set)
    for m in methods:
        defined[m['name']].add(m['module'])
    affinity = defaultdict(Counter)
    for m in methods:
        for f in m['fields']:
            affinity[f][m['module']] += 1
    owner = {}
    for f, by_module in affinity.items():
        module, n = by_module.most_common(1)[0]
        total = sum(by_module.values())
        owner[f] = module if n / total >= share and total >= 2 else None
    clusters = defaultdict(lambda: {'fields': set(), 'methods': [], 'accesses': 0})
    for f, module in owner.items():
        if module:
            clusters[module]['fields'].add(f)
    for m in methods:
        votes = Counter(owner[f] for f in m['fields'] if owner.get(f))
        home = votes.most_common(1)[0][0] if votes else None
        m['subsystem'] = home
        if home:
            clusters[home]['methods'].append(m['name'])
            clusters[home]['accesses'] += sum(1 for f in m['fields'] if owner.get(f) == home)
    by_name = defaultdict(set)
    for m in methods:
        if m['subsystem']:
            by_name[m['name']].add(m['subsystem'])
    cross = Counter()
    for m in methods:
        for c in m['calls']:
            for target in by_name.get(c, ()):
                if m['subsystem'] and target != m['subsystem']:
                    cross[(m['subsystem'], target, c)] += 1
    foreign = Counter()   # a subsystem's methods touching another subsystem's fields
    for m in methods:
        for f in m['fields']:
            if m['subsystem'] and owner.get(f) and owner[f] != m['subsystem']:
                foreign[(m['subsystem'], owner[f], f)] += 1
    return affinity, owner, clusters, cross, foreign, defined


def report(model, key, expand, share, parent):
    methods, blocks = hub_methods(model, key, expand, parent)
    affinity, owner, clusters, cross, foreign, defined = analyse(methods, share)
    ranked = sorted(clusters.items(), key=lambda kv: (-kv[1]['accesses'], kv[0]))
    return {
        'type': '::'.join(key[0] + (key[1],)), 'impl_blocks': sum(blocks.values()),
        'impl_blocks_by_module': dict(blocks.most_common()), 'methods': len(methods),
        'fields': {f: {'owner': owner[f], 'modules': dict(affinity[f].most_common()),
                       'methods': sum(affinity[f].values())} for f in sorted(affinity)},
        'clusters': [{'module': module, 'fields': sorted(c['fields']), 'methods': sorted(c['methods']),
                      'accesses': c['accesses']} for module, c in ranked],
        'cross_subsystem_calls': [{'from': a, 'to': b, 'method': c, 'sites': n}
                                  for (a, b, c), n in sorted(cross.items(), key=lambda kv: (-kv[1], kv[0]))],
        'foreign_field_uses': [{'subsystem': a, 'field_owner': b, 'field': f, 'methods': n}
                               for (a, b, f), n in sorted(foreign.items(), key=lambda kv: (-kv[1], kv[0]))],
        'method_list': methods,
    }


def markdown(result, top=12):
    name = result['type'].split('::')[-1]
    out = [f"# Hub map: `{name}`", '',
           f"- impl blocks: {result['impl_blocks']}; methods: {result['methods']}",
           '- impl blocks by module: ' + ', '.join(f'{m} {n}' for m, n in list(result['impl_blocks_by_module'].items())[:20]),
           '', '## Fields, by the modules whose methods touch them', '',
           '| field | owner | methods | modules (methods touching) |', '|---|---|---|---|']
    for f, row in sorted(result['fields'].items(), key=lambda kv: -kv[1]['methods']):
        modules = ', '.join(f'{m} {n}' for m, n in list(row['modules'].items())[:6])
        out.append(f"| `{f}` | {row['owner'] or 'shared'} | {row['methods']} | {modules} |")
    out += ['', '## Candidate subsystems (heaviest first)', '']
    for c in result['clusters'][:top]:
        out.append(f"- **{c['module']}**: {len(c['methods'])} methods, {c['accesses']} owned-field "
                   f"accesses; fields {', '.join('`' + f + '`' for f in c['fields'])}")
    out += ['', '## Cross-subsystem calls (would become narrow traits)', '']
    for row in result['cross_subsystem_calls'][:40]:
        out.append(f"- {row['from']} -> {row['to']}: `{row['method']}` ({row['sites']})")
    out += ['', '## Fields used outside their subsystem', '']
    for row in result['foreign_field_uses'][:40]:
        out.append(f"- {row['subsystem']} uses {row['field_owner']}'s `{row['field']}` ({row['methods']} methods)")
    return '\n'.join(out) + '\n'


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__.split('\n')[0])
    parser.add_argument('lib')
    parser.add_argument('type')
    parser.add_argument('--expand', nargs='*', default=[])
    parser.add_argument('--share', type=float, default=0.5)
    parser.add_argument('--parent', default='domain_computation::primary_graph')
    parser.add_argument('--root', type=Path, default=Path('.'))
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args(argv)
    segments = tuple(args.type.split('::'))
    model = LibraryModel(args.lib, args.root.resolve())
    found = lookup(model, segments[:-1], (segments[-1],))
    if found is None or found[0] != 'item':
        raise SystemExit('type not found: ' + args.type)
    result = report(model, found[1], set(args.expand), args.share, tuple(args.parent.split('::')))
    args.out.mkdir(parents=True, exist_ok=True)
    name = found[1][1]
    (args.out / f'hub-{name}.json').write_text(json.dumps(result, indent=1, default=sorted) + '\n',
                                              encoding='utf-8', newline='\n')
    (args.out / f'hub-{name}.md').write_text(markdown(result), encoding='utf-8', newline='\n')
    print(json.dumps({'type': result['type'], 'impl_blocks': result['impl_blocks'],
                      'methods': result['methods'], 'subsystems': len(result['clusters'])}))
    return 0


if __name__ == '__main__':
    sys.exit(main())
