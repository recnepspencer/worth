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


def impl_header(generics, self_type, where):
    return from_child(f'impl{generics} {self_type}' + (f' {where}' if where else '')) + ' {\n'


def method_door(header, name):
    """An `impl` block holding `<name>_for_test`, or None when the method is not found."""
    _attributes, generics, self_type, where, body = header
    for method, signature, _text in method_signatures(body):
        if method != name:
            continue
        signature = signature.rstrip(';').strip()
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
    stream = tokens(rest)
    matching = pairs(stream)
    k, turbofish = 0, ''
    if stream and stream[0].value == '<':
        depth, j = 0, 0
        while True:
            depth += {'<': 1, '>': -1}.get(stream[j].value, 0)
            if depth == 0:
                break
            j += 1
        generics = rest[stream[0].start:stream[j].end]
        if re.search(r'\bimpl\b', rest[:rest.find(')') + 1]) is None:
            names = [n for n in generic_arguments(generics)[1:-1].split(', ') if n and not n.startswith("'")]
            turbofish = '::<' + ', '.join(names) + '>' if names else ''
        k = j + 1
    open_index = k
    close_index = matching[open_index]
    inner = without_patterns(rest[stream[open_index].start:stream[close_index].end])[1:-1]
    receiver, arguments = None, []
    for param in split_top(inner):
        param = param.strip()
        if not param:
            continue
        head = param.split(':')[0].strip()
        if head in ('self', '&self', '&mut self', 'mut self') or re.match(r"&\s*'\w+\s+(mut\s+)?self$", head):
            receiver = 'self'
        else:
            arguments.append(re.sub(r'^mut\s+', '', head))
    return turbofish, (receiver, arguments)


def split_top(text):
    parts, depth, current = [], 0, ''
    for index, ch in enumerate(text):
        if ch == '>' and index and text[index - 1] == '-':
            current += ch  # the arrow of a return type, not a closing bracket
            continue
        depth += {'(': 1, '[': 1, '<': 1, '{': 1, ')': -1, ']': -1, '>': -1, '}': -1}.get(ch, 0)
        if ch == ',' and depth == 0:
            parts.append(current)
            current = ''
        else:
            current += ch
    parts.append(current)
    return parts


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
        body = text[stream[k].end:stream[matching[k]].start]
        for index, part in enumerate(split_top(body)):
            part = re.sub(r'(?s)^\s*(?:#\s*\[[^\]]*\]\s*|///[^\n]*\n\s*)*', '', part).strip()
            part = re.sub(r'^pub(?:\s*\([^)]*\))?\s+', '', part)
            if not part:
                continue
            if stream[k].value == '{':
                field, _colon, ty = part.partition(':')
                fields[field.strip()] = ty.strip()
            else:
                fields[str(index)] = part
    if tail := re.search(r'\bwhere\b(.*)$', text[stream[matching[k]].end:] if k < len(stream) and stream[k].value == '(' else ''):
        where = 'where' + tail.group(1).rstrip(';').rstrip()
    return {'name': name, 'generics': generics, 'where': where, 'fields': fields}


def self_type(shape):
    return shape['name'] + generic_arguments(shape['generics'])


def field_door(shape, field, mutable):
    ty = from_child(shape['fields'][field])
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
    """Directory rustc resolves `node`'s child module files in."""
    inline, current = [], node
    while current.declaration is not None and current.declaration.body is not None:
        inline.append(current.names[-1])
        current = index[current.names[:-1]]
    source = Path(current.source)
    path_mounted = current.declaration is not None and getattr(current.declaration, 'path', None)
    mod_rs = source.name in ('mod.rs', 'lib.rs') or path_mounted
    base = source.parent if mod_rs else source.parent / source.stem
    return base.joinpath(*reversed(inline))


def door_file_path(node, index):
    return module_directory(node, index) / 'court_doors_for_test.rs'


def mount_edit(node):
    """(source, change) that mounts the door module at the end of the module, once."""
    # `own_text` drops child module declarations, the door's own included.
    if re.search(r'(?m)^\s*mod court_doors_for_test;', node.text):
        return None
    end = node.offset + len(node.text)
    return node.source, (end, end, '\nmod court_doors_for_test;\n')
