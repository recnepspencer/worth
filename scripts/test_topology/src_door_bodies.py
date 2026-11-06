"""Body check across the door rewrites: moved tests still have their library bodies.

The test-name and body multiset of the library at the base revision must equal the multiset
of the library and every court and testkit now. Bodies are normalized as for the move
(src_multiset) and, on top, only the member door rewrites are reversed:
`m_for_test` back to `m` for generated method doors, `(*expr.f_for_test())` and
`(*expr.f_mut_for_test())` back to `expr.f` for generated field doors, and generated
`T::new_for_test(..)` constructors back to literals (on both sides spelled with explicit
`field: value` pairs, in the door's order, so a reordered literal still differs). Any other change to a
body still changes its digest. Item doors only change paths, which the move's normalization
already reduces to their last segment.
"""
from collections import Counter
from pathlib import Path
import re
import subprocess

from rust_source import tokens
from src_multiset import normalized_bodies
import src_multiset

LIBRARY = 'workspaces/worth-query/crates/worth-query-execution/src'
CRATES = 'workspaces/worth-query/crates'


def door_files(root):
    library = root / LIBRARY
    return sorted(set(library.rglob('court_doors_for_test.rs')) | set(library.rglob('*_court_doors_for_test.rs')))


DOOR = re.compile(r'pub fn (\w+)_for_test\b[^{]*\{\s*(.*?)\s*\}\n', re.S)


def door_renames(root):
    """{generated door identifier: (original name, kind)}, read from the door files themselves:
    a door is reversed only when its body is the plain delegation the generator writes
    (`Self::m(..)`, `&self.f`, `&mut self.f`, `self.f`). A name that is a field door on one type
    and a method door on another gets the kind 'field+method', decided at each site."""
    renames = {}

    def add(door, name, kind):
        if door in renames and renames[door][1] != kind:
            kind = '+'.join(sorted(set(renames[door][1].split('+')) | {kind}))
        renames[door] = (name, kind)
    for path in door_files(root):
        for name, body in DOOR.findall(path.read_text(encoding='utf-8')):
            if re.match(rf'Self::{re.escape(name)}(::<.*?>)?\(', body, re.S):
                add(name + '_for_test', name, 'method')
            elif body == f'&self.{name}':
                add(name + '_for_test', name, 'field')
            elif name.endswith('_mut') and body == f'&mut self.{name[:-4]}':
                add(name + '_for_test', name[:-4], 'field')
            elif name.endswith('_into') and body == f'self.{name[:-5]}':
                add(name + '_for_test', name[:-5], 'into')
    return renames


def constructor_orders(root):
    """{struct name: (named, field order)} for every generated `new_for_test` door; parameters
    are read by balanced tokens, so `fn() -> T` in a type does not end the list."""
    from src_door_sites import Source, split_top
    orders = {}
    for path in door_files(root):
        text = path.read_text(encoding='utf-8')
        source = None
        for block in re.finditer(r'impl(?:<[^{]*?>)?\s+(\w+)[^{]*\{\s*#\[doc\(hidden\)\]\s*pub fn new_for_test\(', text):
            source = source or Source(text)
            params = [source.value(a) for a, _b in split_top(source, source.index(block.end() - 1))]
            named = not all(re.fullmatch(r'field\d+', p) for p in params)
            orders[block.group(1)] = (named, params)
    return orders


def split_arguments(values, open_index, close_index):
    parts, depth, current = [], 0, []
    for value in values[open_index + 1:close_index]:
        if value in ('(', '[', '{', '<'):
            depth += 1
        elif value in (')', ']', '}', '>'):
            depth -= 1
        if value == ',' and depth == 0:
            parts.append(current)
            current = []
        else:
            current.append(value)
    if current:
        parts.append(current)
    return parts


def matching_close(values, index):
    depth = 0
    for j in range(index, len(values)):
        depth += {'(': 1, '[': 1, '{': 1, ')': -1, ']': -1, '}': -1}.get(values[j], 0)
        if depth == 0:
            return j
    return None


def canonical_literals(values, orders):
    """`T::new_for_test(a, b)` and `T { x: a, y }` literals both as `T { x : a , y : y }`."""
    out, i = [], 0
    while i < len(values):
        value = values[i]
        if value == 'new_for_test' and i >= 3 and values[i - 2:i] == [':', ':'] and values[i - 3] in orders \
                and i + 1 < len(values) and values[i + 1] == '(':
            close = matching_close(values, i + 1)
            named, fields = orders[values[i - 3]]
            arguments = split_arguments(values, i + 1, close)
            if close is not None and len(arguments) == len(fields):
                out = out[:-2]
                if named:
                    out.append('{')
                    for k, (field, argument) in enumerate(zip(fields, arguments)):
                        out += ([','] if k else []) + [field, ':'] + argument
                    out.append('}')
                else:
                    out += ['('] + [v for k, a in enumerate(arguments) for v in ([','] if k else []) + a] + [')']
                i = close + 1
                continue
        if value in orders and orders[value][0] and i + 1 < len(values) and values[i + 1] == '{' \
                and not (i and values[i - 1] in ('struct', 'impl', 'enum', 'for', 'trait')):
            close = matching_close(values, i + 1)
            if close is not None:
                out += [value, '{']
                for k, part in enumerate(p for p in split_arguments(values, i + 1, close) if p):
                    if len(part) == 1 and part[0].isidentifier():
                        part = [part[0], ':', part[0]]
                    out += ([','] if k else []) + part
                out.append('}')
                i = close + 1
                continue
        out.append(value)
        i += 1
    return out


def without_doors(values, renames, orders=None):
    """Token values with the generated member door rewrites reversed (and trailing commas, which
    formatting adds, dropped)."""
    values = [v for k, v in enumerate(values) if not (v == ',' and k + 1 < len(values) and values[k + 1] in (')', ']', '}'))]
    values, drop = list(values), set()
    for i, value in enumerate(values):
        if value not in renames:
            continue
        name, kind = renames[value]
        values[i] = name
        if 'into' in kind and values[i + 1:i + 3] == ['(', ')']:
            drop |= {i + 1, i + 2}
            continue
        if 'field' not in kind or values[i + 1:i + 4] != ['(', ')', ')']:
            continue
        depth, j = 0, i + 3
        while j >= 0:
            depth += {')': 1, '(': -1}.get(values[j], 0)
            if depth == 0:
                break
            j -= 1
        if j >= 0 and j + 1 < len(values) and values[j + 1] == '*':
            drop |= {j, j + 1, i + 1, i + 2, i + 3}
    values = [v for k, v in enumerate(values) if k not in drop]
    return canonical_literals(values, orders) if orders else values


def multiset(sources, renames, orders=None):
    original = src_multiset.digest

    def door_digest(values):
        return original(without_doors(values, renames, orders))
    src_multiset.digest = door_digest
    try:
        counts = Counter()
        for text, source in sources:
            counts.update(normalized_bodies(text, source))
        return counts
    finally:
        src_multiset.digest = original


def base_sources(root, base):
    files = subprocess.run(['git', 'ls-tree', '-r', '--name-only', base, LIBRARY], cwd=root,
                           capture_output=True, text=True, check=True).stdout.split()
    for path in files:
        if path.endswith('.rs'):
            text = subprocess.run(['git', 'show', f'{base}:{path}'], cwd=root, capture_output=True,
                                  check=True).stdout.decode('utf-8')
            yield text, path


def current_sources(root):
    crates = root / CRATES
    roots = [crates / 'worth-query-execution/src'] + [
        d for d in sorted(crates.iterdir())
        if d.name.startswith(('worth-query-court-', 'worth-query-testkit-')) and 'execution' in d.name]
    for directory in roots:
        for path in sorted(directory.rglob('*.rs')):
            yield path.read_text(encoding='utf-8'), str(path.relative_to(root))


def body_check(root, base):
    renames = door_renames(root)
    orders = constructor_orders(root)
    before = multiset(base_sources(root, base), renames, orders)
    after = multiset(current_sources(root), renames, orders)
    missing, extra = before - after, after - before
    return {'tests_before': sum(before.values()), 'tests_after': sum(after.values()),
            'missing': sorted('\t'.join(k) for k in missing), 'extra': sorted('\t'.join(k) for k in extra)}
