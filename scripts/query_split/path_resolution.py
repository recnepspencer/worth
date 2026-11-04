"""Where a path written in one module of the library leads, through every way Rust names a module.

A path's head may be `crate`/`super`/`self`, a child module, an import of a module, a module a
glob brings into scope (a glob also imports its target's child modules), or a module a `use`
inside a function body binds. A path may also walk on through a module re-export
(`parent::m::Item` through `pub use child::m`). `Paths.resolve` follows all of them to
`(module, remaining segments)`.
"""
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parent.parent / 'test_topology'))

from rust_source import tokens  # noqa: E402
from src_occurrences import scan  # noqa: E402
from src_paths import resolve  # noqa: E402
from src_surface import declarations as scope_declarations  # noqa: E402


class Paths:
    def __init__(self, model):
        self.model = model
        self._block_modules = {}

    def node_scope(self, node):
        scope = self.model.surface.scopes.get(node.names)
        return scope if scope is not None else scope_declarations(node.own_text())

    def head_module(self, node, head):
        """The module a non-rooted path head names in `node`, or None."""
        index = self.model.index
        if node.names in self.model.surface.scopes:
            found = self.model.surface.origin(node.names, head)[0]
            if found is not None and found[0] != '<external>':
                return found[0] if found[1] is None else None
        scope = self.node_scope(node)
        if head in scope.imports:
            resolved = resolve(scope.imports[head], node.names, index)
            return resolved[0] if resolved is not None and not resolved[1] else None
        if head in scope.declared:
            return None
        for segments in scope.globs:
            resolved = resolve(segments, node.names, index)
            if resolved is not None and not resolved[1] and resolved[0] + (head,) in index:
                return resolved[0] + (head,)
        return self.block_modules(node).get(head)

    def block_modules(self, node):
        """{name: module} a `use` anywhere in the node's text (function bodies included) binds."""
        if node.names not in self._block_modules:
            found = {}
            uses, _paths = scan(node.text, ())
            for use in uses:
                for segments, glob, alias in use.leaves:
                    if not segments or glob:
                        continue
                    resolved = resolve(segments, node.names, self.model.index)
                    if resolved is not None and not resolved[1]:
                        found[alias or segments[-1]] = resolved[0]
            self._block_modules[node.names] = found
        return self._block_modules[node.names]

    def follow(self, module, rest):
        """Walk on through module re-exports."""
        while len(rest) > 1 and module in self.model.surface.scopes:
            found = self.model.surface.origin(module, rest[0])[0]
            if found is None or found[0] == '<external>' or found[1] is not None:
                break
            module, rest = found[0], rest[1:]
            while len(rest) > 1 and module + (rest[0],) in self.model.index:
                module, rest = module + (rest[0],), rest[1:]
        return module, rest

    def module_heads(self, node):
        """Every name heading a path in the node's text that names a module there."""
        heads = {c.names[-1] for c in node.children}
        stream = tokens(node.text)
        candidates = {t.value for k, t in enumerate(stream[:-2]) if t.kind == 'ident'
                      and stream[k + 1].value == ':' and stream[k + 2].value == ':'}
        for name in candidates - heads - {'crate', 'super', 'self'}:
            if self.head_module(node, name) is not None:
                heads.add(name)
        return heads

    def resolve(self, node, segments, through_reexports=True):
        """(module, rest, rooted) for a path written in `node`; rooted means the path names its
        module chain itself (`crate`/`super`/`self`/child) rather than through an import."""
        index = self.model.index
        head = segments[0]
        if head in ('crate', 'super', 'self') or node.names + (head,) in index:
            resolved = resolve(segments, node.names, index)
            rooted = True
        else:
            base = self.head_module(node, head)
            if base is None:
                return None
            resolved = resolve(('self',) + tuple(segments[1:]), base, index)
            rooted = False
        if resolved is None:
            return None
        module, rest = self.follow(*resolved) if through_reexports and resolved[1] else resolved
        return module, rest, rooted
