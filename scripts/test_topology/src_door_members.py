"""Member doors: sealed inherent `<name>_for_test` doors for private methods and fields.

From `cargo check` diagnostics over the courts and testkits:
- E0624 (private method or associated function) gets `pub fn <m>_for_test` delegating with
  the same signature, in the `court_doors_for_test` child of the method's module;
- E0616 (private field) gets `pub fn <f>_for_test(&self) -> &T`, and `<f>_mut_for_test`
  where the court assigns to the field or borrows it mutably (E0596 on a later round);
- E0451 / E0423 (struct literal over private fields) gets `pub fn new_for_test(..) -> Self`.
The call sites in the moved code are rewritten at the diagnostic spans; fields read as
`(*expr.f_for_test())` so the place keeps its type. Anything the scan cannot place exactly is
reported, not guessed.
"""
from collections import defaultdict
import json
from pathlib import Path
import re

from module_tree import read_tree
from rust_source import pairs, replace_spans, tokens
from src_door_constructors import constructed_struct, literal_rewrite
from src_door_members_text import (constructor_door, door_file_path, field_door, method_door,
                                   mount_edit, struct_shape)
from src_door_traits import header_parts
from src_emit_production import own_offset
from src_scope import scope_items

LIB = 'workspaces/worth-query/crates/worth-query-execution/src/lib.rs'
WORKSPACE = 'workspaces/worth-query'
AUTHORITY = re.compile(r'Authority|authority|\bmint\b')
KEYWORDS = {'mut', 'ref', 'move', 'return', 'in', 'if', 'else', 'match', 'let', 'as', 'box', 'break', 'yield'}
FIELD = re.compile(r'field `(\w+)` of struct `(?:[\w:]*::)?(\w+)')


def diagnostics(check):
    for line in Path(check).read_text(encoding='utf-8').splitlines():
        try:
            message = json.loads(line)
        except ValueError:
            continue
        if message.get('reason') == 'compiler-message' and message['message']['level'] == 'error':
            yield message['message']


def owning_impl(index, path, offset):
    """(module node, impl text) of the impl block in `path` that contains byte `offset`."""
    best = None
    for names, node in index.items():
        if node.source is None or Path(node.source).resolve() != path:
            continue
        own = node.own_text()
        for _a, _v, keyword, _n, start, end in scope_items(own):
            s = own_offset(node, start) + node.offset
            if keyword == 'impl' and s <= offset < s + (end - start):
                if best is None or len(names) > len(best[0].names):
                    best = (node, own[start:end])
    return best


def owning_field_door(index, field):
    """(module node, struct shape) whose door file already reads `field`, when unique."""
    found = []
    for names, node in index.items():
        if names and names[-1] == 'court_doors_for_test' and f'pub fn {field}_for_test(&self)' in node.text:
            parent = index[names[:-1]]
            for _a, _v, keyword, item, start, end in scope_items(parent.own_text()):
                if keyword == 'struct':
                    shape = struct_shape(parent.own_text()[start:end])
                    if shape and field in shape['fields'] and f'impl{shape["generics"]} {shape["name"]}' in node.text:
                        found.append((parent, shape))
    return found[0] if len(found) == 1 else None


def struct_declaration(index, name):
    found = []
    for names, node in index.items():
        own = node.own_text()
        for _a, _v, keyword, item, start, end in scope_items(own):
            if keyword == 'struct' and item == name:
                found.append((node, own[start:end]))
    return found[0] if len(found) == 1 else None


def members(root, check, report, write_report):
    tree = read_tree(root / LIB)
    index = {node.names: node for node in tree.walk()}
    doors = defaultdict(dict)        # module names -> {door key: text}
    sites = defaultdict(set)         # court file -> {(start, end, replacement)}
    authority, unplaced, opened = set(), [], set()
    counts = defaultdict(int)
    for diagnostic in diagnostics(check):
        code = (diagnostic.get('code') or {}).get('code')
        primary = next((s for s in diagnostic['spans'] if s['is_primary']), None)
        if primary is None or code not in ('E0624', 'E0616', 'E0596', 'E0594', 'E0451', 'E0423'):
            continue
        site = (root / WORKSPACE / primary['file_name']).resolve()
        if code in ('E0596', 'E0594'):
            found = mutable_sites(diagnostic, site, None)
            owner = owning_field_door(index, found[0]) if found else None
            if owner is None:
                unplaced.append(diagnostic['message'])
                continue
            node, shape = owner
            doors[node.names][('field_mut', found[0], shape['name'])] = field_door(shape, found[0], True)
            sites[site].add(found[1])
            continue
        if code in ('E0451', 'E0423'):
            struct = constructed_struct(diagnostic)
            found = struct_declaration(index, struct) if struct else None
            shape = struct_shape(found[1]) if found else None
            change = literal_rewrite(site.read_bytes(), primary['byte_start'], code, shape)
            if change is None:
                unplaced.append(diagnostic['message'])
                continue
            key = ('new', 'new', struct)
            if key not in doors[found[0].names]:
                doors[found[0].names][key] = constructor_door(shape, change[3])
            sites[site].add(change[:3])
            if AUTHORITY.search(struct):
                authority.add('::'.join(found[0].names) + f'::{struct}::new')
            continue
        if code == 'E0624':
            declared = [s for c in diagnostic['children'] for s in c['spans']] + \
                [s for s in diagnostic['spans'] if not s['is_primary']]
            declared = [s for s in declared if 'worth-query-execution/src' in s['file_name']]
            if not declared:
                unplaced.append(diagnostic['message'])
                continue
            span = declared[0]
            owner = owning_impl(index, (root / WORKSPACE / span['file_name']).resolve(), span['byte_start'])
            name = re.search(r'`(\w+)`', diagnostic['message']).group(1)
            if name.endswith('_for_test'):
                # Already a sealed test door by name: it only needs to be `pub`.
                opened.add(((root / WORKSPACE / span['file_name']).resolve(), span['byte_start']))
                if AUTHORITY.search(name):
                    authority.add(span['file_name'].split('/src/', 1)[-1] + f'::{name}')
                continue
            if owner is None:
                unplaced.append(diagnostic['message'])
                continue
            node, impl_text = owner
            text = method_door(header_parts(impl_text), name)
            if text is None:
                unplaced.append(diagnostic['message'])
                continue
            doors[node.names][('method', name, header_parts(impl_text)[2])] = text
            sites[site].add((primary['byte_start'], primary['byte_end'], name + '_for_test'))
            if AUTHORITY.search(header_parts(impl_text)[2] + ' ' + name):
                authority.add('::'.join(node.names) + f'::{header_parts(impl_text)[2]}::{name}')
        else:
            match = FIELD.search(diagnostic['message'])
            if match is None:
                unplaced.append(diagnostic['message'])
                continue
            field, struct = match.group(1), match.group(2)
            found = struct_declaration(index, struct)
            if found is None:
                unplaced.append(diagnostic['message'])
                continue
            node, struct_text = found
            shape = struct_shape(struct_text)
            if shape is None or field not in shape['fields']:
                unplaced.append(diagnostic['message'])
                continue
            mutable = writes(site, primary['byte_end']) or borrowed_mut(site, primary['byte_start'])
            doors[node.names][('field', field, struct)] = field_door(shape, field, False)
            if mutable:
                doors[node.names][('field_mut', field, struct)] = field_door(shape, field, True)
            sites[site].add((primary['byte_start'], primary['byte_end'], (field, mutable)))
            if AUTHORITY.search(struct):
                authority.add('::'.join(node.names) + f'::{struct}.{field}')
    counts.update(write_doors(index, doors))
    counts['opened'] = open_declarations(opened)
    counts['call_sites'] = rewrite_sites(sites)
    data = {'constructor_doors': sorted('::'.join(m) + f'::{k[2]}::new' for m, d in doors.items() for k in d if k[0] == 'new'),
            'method_doors': sorted('::'.join(m) + f'::{k[2]}::{k[1]}' for m, d in doors.items() for k in d if k[0] == 'method'),
            'field_doors': sorted('::'.join(m) + f'::{k[2]}.{k[1]}' + (' (mut)' if k[0] == 'field_mut' else '')
                                  for m, d in doors.items() for k in d if k[0].startswith('field')),
            'authority_doors': sorted(authority), 'unplaced': sorted(set(unplaced))}
    write_report(report, 'members', data)
    return {k: len(v) for k, v in data.items()}


def open_declarations(opened):
    """Make `*_for_test` methods (already sealed doors by name) `pub` at their declaration."""
    by_file = defaultdict(set)
    for path, at in opened:
        by_file[path].add(at)
    for path, starts in by_file.items():
        data = path.read_bytes()
        for at in sorted(starts, reverse=True):
            head = re.match(rb'pub(?:\s*\([^)]*\))?\s+', data[at:])
            if head:
                data = data[:at] + b'pub ' + data[at + head.end():]
            elif data[at:].startswith((b'fn ', b'const fn', b'async fn', b'unsafe fn')):
                data = data[:at] + b'pub ' + data[at:]
        path.write_bytes(data)
    return len(opened)


def borrowed_mut(path, field_start):
    """Whether the field access starting its name at `field_start` is under `&mut`."""
    data = path.read_bytes()
    begin = expression_start(data, field_start)
    return data[:begin].rstrip().endswith(b'&mut')


def mutable_sites(diagnostic, site, doors_by_field):
    """E0596/E0594 over `(*expr.f_for_test())`: switch to `f_mut_for_test`, add its door."""
    primary = next(s for s in diagnostic['spans'] if s['is_primary'])
    data = site.read_bytes()[primary['byte_start']:primary['byte_end']]
    match = re.search(rb'\.(\w+)_for_test\(\)\)$', data)
    if match is None:
        return None
    field = match.group(1).decode()
    at = primary['byte_start'] + match.start(1)
    return field, (at, at + len(field) + len('_for_test'), field + '_mut_for_test')


def writes(path, end):
    """Whether the field access ending at byte `end` is assigned to (`=`, `+=`, ...)."""
    rest = path.read_bytes()[end:end + 8].decode('utf-8', 'ignore').lstrip()
    return bool(re.match(r'(?:[+\-*/%|&^]|<<|>>)?=(?!=)', rest))


def write_doors(index, doors):
    counts = defaultdict(int)
    for names, entries in sorted(doors.items()):
        node = index[names]
        path = door_file_path(node, index)
        existing = path.read_text(encoding='utf-8') if path.exists() else '//! Sealed test doors over this module\'s private members.\nuse super::*;\n'
        # A door already present (under any formatting) is not written again.
        present = set(re.findall(r'\bpub fn (\w+_for_test)\b', existing))
        added = []
        for key, text in sorted(entries.items()):
            names = set(re.findall(r'\bpub fn (\w+_for_test)\b', text))
            if text not in existing and not names <= present:
                added.append(text)
                present |= names
        for key in entries:
            counts[key[0]] += 1
        if added:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(existing.rstrip('\n') + '\n\n' + '\n\n'.join(added) + '\n', encoding='utf-8', newline='\n')
        edit = mount_edit(node)
        if edit is not None:
            source, change = edit
            text = Path(source).read_text(encoding='utf-8')
            Path(source).write_text(replace_spans(text, [change]), encoding='utf-8', newline='\n')
    return counts


def rewrite_sites(sites):
    count = 0
    for path, changes in sorted(sites.items()):
        text = path.read_bytes().decode('utf-8')
        data = text.encode('utf-8')
        spans = []
        # One field access is reported per target; keep one rewrite per span (mutable wins).
        fields = {}
        for start, end, replacement in changes:
            if not isinstance(replacement, str):
                field, mutable = replacement
                previous = fields.get((start, end))
                fields[(start, end)] = (field, mutable or (previous is not None and previous[1]))
        changes = {c for c in changes if isinstance(c[2], str)} | {(s, e, r) for (s, e), r in fields.items()}
        for start, end, replacement in sorted(changes, key=lambda c: (c[0], c[1])):
            if isinstance(replacement, str):
                spans.append((start, end, replacement))
                continue
            field, mutable = replacement
            begin = expression_start(data, start)
            door = f'{field}_mut_for_test' if mutable else f'{field}_for_test'
            spans.append((begin, begin, '(*'))
            spans.append((start, end, door + '())'))
        # A change inside another change's span waits for the next round (its offsets move).
        spans = [c for c in spans if not any(o[0] <= c[0] and c[1] <= o[1] and o[1] > o[0] and o != c
                                             for o in spans)]
        out = data
        for start, end, replacement in sorted(spans, key=lambda s: (s[0], s[1]), reverse=True):
            out = out[:start] + replacement.encode('utf-8') + out[end:]
            count += 1
        path.write_bytes(out)
    return count


def skip_back(text, i):
    """Index of the last code character at or before `i`, past whitespace and line comments."""
    while i >= 0:
        if text[i].isspace():
            if text[i] == '\n':
                line_start = text.rfind('\n', 0, i) + 1
                comment = text.find('//', line_start, i)
                if comment >= 0:
                    i = comment - 1
                    continue
            i -= 1
            continue
        return i
    return i


def expression_start(data, field_start):
    """Byte where the postfix expression before `.field` begins (identifiers, calls, indexes)."""
    text = data[:field_start].decode('utf-8')
    i = len(text) - 1
    while i >= 0 and text[i].isspace():
        i -= 1
    assert text[i] == '.', 'a field access follows a dot'
    i -= 1
    # Calls and indexes continue a postfix chain; a closing brace ends a block, never a chain.
    closing = {')': '(', ']': '['}
    while i >= 0:
        i = skip_back(text, i)
        if i < 0:
            break
        c = text[i]
        if c in closing:
            depth, j = 0, i
            while j >= 0:
                if text[j] in closing:
                    depth += 1
                elif text[j] in closing.values():
                    depth -= 1
                    if depth == 0:
                        break
                j -= 1
            i = j - 1
            continue  # a call or index group: its callee comes next
        elif c.isalnum() or c == '_':
            j = i
            while j >= 0 and (text[j].isalnum() or text[j] == '_'):
                j -= 1
            if text[j + 1:i + 1] in KEYWORDS:
                break
            i = j
        elif c == '?':
            i -= 1
            continue
        else:
            break
        k = skip_back(text, i)
        if k >= 1 and text[k] == ':' and text[k - 1] == ':':
            i = k - 2
            continue
        if k >= 0 and text[k] == '.' and not (k >= 1 and text[k - 1] == '.'):
            i = k - 1
            continue
        break
    return len(text[:i + 1].encode('utf-8'))
