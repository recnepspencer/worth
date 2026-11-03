"""The member-door plan: doors per owning module, site changes per court file, and their writes.

`apply_plan` validates every file it would change before writing any: each rewritten court or
testkit file must tokenize with balanced delimiters, and each door file must parse under
`rustfmt --edition 2024`. Door files are written rustfmt-formatted. With `dry_run` nothing is
written; the problems found are returned either way.
"""
from collections import defaultdict
from pathlib import Path
import re
import subprocess
import tempfile

from rust_source import pairs, replace_spans, tokens
from src_door_members_text import door_file_path, mount_edit, struct_shape
from src_door_sites import Source
from src_emit_production import own_offset
from src_scope import scope_items

HEADER = "//! Sealed test doors over this module's private members.\nuse super::*;\n"


class Plan:
    def __init__(self, index):
        self.index = index
        self.doors = defaultdict(dict)       # module names -> {key: door text}
        self.changes = defaultdict(set)      # file -> {(start, end, replacement)} in characters
        self.authority, self.unplaced, self.opened = set(), set(), set()
        self._sources = {}

    def source(self, path):
        if path not in self._sources:
            self._sources[path] = Source(path.read_text(encoding='utf-8'))
        return self._sources[path]

    def door(self, names, key, text):
        self.doors[names].setdefault(key, text)

    def change(self, path, change):
        self.changes[path].add(change)

    def summary(self):
        kinds = defaultdict(list)
        for names, entries in self.doors.items():
            for kind, name, owner in entries:
                kinds[kind].append('::'.join(names) + f'::{owner}::{name}')
        return {'method_doors': sorted(kinds['method']),
                'field_doors': sorted(kinds['field']) + sorted(d + ' (mut)' for d in kinds['field_mut']),
                'field_into_doors': sorted(kinds['field_into']),
                'constructor_doors': sorted(kinds['new']),
                'opened': sorted(f'{p.name}@{o}' for p, o in self.opened),
                'authority_doors': sorted(self.authority), 'unplaced': sorted(self.unplaced),
                'site_rewrites': sum(len(c) for c in self.changes.values())}


_IMPLS = {}


def owning_impl(index, path, offset):
    """(module node, impl text) of the innermost impl block in `path` holding char `offset`."""
    if id(index) not in _IMPLS:
        table = defaultdict(list)
        for names, node in index.items():
            if node.source is None:
                continue
            own = node.own_text()
            for _a, _v, keyword, _n, start, end in scope_items(own):
                if keyword == 'impl':
                    s = own_offset(node, start) + node.offset
                    table[Path(node.source).resolve()].append((s, s + (end - start), node, own[start:end]))
        _IMPLS[id(index)] = table
    best = None
    for start, end, node, text in _IMPLS[id(index)].get(path, []):
        if start <= offset < end and (best is None or len(node.names) > len(best[0].names)):
            best = (node, text)
    return best


_STRUCTS = {}


def struct_declaration(index, name):
    """(module node, struct text) of the one struct `name` declares, from a per-index table."""
    if id(index) not in _STRUCTS:
        table = defaultdict(list)
        for node in index.values():
            own = node.own_text()
            for _a, _v, keyword, item, start, end in scope_items(own):
                if keyword == 'struct':
                    table[item].append((node, own[start:end]))
        _STRUCTS[id(index)] = table
    found = _STRUCTS[id(index)].get(name, [])
    return found[0] if len(found) == 1 else None


def owning_field_door(index, field):
    """(module node, struct shape) whose door file already reads `field`, when unique."""
    found = []
    for names, node in index.items():
        if names and names[-1] == 'court_doors_for_test' and re.search(r'pub fn ' + field + r'_for_test\(\s*&self', node.text):
            parent = index[names[:-1]]
            for _a, _v, keyword, _item, start, end in scope_items(parent.own_text()):
                if keyword == 'struct':
                    shape = struct_shape(parent.own_text()[start:end])
                    if shape and field in shape['fields'] and \
                            re.search(r'impl(?:<[^{]*?>)?\s+' + shape['name'] + r'\b', node.text):
                        found.append((parent, shape))
    return found[0] if len(found) == 1 else None


def door_keys(text):
    """{(self type, door name)} of the `impl` blocks in door text."""
    keys = set()
    for block in re.finditer(r'impl(?:<[^{]*?>)?\s+([\w:]+)[^{]*\{(.*?)\n\}', text, re.S):
        keys |= {(block.group(1), n) for n in re.findall(r'\bpub fn (\w+_for_test)\b', block.group(2))}
    return keys


def door_texts(plan):
    """{door file path: (new full text, mount edit or None)} for every module given doors."""
    result = {}
    for names, entries in sorted(plan.doors.items()):
        node = plan.index[names]
        path = door_file_path(node, plan.index)
        existing = path.read_text(encoding='utf-8') if path.exists() else HEADER
        present, added = door_keys(existing), []
        for _key, text in sorted(entries.items()):
            keys = door_keys(text)
            if text not in existing and not keys <= present:
                added.append(text)
                present |= keys
        if added:
            result[path] = (existing.rstrip('\n') + '\n\n' + '\n\n'.join(added) + '\n', mount_edit(node))
    return result


def site_texts(plan):
    result = {}
    for path, changes in sorted(plan.changes.items()):
        # A change inside another change's span waits for the next round (its offsets move).
        kept = [c for c in changes if not any(o[0] <= c[0] and c[1] <= o[1] and o[1] > o[0] and o != c
                                              for o in changes)]
        result[path] = replace_spans(plan.source(path).text, sorted(kept, key=lambda c: (c[0], c[1])))
    return result


def balanced(text):
    try:
        pairs(tokens(text))
        return True
    except ValueError:
        return False


def parses(text):
    with tempfile.TemporaryDirectory() as directory:
        path = Path(directory) / 'door.rs'
        path.write_text(text, encoding='utf-8')
        result = subprocess.run(['rustfmt', '--edition', '2024', str(path)], capture_output=True, text=True)
        return result.returncode == 0, path.read_text(encoding='utf-8')


def apply_plan(plan, dry_run):
    problems, formatted = [], {}
    sites = site_texts(plan)
    for path, text in sites.items():
        if not balanced(text):
            problems.append(f'unbalanced after rewrite: {path}')
    doors = door_texts(plan)
    for path, (text, _mount) in doors.items():
        ok, pretty = parses(text)
        if not ok:
            problems.append(f'door file does not parse: {path}')
        formatted[path] = pretty
    if dry_run or problems:
        return problems
    for path, text in sites.items():
        path.write_text(text, encoding='utf-8', newline='\n')
    mounts = defaultdict(list)
    for path, (_text, mount) in doors.items():
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(formatted[path], encoding='utf-8', newline='\n')
        if mount is not None:
            mounts[mount[0]].append(mount[1])
    for source, changes in mounts.items():
        current = Path(source).read_text(encoding='utf-8')
        Path(source).write_text(replace_spans(current, sorted(changes)), encoding='utf-8', newline='\n')
    open_declarations(plan.opened)
    return problems


def open_declarations(opened):
    """Make `*_for_test` methods (already sealed doors by name) `pub` at their declaration."""
    by_file = defaultdict(set)
    for path, at in opened:
        by_file[path].add(at)
    for path, starts in by_file.items():
        text = path.read_text(encoding='utf-8')
        for at in sorted(starts, reverse=True):
            head = re.match(r'pub(?:\s*\([^)]*\))?\s+', text[at:])
            if head:
                text = text[:at] + 'pub ' + text[at + head.end():]
            elif re.match(r'(?:const |async |unsafe )*fn ', text[at:]):
                text = text[:at] + 'pub ' + text[at:]
        path.write_text(text, encoding='utf-8', newline='\n')
