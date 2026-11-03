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
from src_families import families, function_bodies
from src_paths import Reference, module_index, references
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


def moving_families(rows):
    by_module = {r['module']: r for r in rows}
    moving = {m for m, r in by_module.items() if r['decision'] in MOVING}
    changed = True
    while changed:
        changed = False
        for module, row in sorted(by_module.items()):
            if module in moving:
                continue
            reads = any(s in moving for s in row['support_from'])
            read = any(module in by_module[m]['support_from'] for m in moving)
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
    def __init__(self, lib_source, repository, baseline, admissions):
        self.repository = Path(repository).resolve()
        self.lib = Path(lib_source).resolve()
        self.plan = audit(self.lib, self.repository, baseline, admissions)
        self.tree, found = families(self.lib)
        self.index = module_index(self.tree)
        self.families = {f.names: f for f in found}
        self.moving = [tuple(m.split('::')) for m in moving_families(self.plan['families'])]
        self.production = Surface(self.tree, self.index, list(self.families))
        self.everything = Surface(self.tree, self.index, [])
        self.records = {}
        for family in self.moving:
            for node in self.families[family].node.walk():
                self.records[node.names] = self.record(node, family)

    def record(self, node, family):
        text = node.own_text()
        record = ModuleRecord(node.names, family, node, items_of(text))
        record.tests = [i.name for i in record.items if i.test]
        record.uses = self.resolved(node)
        return record

    def owner_family(self, names):
        return next((f for f in self.moving if names[:len(f)] == f), None)

    def staying_family(self, names):
        return next((f for f in self.families if names[:len(f)] == f and f not in self.moving), None)

    def resolved(self, node):
        """(Reference, origin) pairs; module imports expand to the `alias::name` uses in the text."""
        stream = tokens(node.own_text())
        result = []
        for reference in references(node, self.index):
            for target in self.expand(reference, node, stream):
                if target.item is None:
                    result.append((target, (target.module, None)))
                    continue
                origin, _gated = self.everything.origin(target.module, target.item)
                # Macro-generated names resolve no declaration; the named module still owns them.
                result.append((target, origin or (target.module, target.item)))
        return result

    def expand(self, reference, node, stream):
        if reference.glob:
            scope = self.everything.scopes.get(reference.module)
            words = {t.value for t in stream if t.kind == 'ident'}
            return [Reference(reference.module, n) for n in sorted(self.visible(reference.module) & words)] \
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
