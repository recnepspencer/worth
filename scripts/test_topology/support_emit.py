"""Emit canonical support facades and mechanically public test-only declarations."""
from collections import defaultdict
import json

from cargo_inventory import aliases, dependency_rows
from court_plan_dependencies import local_dependency
from rust_source import code, crate_uses
from rust_items import items
from module_bindings import use_leaves
from rust_source import tokens
from source_rewrite import relpath, rewritten_file
from module_tree import Node
from support_visibility import promote
from cfg_context import rebind
from court_packing import first_fit_decreasing


def layout_support(model, kits):
    contexts = {}
    while True:
        changed = False
        for record in model.canonical:
            text, _receipts = model.bindings.rewrite(record['helper'], record, record['support_owner'])
            active = any(item.keyword not in ('inner-attribute', 'use') or
                         (item.keyword == 'use' and bool(use_leaves(tokens(text[item.start:item.end])[next(
                             i for i, t in enumerate(tokens(text[item.start:item.end])) if t.value == 'use') + 1:-1])))
                         for item in items(text))
            changed |= record.get('active', True) != active
            record['active'] = active
            contexts[id(record)] = text
        if not changed:
            break
    # Keep a responsibility subtree whole unless it alone exceeds the ceiling.
    families = defaultdict(list)
    for record in model.canonical:
        if record.get('active') and record['support_owner'] and '-testkit' in record['support_owner']:
            families[record['support_owner']].append(record)
    for name, records in sorted(families.items()):
        def areas(population, prefix=()):
            size = sum(len(r['helper'].splitlines()) + 12 for r in population)
            if size <= 15000:
                return [{'path': prefix, 'lines': size, 'units': population}]
            grouped = defaultdict(list)
            for record in population:
                route = record['route']
                key = route[:len(prefix) + 1] if len(route) > len(prefix) else route
                grouped[key].append(record)
            if len(grouped) == 1 and next(iter(grouped)) == prefix:
                raise ValueError('indivisible support exceeds ceiling: ' + name + '/' + '/'.join(prefix))
            return [a for path, group in sorted(grouped.items()) for a in areas(group, path)]
        pieces = first_fit_decreasing(areas(records), 15000)
        if len(pieces) > 1:
            for piece in pieces:
                suffix = '-'.join(piece['areas'][0]['path']).replace('_', '-')
                for area in piece['areas']:
                    for record in area['units']:
                        record['support_owner'] = name + '-' + suffix
    roots = defaultdict(list)
    for record in model.canonical:
        name = record['support_owner']
        if not name or not record['active']:
            continue
        is_kit = '-testkit' in name
        base = model.workspace / 'crates' / name
        directory = base / 'src' if is_kit else base / 'tests' / name.replace('-', '_') / 'support'
        record['wrapper'] = directory.joinpath(*record['route'], 'mod.rs')
        record['declarations'] = record['wrapper'].with_name('declarations.rs')
        if is_kit:
            kit = kits.setdefault(name, {'name': name, 'files': [], 'readers': set(), 'nodes': []})
            kit['readers'].update(record['readers'])
        roots[name].append(record)
    return roots


def canonical_roots(model, roots, assignments, effects, rewrites):
    dependencies = defaultdict(list)
    for name, population in sorted(roots.items()):
      for record in population:
        is_kit = '-testkit' in name
        text, receipts = model.bindings.rewrite(record['helper'], record, name)
        rewrites.extend(receipts)
        text, receipts = model.executables.rewrite(text, record['node'].source)
        rewrites.extend(receipts)
        text, receipts = rebind(text, record['features'], True)
        rewrites.extend({**r, 'source': str(record['node'].source), 'scope': '::'.join(record['node'].names)} for r in receipts)
        node = Node(record['node'].source, record['node'].names, text)
        text, receipts = rewritten_file(node, record['declarations'], record['owner'], assignments)
        rewrites.extend(receipts)
        text, receipts = promote(text, 'pub' if is_kit else 'pub(crate)')
        rewrites.extend({**r, 'source': str(record['node'].source), 'scope': '::'.join(record['node'].names)} for r in receipts)
        effects.write(record['declarations'], text)
        package = record['owner']['cargo']
        names = aliases(package) | {package['name'].replace('-', '_')}
        used = crate_uses(record['helper'], names)
        rows = dependency_rows(package, used)
        if package['name'].replace('-', '_') in used:
            rows.append(local_dependency(package['name'], record['owner']['root']))
        dependencies[name].extend(rows)
    for name, population in sorted(roots.items()):
        emit_facades(name, population, model.workspace, effects)
        for record in population:
            source = record['declarations']
            content = effects.writes[source.relative_to(effects.root).as_posix()]['content']
            available = {other.replace('-', '_'): other for other in roots if other != name and '-testkit' in other}
            for alias in crate_uses(content, set(available)):
                other = available[alias]
                dependencies[name].append(local_dependency(other, model.workspace / 'crates' / other))
    return dependencies


def emit_facades(name, population, workspace, effects):
    is_kit = '-testkit' in name
    base = workspace / 'crates' / name
    directory = base / 'src' if is_kit else base / 'tests' / name.replace('-', '_') / 'support'
    scopes = {(): []}
    for record in population:
        route = record['route']
        for n in range(len(route) + 1):
            scopes.setdefault(route[:n], [])
        visibility = 'pub' if is_kit else 'pub(crate)'
        scopes[route].append('#[path = "declarations.rs"]\nmod declarations;\n' + visibility + ' use self::declarations::*;\n')
    for route in sorted(scopes):
        if not route and not is_kit:
            continue
        source = (directory / 'lib.rs') if not route and is_kit else directory.joinpath(*route, 'mod.rs')
        text = ''.join(scopes[route])
        for child in sorted(p for p in scopes if len(p) == len(route) + 1 and p[:-1] == route):
            target = directory.joinpath(*child, 'mod.rs')
            record = next((r for r in population if r['route'] == child), None)
            attributes = rebind(record['attributes'], record['features'], True)[0] if record else ''
            text += attributes + '#[allow(dead_code, unused_imports)]\n#[path = ' + json.dumps(relpath(target, source.parent)) + ']\n' + ('pub' if is_kit else 'pub(crate)') + ' mod ' + child[-1] + ';\n'
        effects.write(source, text)


def binary_support_imports(name, population):
    result = ''
    seen = set()
    for record in population:
        first = record['route'][0]
        if first in seen:
            continue
        seen.add(first)
        target = record['wrapper']
        for _ in record['route'][1:]:
            target = target.parent.parent / 'mod.rs'
        result += '#[allow(dead_code, unused_imports)]\n#[path = ' + json.dumps(name.replace('-', '_') + '/support/' + first + '/mod.rs') + ']\npub(crate) mod ' + first + ';\n'
    return result
