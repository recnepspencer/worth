"""Item doors: the sealed `court_doors_for_test` module of worth-query-execution.

Moved courts and testkits name library items the library does not publish. Each such item is
re-exported once, explicitly, from `court_doors_for_test::<the path the court used>::Item`;
the item itself becomes `pub` and every module on its defining path at least `pub(crate)`,
so the door is the only external path. Items the facade already publishes keep their facade
path. Module references (`use worth_query_execution::a::m;` then `m::X`) re-export the items
used through them; globs expand to the names the file uses.
"""
from collections import defaultdict
from dataclasses import dataclass, field
import re

from rust_source import tokens
from src_emit_production import absolute, own_offset, visibility_span
from src_scope import scope_items

LIBRARY = 'worth_query_execution'
DOOR = 'court_doors_for_test'
ITEM_KEYWORDS = ('fn', 'struct', 'enum', 'const', 'static', 'type', 'trait', 'union')


@dataclass
class Reference:
    path: object
    start: int
    end: int
    segments: tuple
    glob: bool = False
    through_door: bool = False


@dataclass
class ItemDoors:
    # door path (module segments the court used) -> {name: defining (module, name)}
    doors: dict = field(default_factory=lambda: defaultdict(dict))
    rewrites: dict = field(default_factory=lambda: defaultdict(list))
    unresolved: list = field(default_factory=list)


def library_paths(path, text):
    """Maximal `worth_query_execution::a::b...` paths outside the facade and the door."""
    stream, result, i = tokens(text), [], 0
    while i < len(stream):
        token = stream[i]
        rooted = token.value == LIBRARY and not (i >= 2 and stream[i - 1].value == ':' and stream[i - 2].value == ':')
        if not rooted:
            i += 1
            continue
        segments, j = [], i + 1
        while j + 2 < len(stream) and stream[j].value == ':' and stream[j + 1].value == ':' \
                and stream[j + 2].kind == 'ident':
            segments.append(stream[j + 2].value)
            j += 3
        glob = j + 2 < len(stream) and stream[j].value == ':' and stream[j + 1].value == ':' \
            and stream[j + 2].value == '*'
        through_door = bool(segments) and segments[0] == DOOR
        if through_door:
            # Already a door path: plan it again (so a rerun keeps the door) but do not rewrite.
            segments = segments[1:]
        if segments and segments[0] != 'facade':
            end = stream[j - 1].end
            result.append(Reference(path, token.start, end, tuple(segments), glob, through_door))
        i = j
    return result


def split_module(index, segments):
    module, k = (), 0
    while k < len(segments) and module + (segments[k],) in index:
        module, k = module + (segments[k],), k + 1
    return module, segments[k:]


def public_paths(surface):
    """Defining (module, name) -> shortest public path the facade already offers."""
    best = {}
    for module in sorted(surface.public_modules):
        if module[:1] == (DOOR,):
            continue  # the door itself is not a facade
        for name, origin in sorted(surface.exports(module).items()):
            path = module + (name,)
            if origin not in best or len(path) < len(best[origin]):
                best[origin] = path
    return best


def plan_items(files, index, surface):
    """ItemDoors for every library path the given court/testkit files name."""
    published = public_paths(surface)
    result = ItemDoors()
    for path in files:
        text = path.read_text(encoding='utf-8')
        for reference in library_paths(path, text):
            module, rest = split_module(index, reference.segments)
            if not rest:
                aliases = module_alias_uses(text + mounted_foreign_text(path, text), reference, module, index)
                if reference.glob:
                    aliases = glob_uses(text, module, surface)
                for name in aliases:
                    add_door(result, surface, published, module, name)
                if reference.glob:
                    result.rewrites[path].append((reference, module, None, sorted(aliases)))
                elif not aliases and is_import(text, reference) and 'worth-query-certification/' not in text:
                    # (A module mounting foreign fixture source keeps its imports: that source reads them.)
                    # An import of a library module nothing names through: an over-broad glob
                    # expansion. It has no door; the import goes.
                    result.rewrites[path].append((reference, module, None, ('drop',)))
                else:
                    result.rewrites[path].append((reference, module, None, None))
                continue
            name = rest[0]
            target = add_door(result, surface, published, module, name)
            result.rewrites[path].append((reference, module, name, target))
    return result


def add_door(result, surface, published, module, name):
    origin, _gated = surface.origin(module, name)
    if origin is not None and origin[0] == '<external>':
        # A library module's import of another crate's item: name the item where it lives.
        return ('external', origin[1])
    if origin is None:
        # Declared by an item macro the scan cannot expand: re-export the name where it is.
        result.unresolved.append((module, name))
        origin = (module, name)
    if origin in published:
        return ('public', published[origin])
    result.doors[module][name] = origin
    return ('door', module + (name,))


def mounted_foreign_text(path, text):
    """Text of the certification fixture files this file mounts: they read its imports."""
    result = []
    for literal in re.findall(r'#\[path = "([^"]*worth-query-certification/[^"]*)"\]', text):
        target = (path.parent / literal).resolve()
        if target.is_file():
            result.append(target.read_text(encoding='utf-8'))
            # Mounted directories (`mod.rs` siblings) are read too.
            if target.name == 'mod.rs':
                result += [f.read_text(encoding='utf-8') for f in sorted(target.parent.rglob('*.rs'))]
    for literal in re.findall(r'include!\(\s*"([^"]*worth-query-certification/[^"]*)"', text):
        target = (path.parent / literal).resolve()
        if target.is_file():
            result.append(target.read_text(encoding='utf-8'))
    return '\n'.join(result)


def is_import(text, reference):
    line_start = text.rfind('\n', 0, reference.start) + 1
    return re.match(r'\s*(?:pub(?:\([^)]*\))?\s+)?use\s+$', text[line_start:reference.start]) is not None


def module_alias_uses(text, reference, module, index):
    """Items named through an imported library module: `m::X` after `use ...::m [as a];`."""
    after = text[reference.end:reference.end + 80]
    alias = re.match(r'\s+as\s+(\w+)', after)
    head = alias.group(1) if alias else module[-1]
    names = set()
    stream = tokens(text)
    for i in range(len(stream) - 3):
        if stream[i].value == head and stream[i + 1].value == ':' and stream[i + 2].value == ':' \
                and not (i >= 2 and stream[i - 1].value == ':'):
            segments, j = [], i + 3
            while j < len(stream) and stream[j].kind == 'ident':
                segments.append(stream[j].value)
                if j + 2 < len(stream) and stream[j + 1].value == ':' and stream[j + 2].value == ':':
                    j += 3
                else:
                    break
            sub, rest = split_module(index, module + tuple(segments))
            if rest and sub == module:
                names.add(rest[0])
    return names


def glob_uses(text, module, surface):
    scope = surface.scopes.get(module)
    if scope is None:
        return set()
    words = {t.value for t in tokens(text) if t.kind == 'ident'}
    return {name for name in scope.names if name in words}


def door_files(doors, root_dir):
    """{path: text} of the door module tree (one file per door module, explicit re-exports)."""
    tree = defaultdict(set)
    for module in doors:
        for depth in range(len(module)):
            tree[module[:depth]].add(module[depth])
    files = {}
    for module in sorted(set(tree) | set(doors)):
        lines = ['//! Sealed test doors: the only external path to these library items.' if not module
                 else f'//! Doors under `{"::".join(module)}`.']
        for child in sorted(tree.get(module, ())):
            lines.append(f'#[path = "{child}/mod.rs"]\npub mod {child};')
        for name, (defining, item) in sorted(doors.get(module, {}).items()):
            target = '::'.join(('crate',) + defining + (item,))
            lines.append(f'pub use {target}{"" if item == name else " as " + name};')
        rel = '/'.join(module + ('mod.rs',))
        files[f'{root_dir}/{rel}'] = '\n'.join(lines) + '\n'
    return files


def visibility_edits(doors, index):
    """{source: [(start, end, replacement)]}: door items `pub`, their modules `pub(crate)`."""
    edits, seen = defaultdict(set), set()
    for _module, entries in sorted(doors.items()):
        for _name, (defining, item) in sorted(entries.items()):
            if (defining, item) in seen:
                continue
            seen.add((defining, item))
            node = index[defining]
            own = node.own_text()
            for _a, _v, keyword, name, start, end in scope_items(own):
                if name == item and keyword in ITEM_KEYWORDS:
                    s, e = own_offset(node, start), own_offset(node, start) + (end - start)
                    vs, ve = visibility_span(node.text, s, e)
                    if node.text[vs:ve] != 'pub':
                        a, b = absolute(node, vs, ve)
                        edits[node.source].add((a, b, 'pub' if ve > vs else 'pub '))
            for depth in range(2, len(defining) + 1):
                child = index[defining[:depth]]
                parent = index[defining[:depth - 1]]
                vs, ve = visibility_span(parent.text, child.declaration.start, child.declaration.end)
                current = parent.text[vs:ve]
                if current in ('', 'pub(super)', 'pub(self)') or current.startswith('pub(in'):
                    a, b = absolute(parent, vs, ve)
                    edits[parent.source].add((a, b, 'pub(crate)' if ve > vs else 'pub(crate) '))
    return {source: sorted(changes) for source, changes in edits.items()}
