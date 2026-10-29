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
        if token.kind != 'ident' or (i and stream[i - 1].value in (':', '$')):
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
        destination = bindings.lookup(record, path, path[0] in ('crate', 'self', 'super'))
        if destination:
            canonical = destination[0]['canonical']
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
