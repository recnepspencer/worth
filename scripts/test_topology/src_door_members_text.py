"""Text of the sealed member doors and where they are mounted.

Each owning module gets a `court_doors_for_test` child (`use super::*;`), so the doors see
the module's private members. A method door repeats the method's signature under
`<name>_for_test` and delegates through `Self::<name>`; a field door borrows the field; a
constructor door builds the struct from its fields in declaration order.
"""
from pathlib import Path
import re

from rust_source import pairs, tokens
from src_door_traits import generic_arguments, method_signatures, without_patterns

DOC = '    #[doc(hidden)]\n'


def from_child(text):
    """Text copied from a module, re-rooted for its `court_doors_for_test` child."""
    text = re.sub(r'\b((?:super::)+)', r'super::\1', text)
    return re.sub(r'(?<![\w:])self::', 'super::', text)


def without_defaults(generics):
    """`<T, U = X, const N: usize = 3>` -> `<T, U, const N: usize>`: defaults are not allowed on
    impl or fn generics."""
    if not generics:
        return generics
    from src_door_sites import Source
    source = Source(generics)
    out, depth, skipping = [], 0, False
    for k, token in enumerate(source.stream):
        value = token.value
        arrow = value == '>' and k and source.value(k - 1) == '-' and source.adjacent(k - 1, k)
        if value in ('<', '(', '['):
            depth += 1
        elif value in ('>', ')', ']') and not arrow:
            depth -= 1
        if depth == 1 and value == '=' and not skipping:
            skipping = True
            continue
        if skipping and ((depth == 1 and value == ',') or depth == 0):
            skipping = False
        if not skipping:
            out.append(token)
    text, previous = '', None
    for token in out:
        if previous is not None and token.start > previous.end and not generics[previous.end:token.start].strip():
            text += ' '
        text += token.value
        previous = token
    return text.replace(' ,', ',').replace(' >', '>').replace('< ', '<')


def impl_header(generics, self_type, where):
    return from_child(f'impl{without_defaults(generics)} {self_type}' + (f' {where}' if where else '')) + ' {\n'


def method_door(header, name):
    """An `impl` block holding `<name>_for_test`, or None when the method is not found."""
    _attributes, generics, self_type, where, body = header
    for method, signature, _text in method_signatures(body):
        if method != name:
            continue
        signature = signature.rstrip(';').strip()
        # Attributes on the method (`#[doc(hidden)]`, `#[expect(..)]`) stay with the method.
        signature = re.sub(r'^(?:#\s*\[(?:[^\[\]]|\[[^\]]*\])*\]\s*)+', '', signature)
        match = re.match(r'((?:(?:const|async|unsafe)\s+)*)fn\s+' + name + r'\b', signature)
        if match is None:
            return None
        qualifiers = match.group(1)
        door_signature = signature[:match.end()] + '_for_test' + signature[match.end():]
        call_generics, params = call_parts(signature[match.end():])
        receiver, arguments = params
        call = f'Self::{name}{call_generics}(' + ', '.join(([receiver] if receiver else []) + arguments) + ')'
        if 'async' in qualifiers:
            call += '.await'
        if 'unsafe' in qualifiers:
            call = f'unsafe {{ {call} }}'
        door_signature = door_signature.replace('const fn', 'fn', 1)
        return (impl_header(generics, self_type, where) + DOC +
                f'    pub {from_child(door_signature)} {{\n        {call}\n    }}\n}}')
    return None


def call_parts(rest):
    """(turbofish, (receiver, argument names)) for a signature tail `<G>(params) -> R`."""
    from src_door_sites import Source, split_top as split_parts
    source = Source(rest)
    k, turbofish = 0, ''
    if source.value(0) == '<':
        depth, j = 0, 0
        while True:
            value = source.value(j)
            if value == '<':
                depth += 1
            elif value == '>' and not (source.value(j - 1) == '-' and source.adjacent(j - 1, j)):
                depth -= 1
            if depth == 0:
                break
            j += 1
        generics = source.span_text(0, j)
        params_text = source.span_text(j + 1, source.close[j + 1]) if source.value(j + 1) == '(' else ''
        if 'impl' not in {t.value for t in tokens(params_text)}:
            names = [n for n in generic_arguments(generics)[1:-1].split(', ') if n and not n.startswith("'")]
            turbofish = '::<' + ', '.join(names) + '>' if names else ''
        k = j + 1
    receiver, arguments = None, []
    for index, (a, b) in enumerate(split_parts(source, k)):
        colon = next((m for m in range(a, b + 1) if source.value(m) == ':' and source.value(m + 1) != ':'
                      and source.value(m - 1) != ':'), None)
        head = source.span_text(a, colon - 1) if colon is not None else source.span_text(a, b)
        if re.fullmatch(r"(?:&\s*(?:'\w+\s+)?)?(?:mut\s+)?self", head.strip()):
            receiver = 'self'
            continue
        name = re.sub(r'^mut\s+', '', head.strip())
        arguments.append(name if re.fullmatch(r'[A-Za-z_]\w*', name) else f'_arg{index}')
    return turbofish, (receiver, arguments)


def struct_shape(text):
    """Generics, where clause, and named (or positional) fields of a struct item."""
    stream = tokens(text)
    start = next((k for k, t in enumerate(stream) if t.value == 'struct'), None)
    if start is None:
        return None
    name, i = stream[start + 1].value, start + 2
    generics = ''
    if i < len(stream) and stream[i].value == '<':
        depth, j = 0, i
        while True:
            depth += {'<': 1, '>': -1}.get(stream[j].value, 0)
            if depth == 0:
                break
            j += 1
        generics, i = text[stream[i].start:stream[j].end], j + 1
    matching = pairs(stream)
    where = ''
    k = i
    while k < len(stream) and stream[k].value not in ('{', '(', ';'):
        k += 1
    if k > i and stream[i].value == 'where':
        where = text[stream[i].start:stream[k].start].strip()
    fields = {}
    if k < len(stream) and stream[k].value in ('{', '('):
        from src_door_sites import Source, split_top as split_parts
        source = Source(text)
        for index, (a, b) in enumerate(split_parts(source, k)):
            while source.value(a) == '#' and source.value(a + 1) == '[':
                a = source.close[a + 1] + 1  # attributes (doc comments are already gone)
            if source.value(a) == 'pub':
                a += 1
                if source.value(a) == '(':
                    a = source.close[a] + 1
            if a > b:
                continue
            if stream[k].value == '{':
                fields[source.value(a)] = source.span_text(a + 2, b)
            else:
                fields[str(index)] = source.span_text(a, b)
    if tail := re.search(r'\bwhere\b(.*)$', text[stream[matching[k]].end:] if k < len(stream) and stream[k].value == '(' else ''):
        where = 'where' + tail.group(1).rstrip(';').rstrip()
    return {'name': name, 'generics': generics, 'where': where, 'fields': fields}


def self_type(shape):
    return shape['name'] + generic_arguments(shape['generics'])


def field_door(shape, field, mutable):
    """Read door (`&`), mutable door (`mutable=True`), or by-value door (`mutable='into'`)."""
    ty = from_child(shape['fields'][field])
    if mutable == 'into':
        suffix, receiver, borrow = '_into_for_test', 'self', ''
    else:
        suffix, receiver, borrow = ('_mut_for_test', '&mut self', '&mut ') if mutable else ('_for_test', '&self', '&')
    return (impl_header(shape['generics'], self_type(shape), shape['where']) + DOC +
            f'    pub fn {field}{suffix}({receiver}) -> {borrow}{ty} {{\n        {borrow}self.{field}\n    }}\n}}')


def constructor_door(shape, order=None):
    """`new_for_test` over the struct's fields, in the order of the first literal it replaces."""
    fields = {name: shape['fields'][name] for name in (order or shape['fields'])}
    positional = all(name.isdigit() for name in fields)
    params = ', '.join(f'{"field" + n if positional else n}: {from_child(t)}' for n, t in fields.items())
    body = ('Self(' + ', '.join('field' + n for n in fields) + ')') if positional else \
        ('Self { ' + ', '.join(fields) + ' }')
    return (impl_header(shape['generics'], self_type(shape), shape['where']) + DOC +
            f'    pub fn new_for_test({params}) -> Self {{\n        {body}\n    }}\n}}')


def module_directory(node, index):
    """(directory rustc resolves `node`'s child module files in, path-mounted file stem or None)."""
    inline, current = [], node
    while current.declaration is not None and current.declaration.body is not None:
        inline.append(current.names[-1])
        current = index[current.names[:-1]]
    source = Path(current.source)
    path_mounted = current.declaration is not None and getattr(current.declaration, 'path', None)
    mod_rs = source.name in ('mod.rs', 'lib.rs') or path_mounted
    base = source.parent if mod_rs else source.parent / source.stem
    stem = source.stem if path_mounted and source.name != 'mod.rs' and not inline else None
    return base.joinpath(*reversed(inline)), stem


def door_file_path(node, index):
    """A path-mounted module shares its directory with its parent, so its door file is named
    for it (`<stem>_court_doors_for_test.rs`) and mounted by `#[path]`."""
    directory, stem = module_directory(node, index)
    return directory / (f'{stem}_court_doors_for_test.rs' if stem else 'court_doors_for_test.rs')


def mount_edit(node, index=None):
    """(source, change) that mounts the door module at the end of the module, once."""
    if re.search(r'(?m)^\s*mod court_doors_for_test;', node.text):
        return None
    path = door_file_path(node, index) if index is not None else None
    attribute = f'#[path = "{path.name}"]\n' if path is not None and path.name != 'court_doors_for_test.rs' else ''
    end = node.offset + len(node.text)
    return node.source, (end, end, f'\n{attribute}mod court_doors_for_test;\n')
