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
        if len(names) > 1:
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


def assign_readers(model, units):
    owner_courts = defaultdict(set)
    edges = {id(record): references(record['helper'], record, model.bindings) for record in model.canonical}
    for unit in units:
        owner_courts[unit['owner']['cargo']['name']].add(unit['court'])
        record = model.record(unit)
        queue = [record['canonical']] + references(record['test'], record, model.bindings)
        visited = set()
        while queue:
            helper = queue.pop()
            if id(helper) in visited:
                continue
            visited.add(id(helper))
            helper['readers'].add(unit['court'])
            queue.extend(edges[id(helper)])
    for record in model.canonical:
        # Unreferenced declarations still leave the product test tree intact.
        if not record['readers']:
            courts = owner_courts[record['owner']['cargo']['name']]
            if courts:
                record['readers'].add(min(courts))
        family = record['owner']['cargo']['name'].removeprefix(model.stem + '-').removeprefix('worth-')
        record['support_owner'] = model.stem + '-testkit-' + family if len(record['readers']) > 1 else next(iter(record['readers']), None)
