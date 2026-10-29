"""Derive helper compilation readers through resolved old lexical references."""
from collections import defaultdict

from rust_source import tokens


def references(text, record, bindings):
    found = {}
    paths = list(record['imports'])
    stream = tokens(text)
    i = 0
    while i < len(stream):
        token = stream[i]
        if token.kind != 'ident' or ((i and stream[i - 1].value == '$') or (i >= 2 and stream[i - 1].value == ':' and stream[i - 2].value == ':')):
            i += 1
            continue
        names, j = [token.value], i + 1
        while j + 2 < len(stream) and [stream[j].value, stream[j + 1].value] == [':', ':'] and stream[j + 2].kind == 'ident':
            names.append(stream[j + 2].value)
            j += 3
        paths.append(names)
        i = j
    for path in paths:
        path = path[:-1] if path and path[-1] == '*' else path
        if not path:
            continue
        resolved = bindings.resolve(record, path)
        if resolved and resolved[0] in ('item', 'case'):
            canonical = resolved[1]['canonical']
            found[id(canonical)] = canonical
    for symbol in bindings.trait_symbols(record).values():
        canonical = symbol[1]['canonical']
        found[id(canonical)] = canonical
    return list(found.values())


def compilation_inputs(model, units):
    result = {}
    edges = {id(record): references(record['helper'], record, model.bindings) for record in model.canonical}
    attach_implementations(model, edges)
    for unit in units:
        record = model.record(unit)
        queue = [record['canonical']] + references(record['test'], record, model.bindings)
        visited, selected = set(), []
        while queue:
            helper = queue.pop()
            if id(helper) in visited:
                continue
            visited.add(id(helper))
            selected.append(helper)
            queue.extend(edges[id(helper)])
        result[unit['key']] = selected
    assign_unreferenced(model, units, edges, result)
    return result


def assign_readers(model, units):
    owner_courts = defaultdict(set)
    inputs = compilation_inputs(model, units)
    for unit in units:
        owner_courts[unit['owner']['cargo']['name']].add(unit['court'])
        for helper in inputs[unit['key']]:
            helper['readers'].add(unit['court'])
    for record in model.canonical:
        # Unreferenced declarations still leave the product test tree intact.
        if not record['readers']:
            courts = owner_courts[record['owner']['cargo']['name']]
            if courts:
                record['readers'].add(min(courts))
        family = record['owner']['cargo']['name'].removeprefix(model.stem + '-').removeprefix('worth-')
        record['support_owner'] = model.stem + '-testkit-' + family if len(record['readers']) > 1 else next(iter(record['readers']), None)


def attach_implementations(model, edges):
    from rust_items import items
    for record in model.canonical:
        for item in items(record['helper']):
            if item.keyword != 'impl' or not item.body:
                continue
            stream = tokens(record['helper'][item.start:item.body[0] - 1])
            start = next(i for i, t in enumerate(stream) if t.value == 'impl') + 1
            if start < len(stream) and stream[start].value == '<':
                depth = 1
                start += 1
                while start < len(stream) and depth:
                    depth += (stream[start].value == '<') - (stream[start].value == '>')
                    start += 1
            depth, receiver = 0, start
            for i in range(start, len(stream)):
                depth += (stream[i].value == '<') - (stream[i].value == '>')
                if not depth and stream[i].value == 'for':
                    receiver = i + 1
                    break
            for offset in sorted({start, receiver}):
                if offset >= len(stream) or stream[offset].kind != 'ident':
                    continue
                names, i = [stream[offset].value], offset + 1
                while i + 2 < len(stream) and [stream[i].value, stream[i + 1].value] == [':', ':']:
                    names.append(stream[i + 2].value)
                    i += 3
                target = model.bindings.lookup(record, names)
                if target:
                    canonical = target[0]['canonical']
                    if canonical is not record:
                        edges[id(canonical)].append(record)


def has_meaning(record):
    from rust_items import items
    if '_meaning' not in record:
        record['_meaning'] = any(i.keyword not in ('use', 'inner-attribute') for i in items(record['helper']))
    return record['_meaning']


def assign_unreferenced(model, units, edges, result):
    assigned = {id(r) for population in result.values() for r in population}
    for record in model.canonical:
        if id(record) in assigned or not has_meaning(record):
            continue
        candidates = [u for u in units if u['owner'] is record['owner']]
        if not candidates:
            continue
        def distance(unit):
            left, right = unit['node'].names, record['node'].names
            prefix = 0
            for a, b in zip(left, right):
                if a != b:
                    break
                prefix += 1
            return (unit['target'] != record['target'], -prefix, unit['owner_area'], unit['key'])
        unit = min(candidates, key=distance)
        queue = [record]
        while queue:
            source = queue.pop()
            if id(source) in assigned:
                continue
            assigned.add(id(source))
            result[unit['key']].append(source)
            queue.extend(edges[id(source)])
