"""Inherent impls of library types inside moving test modules become sealed extension traits.

An inherent impl cannot leave its type's crate, and a test door that calls test helpers
cannot stay in production. A trait the moved code owns can be implemented for a library type
in the testkit (a local trait for a foreign type), so each such impl becomes
`pub trait <Module><Type>ForTest { signatures }` plus `impl <Module><Type>ForTest for <Type>`,
with the same method bodies. Moved modules that call the methods import the trait.
"""
from dataclasses import dataclass

from rust_source import tokens
from src_kept_impls import impl_trait, library_definition

GENERIC_SKIP = ('dyn', 'mut', 'const', 'where')


@dataclass
class Door:
    start: int
    end: int
    trait: str
    methods: tuple
    replacement: str


def camel(name):
    return ''.join(part[:1].upper() + part[1:] for part in name.split('_') if part)


def header_parts(text):
    """(attributes, generics, self type, where clause, body) of an impl item."""
    stream = tokens(text)
    impl = next(k for k, t in enumerate(stream) if t.value == 'impl')
    attributes = text[:stream[impl].start]
    i, generics = impl + 1, ''
    if stream[i].value == '<':
        depth, j = 0, i
        while True:
            depth += {'<': 1, '>': -1}.get(stream[j].value, 0)
            if depth == 0:
                break
            j += 1
        generics, i = text[stream[i].start:stream[j].end], j + 1
    depth, k = 0, i
    while not (stream[k].value in ('{', 'where') and depth == 0):
        depth += {'<': 1, '>': -1, '(': 1, ')': -1}.get(stream[k].value, 0)
        k += 1
    self_type = text[stream[i].start:stream[k].start].strip()
    where = ''
    if stream[k].value == 'where':
        w = k
        while stream[k].value != '{':
            k += 1
        where = text[stream[w].start:stream[k].start].strip()
    body = text[stream[k].end:text.rfind('}')]
    return attributes, generics, self_type, where, body


def generic_arguments(generics):
    """`<'a, T: Bound, const N: usize>` -> `<'a, T, N>`."""
    if not generics:
        return ''
    inner, parts, depth, current = generics[1:-1], [], 0, ''
    for ch in inner:
        if ch in '<([':
            depth += 1
        elif ch in '>)]':
            depth -= 1
        if ch == ',' and depth == 0:
            parts.append(current)
            current = ''
        else:
            current += ch
    if current.strip():
        parts.append(current)
    names = []
    for part in parts:
        words = part.strip().split()
        if words and words[0] == 'const':
            names.append(words[1].rstrip(':'))
        elif words:
            names.append(words[0].split(':')[0])
    return '<' + ', '.join(names) + '>'


def method_signatures(body):
    """(name, signature without visibility, method text without visibility) per fn in an impl body."""
    from src_scope import scope_items
    import re
    result = []
    for attributes, _vis, keyword, name, start, end in scope_items(body):
        if keyword != 'fn':
            continue
        method = body[start:end]
        method = re.sub(r'(?m)^(\s*(?:#\s*\[[^\]]*\]\s*)*)pub(?:\s*\([^)]*\))?\s+', r'\1', method, count=1)
        stream = tokens(method)
        depth, brace = 0, None
        for t in stream:
            if t.value in ('(', '['):
                depth += 1
            elif t.value in (')', ']'):
                depth -= 1
            elif t.value == '{' and depth == 0:
                brace = t.start
                break
        if brace is None:
            continue
        result.append((name, method[:brace].rstrip() + ';', method))
    return result


def door_traits(model):
    """Set `record.doors` on every moving record; return all doors."""
    from src_hooks import impl_target
    doors = []
    for names, record in model.records.items():
        record.doors = []
        if record.extracted or record.foreign:
            continue
        own = record.node.own_text()
        used = {}
        for item in record.items:
            if item.keyword != 'impl':
                continue
            text = own[item.start:item.end]
            target = impl_target(text)
            if impl_trait(text) is not None or not target or not library_definition(model, names, target):
                continue
            base = camel(names[-1]) + target + 'ForTest'
            used[base] = used.get(base, 0) + 1
            trait = base if used[base] == 1 else f'{base}{used[base]}'
            door = convert(text, trait)
            if door is not None:
                record.doors.append(Door(item.start, item.end, trait, door[0], door[1]))
        doors.extend(record.doors)
    return doors


def convert(text, trait):
    attributes, generics, self_type, where, body = header_parts(text)
    methods = method_signatures(body)
    if not methods:
        return None
    arguments = generic_arguments(generics)
    signatures = '\n'.join('    ' + sig.strip() for _n, sig, _m in methods)
    bodies = '\n\n'.join(method.strip('\n') for _n, _s, method in methods)
    where_clause = (' ' + where) if where else ''
    replacement = (f'{attributes}/// Test doors on a library type, owned by the moved courts.\n'
                   f'pub trait {trait}{generics}{where_clause} {{\n{signatures}\n}}\n\n'
                   f'{attributes}impl{generics} {trait}{arguments} for {self_type}{where_clause} {{\n{bodies}\n}}')
    return tuple(n for n, _s, _m in methods), replacement
