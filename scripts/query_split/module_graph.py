"""Production reference graph over chosen module nodes of one library, with lines per node.

A node is a module path; every other module belongs to its longest node prefix.
References are distinct (source file, defining item) pairs, with re-exports resolved
to the defining module and test-gated families and items excluded on both ends.
"""
from collections import defaultdict
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parent.parent / 'test_topology'))

from rust_source import tokens  # noqa: E402
from src_families import families, test_gated  # noqa: E402
from src_paths import Reference, module_index, references  # noqa: E402
from src_scope import scope_items  # noqa: E402
from src_surface import Surface  # noqa: E402


class ProductionView:
    """A module node whose own text omits test-gated items, so unit-test imports never couple."""

    def __init__(self, node):
        self.names, self.source = node.names, node.source
        text = node.own_text()
        kept, position = [], 0
        for attributes, _visibility, _keyword, _name, start, end in scope_items(text):
            if test_gated(attributes):
                kept.append(text[position:start])
                position = end
        kept.append(text[position:])
        self._text = ''.join(kept)

    def own_text(self):
        return self._text


class LibraryModel:
    """The parsed library: module index, public surface and test families, read once."""

    def __init__(self, lib_source, repository):
        self.repository = Path(repository).resolve()
        self.tree, self.families = families(Path(lib_source))
        self.index = module_index(self.tree)
        self.surface = Surface(self.tree, self.index, [f.names for f in self.families])
        self.views = {names: ProductionView(node) for names, node in self.index.items()
                      if not self.surface.in_family(names)}
        self._references = {}

    def relative(self, path):
        return Path(path).resolve().relative_to(self.repository).as_posix()

    def resolved_references(self, names):
        """(source file, defining module, item or None, kind) for one production module."""
        if names in self._references:
            return self._references[names]
        view, result = self.views[names], set()
        words = {t.value for t in tokens(view.own_text()) if t.kind == 'ident'}
        for reference in references(view, self.index):
            for target in self.expand(reference, words):
                found = self.surface.origin(target.module, target.item)[0] if target.item \
                    else (target.module, None)
                if found is None or found[0] == '<external>' or self.surface.in_family(found[0]):
                    continue
                module, item = found
                kind = self.surface.scopes[module].kinds.get(item, 'module') if item else 'module'
                result.add((self.relative(view.source), module, item, kind))
        self._references[names] = sorted(result, key=lambda r: (r[0], r[1], r[2] or '', r[3]))
        return self._references[names]

    def expand(self, reference, words):
        if not reference.glob:
            return [reference]
        scope = self.surface.scopes.get(reference.module)
        if scope is None:
            return []
        return [Reference(reference.module, name) for name in sorted(scope.names & words)]

    def production_lines(self, names):
        """Lines of a module's own production text (children counted at their own node)."""
        view = self.views.get(names)
        return 0 if view is None else view.own_text().count('\n')


def owner(names, nodes):
    best = None
    for node in nodes:
        if names[:len(node)] == node and (best is None or len(node) > len(best)):
            best = node
    return best


def node_graph(model, nodes):
    """(edge -> {(file, item)}, item kinds, node -> production lines) over the chosen nodes."""
    nodes = sorted(set(nodes))
    edges, kinds, lines = defaultdict(set), {}, defaultdict(int)
    for names in sorted(model.views):
        source_node = owner(names, nodes)
        if source_node is None:
            continue
        lines[source_node] += model.production_lines(names)
        for source, module, item, kind in model.resolved_references(names):
            target_node = owner(module, nodes)
            if target_node is None or target_node == source_node:
                continue
            key = '::'.join(module) + ('::' + item if item else '')
            edges[(source_node, target_node)].add((source, key))
            kinds[key] = kind
    return edges, kinds, dict(lines)


def flattened_nodes(model, expanded):
    """Every child of each expanded module, plus each expanded module's own root as a node."""
    result = {()}
    for parent in expanded:
        result.add(parent)
        for child in model.index[parent].children:
            if not model.surface.in_family(child.names):
                result.add(child.names)
    for names in list(model.views):
        if len(names) == 1:
            result.add(names)
    return sorted(result)


def label(names):
    return '::'.join(names) if names else '<crate root>'
