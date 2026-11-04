"""Missing doors behind call sites that already name them (E0599).

`no method named `<m>_for_test` found for T` (or `no function or associated item named ..`)
at a court or testkit site means the site was rewritten but its door was never written. The
door is placed as E0624/E0616 would place it: a delegating method door in the module of the
one inherent `impl T` that defines `fn m`; otherwise a field door when T's struct has field
`m` (`m_mut` for a mutable door, `m_into` for a by-value door). The site is left as it is. A
name with no single owner stays unplaced.
"""
from collections import defaultdict
import re

from src_door_members_text import field_door, method_door, struct_shape
from src_door_traits import header_parts
from src_door_writes import struct_declaration
from src_scope import scope_items

MISSING = re.compile(r'no (?:method|function or associated item) named `(\w+)_for_test` found for '
                     r'(?:mutable reference|reference|struct|enum|union) `&?(?:mut )?(?:[\w:]*::)?(\w+)')
AUTHORITY = re.compile(r'Authority|authority|\bmint\b')
_INHERENT = {}


def inherent_impls(index, type_name):
    """[(module node, impl header parts)] of every inherent `impl` of `type_name`."""
    if id(index) not in _INHERENT:
        table = defaultdict(list)
        for node in index.values():
            if node.source is None or (node.names and node.names[-1] == 'court_doors_for_test'):
                continue
            own = node.own_text()
            for _a, _v, keyword, _n, start, end in scope_items(own):
                if keyword != 'impl':
                    continue
                try:
                    header = header_parts(own[start:end])
                except (IndexError, StopIteration):
                    continue  # not an impl block (`impl Trait` in a type position)
                self_type = re.match(r'\s*(?:[\w]+::)*(\w+)', header[2])
                if self_type and not re.search(r'\bfor\b', header[2]):
                    table[self_type.group(1)].append((node, header))
        _INHERENT[id(index)] = table
    return _INHERENT[id(index)].get(type_name, [])


def missing(plan, root, diagnostic, site, source, span):
    match = MISSING.search(diagnostic['message'])
    if match is None:
        return False
    name, type_name = match.groups()
    found = [(node, header, text) for node, header in inherent_impls(plan.index, type_name)
             for text in [method_door(header, name)] if text is not None]
    if len(found) == 1:
        node, header, text = found[0]
        plan.door(node.names, ('method', name, header[2]), text)
        if AUTHORITY.search(header[2] + ' ' + name):
            plan.authority.add('::'.join(node.names) + f'::{header[2]}::{name}')
        return True
    if found:
        return False
    declared = struct_declaration(plan.index, type_name)
    shape = struct_shape(declared[1]) if declared else None
    if shape is None:
        return False
    for field, mutable, kind in ((name, False, 'field'), (name[:-4], True, 'field_mut'),
                                 (name[:-5], 'into', 'field_into')):
        suffix = {False: '', True: '_mut', 'into': '_into'}[mutable]
        if name == field + suffix and field in shape['fields']:
            plan.door(declared[0].names, (kind, field, shape['name']), field_door(shape, field, mutable))
            if AUTHORITY.search(shape['name']):
                plan.authority.add('::'.join(declared[0].names) + f'::{shape["name"]}.{field}')
            return True
    return False
