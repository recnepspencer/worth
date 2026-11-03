"""The in-source test families that leave a library, with their items and resolved references.

A family moves when the audit marks it court-shaped (it constructs a runtime root, or needs
only the public surface), and the move set is closed both ways over support edges: a family
that reads a moving family, or that a moving family reads, moves too. What remains is
genuine private-logic unit testing with no shared support.
"""
from dataclasses import dataclass, field
from pathlib import Path

from rust_source import tokens
from src_audit import audit
from src_families import ExpandedView, families, file_text, function_bodies, included_files
from src_paths import Reference, module_index, references, resolve
from src_scope import scope_items
from src_surface import Surface

MOVING = ('runtime-root-private', 'runtime-root-public', 'public-surface')


@dataclass
class Item:
    """One top-level item of a module scope; `test` marks a test function."""
    keyword: str
    name: str
    start: int
    end: int
    test: bool
    attributes: str
    visibility: str


@dataclass
class ModuleRecord:
    names: tuple
    family: tuple
    node: object
    items: list = field(default_factory=list)
    tests: list = field(default_factory=list)
    uses: list = field(default_factory=list)
    extracted: set = field(default_factory=set)
    extracted_spans: list = field(default_factory=list)
    includes: list = field(default_factory=list)
    kept_impls: list = field(default_factory=list)
    kept: set = field(default_factory=set)
    kept_names: set = field(default_factory=set)
    doors: list = field(default_factory=list)
    foreign: bool = False


def moving_families(rows, forced=(), excluded=()):
    """excluded: support modules production test code anchors; they stay and compile in place."""
    by_module = {r['module']: r for r in rows}
    excluded = set(excluded)
    moving = ({m for m, r in by_module.items() if r['decision'] in MOVING} | set(forced)) - excluded
    changed = True
    while changed:
        changed = False
        for module, row in sorted(by_module.items()):
            if module in moving or module in excluded:
                continue
            reads = any(s in moving for s in row['support_from'])
            read = any(module in by_module[m]['support_from'] for m in moving if m in by_module)
            if reads or read:
                moving.add(module)
                changed = True
    return sorted(moving)


def items_of(text):
    names = {name for name, _digest in function_bodies(text)}
    result = []
    for attributes, visibility, keyword, name, start, end in scope_items(text):
        result.append(Item(keyword, name, start, end, keyword == 'fn' and name in names,
                           attributes, visibility))
    return result


class MoveModel:
    def __init__(self, lib_source, repository, baseline, admissions, forced=(), excluded=()):
        self.repository = Path(repository).resolve()
        self.lib = Path(lib_source).resolve()
        self.plan = audit(self.lib, self.repository, baseline, admissions)
        self.tree, found = families(self.lib)
        self.index = module_index(self.tree)
        self.families = {f.names: f for f in found}
        self.moving = [tuple(m.split('::')) for m in moving_families(self.plan['families'], forced, excluded)]
        self.production = Surface(self.tree, self.index, list(self.families))
        self.everything = Surface(self.tree, self.index, [])
        self.records = {}
        for family in self.moving:
            for node in self.families[family].node.walk():
                self.records[node.names] = self.record(node, family)

    def record(self, node, family):
        text = node.own_text()
        record = ModuleRecord(node.names, family, node, items_of(text))
        record.includes = included_files(node)
        record.tests = [i.name for i in record.items if i.test] + [
            name for _literal, target in record.includes for name, _digest in function_bodies(file_text(target))]
        record.uses = self.resolved(ExpandedView(node))
        return record

    def owner_family(self, names):
        return next((f for f in self.moving if names[:len(f)] == f), None)

    def staying_family(self, names):
        return next((f for f in self.families if names[:len(f)] == f and f not in self.moving), None)

    def resolved(self, node):
        """(Reference, origin) pairs; module imports expand to the `alias::name` uses in the text."""
        stream = tokens(node.own_text())
        result = []
        for reference in references(node, self.index) + self.child_paths(node):
            for target in self.expand(reference, node, stream):
                if target.item is None:
                    result.append((target, (target.module, None)))
                    continue
                origin, _gated = self.everything.origin(target.module, target.item)
                # Macro-generated names resolve no declaration; the named module still owns them.
                result.append((target, origin or (target.module, target.item)))
        return result

    def alias_module(self, names, head):
        """The module an imported name (`use a::b as graph;`, or via a glob) denotes, if any."""
        if names not in self.index:
            return None
        origin, _gated = self.everything.origin(names, head)
        if origin is not None and origin[0] != '<external>' and origin[1] is None:
            return origin[0]
        return None

    def heads(self, names):
        """Unrooted path heads that denote modules: child modules and imported module aliases."""
        if names not in self.index:
            return {}
        result = {c.names[-1]: c.names for c in self.index[names].children}
        for name in self.visible(names):
            if name not in result:
                module = self.alias_module(names, name)
                if module is not None:
                    result[name] = module
        return result

    def child_paths(self, node):
        """Unrooted paths through child modules or imported module aliases (`graph::x::Item`)."""
        from src_occurrences import scan
        heads = self.heads(node.names)
        if not heads:
            return []
        result = []
        for occurrence in scan(node.own_text(), set(heads))[1]:
            head = occurrence.segments[0]
            if head in heads:
                resolved = resolve(('crate',) + heads[head] + occurrence.segments[1:], node.names, self.index)
                if resolved is not None:
                    module, rest = resolved
                    result.append(Reference(module, rest[0] if rest else None))
        return result

    def expand(self, reference, node, stream):
        if reference.glob:
            scope = self.everything.scopes.get(reference.module)
            words = {t.value for t in stream if t.kind == 'ident'}
            # The glob itself reads its module (the rewrite keeps it a glob), plus each name it binds.
            return [Reference(reference.module, None)] + [Reference(reference.module, n) for n in
                                                          sorted(self.visible(reference.module) & words)] \
                if scope is not None else []
        if reference.item is None or (reference.module + (reference.item,)) in self.index:
            module = reference.module if reference.item is None else reference.module + (reference.item,)
            alias = reference.alias or module[-1]
            used = {stream[i + 3].value for i in range(len(stream) - 3)
                    if stream[i].value == alias and stream[i + 1].value == ':' and stream[i + 2].value == ':'
                    and stream[i + 3].kind == 'ident' and (i == 0 or stream[i - 1].value not in (':', '.'))}
            return [Reference(module, None, alias=alias)] + [Reference(module, n) for n in sorted(used)]
        return [reference]

    def visible(self, module, seen=frozenset()):
        """Names a glob import of `module` brings into scope, following its own glob imports."""
        scope = self.everything.scopes.get(module)
        if scope is None or module in seen:
            return set()
        names = set(scope.names) | {c.names[-1] for c in self.index[module].children}
        for segments in scope.globs:
            target = self.everything.target(segments, module)
            if target is not None and not target[1]:
                names |= self.visible(target[0], seen | {module})
        return names


class ExtractedNode:
    """Test-dependent `cfg(test)` items of a production module, moved as one support scope."""

    def __init__(self, module_node, text):
        self.names, self.source, self.offset = module_node.names, module_node.source, None
        self.declaration, self.children, self.text = None, [], text

    def own_text(self):
        return self.text

    def walk(self):
        yield self


def add_extracted(model, module, items):
    """Record extracted items of production `module` (name, own-text spans) as a support scope."""
    node = model.index[module]
    own = node.own_text()
    spans = sorted({span for _name, item_spans in items for span in item_spans})
    text = '\n\n'.join(own[s:e] for s, e in spans) + '\n'
    record = ModuleRecord(module, ('<extracted>',) + module, ExtractedNode(node, text), items_of(text))
    record.uses = model.resolved(record.node)
    record.extracted = {name for name, _spans in items}
    record.extracted_spans = spans
    model.records[module] = record
    return record
