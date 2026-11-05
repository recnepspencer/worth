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


IMPL_BLOCK = re.compile(r'(?ms)^impl\b(.*?)\{\n(.*?)^\}\n?')


def header_key(header):
    """An impl header as rustfmt-insensitive tokens: no whitespace, no trailing commas, no
    generic defaults."""
    from src_door_members_text import without_defaults
    header = header.strip()
    generics = re.match(r'(<.*?>)\s', header + ' ', re.S) if header.startswith('<') else None
    if generics:
        header = without_defaults(generics.group(1)) + header[generics.end(1):]
    values = [token.value for token in tokens(header)]
    kept = [v for k, v in enumerate(values) if not (v == ',' and k + 1 < len(values) and values[k + 1] in ('>', ')', ']'))]
    return ' '.join(kept)


_DEFINITIONS = {}
DEFINES = re.compile(r'\b(?:struct|enum|union|type)\s+([A-Z]\w*)')


def aliased(door_path, name):
    """The type `name` stands for in the module a door file extends: a `use ... Real as name`
    there names Real, so two modules that alias different types alike stay apart."""
    directory = Path(door_path).parent
    for module in (directory.with_suffix('.rs'), directory / 'mod.rs'):
        if module.exists():
            found = re.search(rf'\b(\w+)\s+as\s+{name}\b', module.read_text(encoding='utf-8'))
            if found:
                return found.group(1)
    return name


def type_owner(library, header, door_path):
    """The file defining the impl's self type, so two types spelled alike stay apart. A name
    defined more than once resolves to the definition nearest the door file (longest shared
    path); an unknown name to the door's own directory."""
    library = Path(library)
    if library not in _DEFINITIONS:
        table = defaultdict(list)
        for path in sorted(library.rglob('*.rs')):
            if not path.name.endswith('court_doors_for_test.rs') and 'court_doors_for_test' not in path.parts:
                for name in set(DEFINES.findall(path.read_text(encoding='utf-8'))):
                    table[name].append(path)
        _DEFINITIONS[library] = table
    rest = re.sub(r'^\s*<.*?>\s', ' ', header + ' ', count=1, flags=re.S) if header.lstrip().startswith('<') else header
    self_type = re.match(r'\s*&?\s*((?:[\w]+::)*)(\w+)', rest)
    if self_type is None:
        return str(Path(door_path).parent)
    candidates = _DEFINITIONS[library].get(aliased(door_path, self_type.group(2)), [])
    if not candidates:
        return str(Path(door_path).parent)
    door = Path(door_path).parts
    return str(max(candidates, key=lambda c: (len([1 for a, b in zip(c.parts, door) if a == b]), -len(c.parts))))


def door_keys(text, door_path=None, library=None):
    """{(self type owner, exact impl header, door name)} of the `impl` blocks in door text. The
    owner is the file defining the self type when `library` is given."""
    keys = set()
    for block in IMPL_BLOCK.finditer(text):
        owner = type_owner(library, block.group(1), door_path) if library is not None else None
        keys |= {(owner, header_key(block.group(1)), name)
                 for name in re.findall(r'\bpub fn (\w+_for_test)\b', block.group(2))}
    return keys


def door_files(library):
    return sorted(set(Path(library).rglob('court_doors_for_test.rs')) | set(Path(library).rglob('*_court_doors_for_test.rs')))


def normalized_doors(library):
    """{door file: text} with defaults stripped from impl headers and a door that repeats an
    (self type's defining file, exact header, name) seen earlier (in path order) removed: an
    inherent method name is one namespace per type across the crate."""
    from src_door_members_text import without_defaults
    seen, result = set(), {}
    for path in door_files(library):
        text = path.read_text(encoding='utf-8')

        def keep(block):
            header = block.group(1)
            names = set(re.findall(r'\bpub fn (\w+_for_test)\b', block.group(2)))
            owner = type_owner(library, header, path)
            keys = {(owner, header_key(header), n) for n in names}
            if keys and keys <= seen:
                return ''
            seen.update(keys)
            stripped = re.sub(r'^(\s*)(<.*?>)(?=\s)', lambda m: m.group(1) + without_defaults(m.group(2)),
                              header, count=1, flags=re.S)
            return 'impl' + stripped + '{\n' + block.group(2) + '}\n'
        new = IMPL_BLOCK.sub(keep, text)
        new = re.sub(r'\n{3,}', '\n\n', new)
        if new != text:
            result[path] = new
    return result


def door_texts(plan):
    """{door file path: (new full text, mount edit or None)} for every module given doors."""
    result = {}
    # One inherent namespace per type across the crate: keys already in any door file count.
    present, library = set(), None
    if plan.doors:
        library = door_file_path(plan.index[()], plan.index).parent
        for path in door_files(library):
            present |= door_keys(path.read_text(encoding='utf-8'), path, library)
    for names, entries in sorted(plan.doors.items()):
        node = plan.index[names]
        path = door_file_path(node, plan.index)
        existing = path.read_text(encoding='utf-8') if path.exists() else HEADER
        added = []
        for _key, text in sorted(entries.items()):
            keys = door_keys(text, path, library)
            if text not in existing and not keys <= present:
                added.append(text)
                present |= keys
        if added:
            result[path] = (existing.rstrip('\n') + '\n\n' + '\n\n'.join(added) + '\n', mount_edit(node, plan.index))
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
