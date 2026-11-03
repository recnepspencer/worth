"""Member doors: sealed inherent `<name>_for_test` doors for private methods and fields.

From `cargo check` diagnostics over the courts and testkits:
- E0624 (private method or associated function): `pub fn <m>_for_test` delegating with the
  same signature, in the `court_doors_for_test` child of the method's module (a method already
  named `*_for_test` is only made `pub`);
- E0616 (private field): `<f>_for_test(&self) -> &T`, read at the site as `(*expr.f_for_test())`,
  or `<f>_mut_for_test` where the site assigns or borrows mutably (E0596/E0594 later switch a
  read site), or `<f>_into_for_test(self)` where the site moves the field out (E0507);
- E0451 / E0423 (struct literal over private fields): `new_for_test(..)` in the first literal's
  field order; a functional update sets its fields through mutable doors instead.
All site analysis is on `rust_source` tokens (src_door_sites). `dry_run` writes nothing and
reports what it would do; every changed file is validated first (tokenizer balance for sites,
rustfmt for door files), and nothing is written if any validation fails.
"""
from collections import defaultdict
import json
from pathlib import Path
import re

from module_tree import read_tree
from src_door_members_text import constructor_door, field_door, method_door, struct_shape
from src_door_sites import (Source, assigned_after, borrowed_mut, enclosing_brace, field_sites,
                            is_pattern, literal_parts, path_before)
from src_door_traits import header_parts
from src_door_writes import Plan, apply_plan, open_declarations, owning_field_door, owning_impl, struct_declaration

LIB = 'workspaces/worth-query/crates/worth-query-execution/src/lib.rs'
WORKSPACE = 'workspaces/worth-query'
AUTHORITY = re.compile(r'Authority|authority|\bmint\b')
FIELD = re.compile(r'field `(\w+)` of struct `(?:[\w:]*::)?(\w+)')
NAMED = re.compile(r'of struct `(?:[\w:]*::)?(\w+)')


def diagnostics(check):
    for line in Path(check).read_text(encoding='utf-8').splitlines():
        try:
            message = json.loads(line)
        except ValueError:
            continue
        if message.get('reason') == 'compiler-message' and message['message']['level'] == 'error':
            yield message['message']


def members(root, check, report, write_report, dry_run=False):
    tree = read_tree(root / LIB)
    index = {node.names: node for node in tree.walk()}
    plan = Plan(index)
    for diagnostic in diagnostics(check):
        code = (diagnostic.get('code') or {}).get('code')
        primary = next((s for s in diagnostic['spans'] if s['is_primary']), None)
        handler = HANDLERS.get(code)
        if primary is None or handler is None:
            continue
        site = (root / WORKSPACE / primary['file_name']).resolve()
        if 'worth-query-execution/src' in str(site):
            continue  # the library's own errors are not door sites
        source = plan.source(site)
        span = (source.char_offset(primary['byte_start']), source.char_offset(primary['byte_end']))
        if not handler(plan, root, diagnostic, site, source, span):
            plan.unplaced.add(diagnostic['message'])
    problems = apply_plan(plan, dry_run)
    data = plan.summary()
    data['validation_problems'] = problems
    write_report(report, 'members', data)
    return {k: len(v) if isinstance(v, (list, set)) else v for k, v in data.items()}


def method(plan, root, diagnostic, site, source, span):
    declared = [s for c in diagnostic['children'] for s in c['spans']] + \
        [s for s in diagnostic['spans'] if not s['is_primary']]
    declared = [s for s in declared if 'worth-query-execution/src' in s['file_name']]
    name = re.search(r'`(\w+)`', diagnostic['message']).group(1)
    if not declared:
        return False
    path = (root / WORKSPACE / declared[0]['file_name']).resolve()
    offset = plan.source(path).char_offset(declared[0]['byte_start'])
    if name.endswith('_for_test'):
        plan.opened.add((path, offset))
        if AUTHORITY.search(name):
            plan.authority.add(f'{path.name}::{name}')
        return True
    owner = owning_impl(plan.index, path, offset)
    if owner is None:
        return False
    node, impl_text = owner
    header = header_parts(impl_text)
    text = method_door(header, name)
    if text is None:
        return False
    plan.door(node.names, ('method', name, header[2]), text)
    plan.change(site, (span[0], span[1], name + '_for_test'))
    if AUTHORITY.search(header[2] + ' ' + name):
        plan.authority.add('::'.join(node.names) + f'::{header[2]}::{name}')
    return True


def field(plan, root, diagnostic, site, source, span):
    match = FIELD.search(diagnostic['message'])
    found = struct_declaration(plan.index, match.group(2)) if match else None
    shape = struct_shape(found[1]) if found else None
    if shape is None or match.group(1) not in shape['fields']:
        return False
    name = match.group(1)
    located = field_sites(source, span[0])
    if located is None:
        return False
    insert, token_span, field_index, first = located
    mutable = assigned_after(source, field_index) or borrowed_mut(source, first)
    kind = 'field_mut' if mutable else 'field'
    plan.door(found[0].names, (kind, name, shape['name']), field_door(shape, name, mutable))
    plan.change(site, (insert, insert, '(*'))
    plan.change(site, (token_span[0], token_span[1], f'{name}{"_mut" if mutable else ""}_for_test())'))
    if AUTHORITY.search(shape['name']):
        plan.authority.add('::'.join(found[0].names) + f'::{shape["name"]}.{name}')
    return True


def read_door_in(source, span):
    """(token index, field) of the first `f_for_test()` read door inside a span."""
    for k, token in enumerate(source.stream):
        if span[0] <= token.start < span[1] and token.kind == 'ident' and token.value.endswith('_for_test') \
                and not token.value.endswith(('_mut_for_test', '_into_for_test')) \
                and source.value(k - 1) == '.' and source.value(k + 1) == '(' and source.value(k + 2) == ')':
            return k, token.value[:-len('_for_test')]
    return None


def mutable_read(plan, root, diagnostic, site, source, span):
    found = read_door_in(source, span)
    owner = owning_field_door(plan.index, found[1]) if found else None
    if owner is None:
        return False
    node, shape = owner
    token = source.stream[found[0]]
    plan.door(node.names, ('field_mut', found[1], shape['name']), field_door(shape, found[1], True))
    plan.change(site, (token.start, token.end, found[1] + '_mut_for_test'))
    return True


def moved_read(plan, root, diagnostic, site, source, span):
    first, last = source.index(span[0]), None
    for k, token in enumerate(source.stream):
        if token.end == span[1]:
            last = k
    if first is None or last is None or source.value(first) != '(' or source.value(first + 1) != '*' \
            or source.close.get(first) != last:
        return False
    door = last - 3
    if source.value(door - 1) != '.' or not source.value(door).endswith('_for_test'):
        return False
    name = source.value(door)[:-len('_for_test')]
    owner = owning_field_door(plan.index, name)
    if owner is None:
        return False
    node, shape = owner
    plan.door(node.names, ('field_into', name, shape['name']), field_door(shape, name, 'into'))
    expression = source.span_text(first + 2, door - 2)
    plan.change(site, (span[0], span[1], f'{expression}.{name}_into_for_test()'))
    return True


def constructed(plan, root, diagnostic, site, source, span):
    match = NAMED.search(diagnostic['message'])
    name = match.group(1) if match else None
    for child in diagnostic['children']:
        named = re.search(r'struct named `(\w+)`', child['message'])
        name = name or (named.group(1) if named else None)
    found = struct_declaration(plan.index, name) if name else None
    shape = struct_shape(found[1]) if found else None
    if shape is None:
        return False
    code = (diagnostic.get('code') or {}).get('code')
    at = source.index(span[0])
    if code == 'E0423':
        end = at
        while source.value(end + 1) == ':' and source.value(end + 2) == ':':
            end += 3
        if source.value(end + 1) != '(':
            return False
        token = source.stream[end]
        plan.door(found[0].names, ('new', 'new', shape['name']), constructor_door(shape, tuple(shape['fields'])))
        plan.change(site, (token.end, token.end, '::new_for_test'))
        return True
    brace = enclosing_brace(source, at)
    path = path_before(source, brace) if brace is not None else None
    parts = literal_parts(source, brace) if path else None
    if parts is None or is_pattern(source, path[0], brace):
        return False
    fields, base = parts
    values = [(f, v if v is not None else f) for f, v in fields]
    end = source.stream[source.close[brace]].end
    begin = source.stream[path[0]].start
    if base is None:
        if {f for f, _ in values} != set(shape['fields']):
            return False
        plan.door(found[0].names, ('new', 'new', shape['name']), constructor_door(shape, tuple(f for f, _ in values)))
        plan.change(site, (begin, end, f'{path[1]}::new_for_test({", ".join(v for _, v in values)})'))
        return True
    # A functional update: the field values first (in written order), then the base, then sets.
    lets = ''.join(f'let __{f} = {v}; ' for f, v in values)
    sets = ''.join(f'*__base.{f}_mut_for_test() = __{f}; ' for f, _ in values)
    for f, _ in values:
        plan.door(found[0].names, ('field_mut', f, shape['name']), field_door(shape, f, True))
    plan.change(site, (begin, end, f'{{ {lets}let mut __base = {base}; {sets}__base }}'))
    return True


HANDLERS = {'E0624': method, 'E0616': field, 'E0596': mutable_read, 'E0594': mutable_read,
            'E0507': moved_read, 'E0451': constructed, 'E0423': constructed}
