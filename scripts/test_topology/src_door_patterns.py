"""Destructuring `let` patterns over private fields (E0451) through the parts door.

`let T { a, b: x, mut c, .. } = EXPR;` becomes `let (a, x, mut c, ..) = EXPR.into_parts_for_test();`,
the tuple in the struct's declaration order, and T's `into_parts_for_test` door is written if it
is missing. A field the pattern leaves out (`..`) is `_` when EXPR is a temporary; when EXPR is a
local it is bound as `_<field>_rest`, so it still lives to the end of the scope as it did in place.
Only `let` statements are rewritten; any other pattern stays unplaced.
"""
import re

from src_door_members_text import struct_shape
from src_door_parts import parts_door
from src_door_sites import enclosing_brace, path_before, split_top
from src_door_writes import struct_declaration

NAMED = re.compile(r'of struct `(?:[\w:]*::)?(\w+)`')
POSTFIX = {'.', ':', '?', '!'}


def pattern_entries(source, brace):
    """{field: pattern text} and whether the pattern has `..`, or None if it cannot be read."""
    entries, rest = {}, False
    for a, b in split_top(source, brace):
        if source.value(a) == '.' and source.value(a + 1) == '.' and b == a + 1:
            rest = True
        elif source.value(a + 1) == ':' and source.value(a + 2) != ':':
            entries[source.value(a)] = source.span_text(a + 2, b)
        elif a == b:
            entries[source.value(a)] = source.value(a)
        elif b == a + 1 and source.value(a) in ('mut', 'ref'):
            entries[source.value(b)] = source.span_text(a, b)
        else:
            return None
    return entries, rest


def statement_end(source, start):
    """Index of the `;` (or `else`) ending the `let` initializer that starts at token `start`."""
    k = start
    while k < len(source.stream):
        if k in source.close:
            k = source.close[k] + 1
            continue
        if source.value(k) in (';', 'else'):
            return k
        k += 1
    return None


def simple_postfix(source, a, b):
    k = a
    while k <= b:
        if k in source.close:
            k = source.close[k] + 1
            continue
        token = source.stream[k]
        if token.kind != 'ident' and source.value(k) not in POSTFIX and not source.value(k).isdigit():
            return False
        k += 1
    return True


def destructured(plan, root, diagnostic, site, source, span):
    match = NAMED.search(diagnostic['message'])
    found = struct_declaration(plan.index, match.group(1)) if match else None
    shape = struct_shape(found[1]) if found else None
    at = source.index(span[0])
    brace = enclosing_brace(source, at) if shape and at is not None else None
    path = path_before(source, brace) if brace is not None else None
    if path is None or source.value(path[0] - 1) != 'let' or source.value(source.close[brace] + 1) != '=':
        return False
    read = pattern_entries(source, brace)
    if read is None or set(read[0]) - set(shape['fields']):
        return False
    entries, rest = read
    if not rest and set(entries) != set(shape['fields']):
        return False
    first = source.close[brace] + 2
    end = statement_end(source, first)
    if end is None:
        return False
    local = end == first + 1 and source.stream[first].kind == 'ident'
    tuple_parts = [entries.get(f, f'_{f}_rest' if local else '_') for f in shape['fields']]
    plan.door(found[0].names, ('parts', 'into_parts', shape['name']), parts_door(shape))
    plan.change(site, (source.stream[path[0]].start, source.stream[source.close[brace]].end,
                       '(' + ', '.join(tuple_parts) + ',)'))
    tail = source.stream[end - 1].end
    if simple_postfix(source, first, end - 1):
        plan.change(site, (tail, tail, '.into_parts_for_test()'))
    else:
        head = source.stream[first].start
        plan.change(site, (head, head, '('))
        plan.change(site, (tail, tail, ').into_parts_for_test()'))
    return True
