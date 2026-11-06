"""A library's public reach and its `cfg(test)` production hooks, read from source."""
from dataclasses import dataclass, field

from rust_source import pairs, tokens
from src_families import test_gated
from src_paths import resolve, use_trees
from src_scope import scope_items

ITEM_KEYWORDS = ('fn', 'struct', 'enum', 'const', 'static', 'type', 'trait', 'union')


@dataclass
class Declarations:
    """What one module scope declares and imports, split by bare `pub` and test gating."""
    declared: set = field(default_factory=set)
    public: set = field(default_factory=set)
    gated: set = field(default_factory=set)
    imports: dict = field(default_factory=dict)
    public_imports: set = field(default_factory=set)
    globs: list = field(default_factory=list)
    public_globs: list = field(default_factory=list)
    members: set = field(default_factory=set)
    gated_members: set = field(default_factory=set)
    kinds: dict = field(default_factory=dict)

    @property
    def names(self):
        return self.declared | set(self.imports)


def declarations(text):
    result = Declarations()
    for attributes, visibility, keyword, name, start, end in scope_items(text):
        gated = test_gated(attributes)
        item_text = text[start:end]
        if keyword == 'use':
            for segments, glob, alias in use_trees(item_text):
                if not segments:
                    continue
                if glob:
                    result.globs.append(segments)
                    if visibility == 'pub' and not gated:
                        result.public_globs.append(segments)
                    continue
                leaf = alias or segments[-1]
                result.imports[leaf] = segments
                if visibility == 'pub':
                    result.public_imports.add(leaf)
                if gated:
                    result.gated.add(leaf)
            continue
        if keyword in ('impl', 'trait'):
            every, gated_only = methods(item_text, gated)
            result.members |= every
            result.gated_members |= gated_only
        if keyword in ITEM_KEYWORDS or keyword == 'macro_rules':
            declared = name
        elif keyword not in ('mod', 'impl', 'extern') and '!' in item_text[:80]:
            declared = macro_static(item_text)
        else:
            declared = None
        if declared:
            result.declared.add(declared)
            result.kinds[declared] = keyword if keyword in ITEM_KEYWORDS + ('macro_rules',) else 'static'
            if visibility == 'pub':
                result.public.add(declared)
            if gated:
                result.gated.add(declared)
    return result


def methods(text, whole):
    stream = tokens(text)
    opening = next((i for i, t in enumerate(stream) if t.value == '{'), None)
    if opening is None:
        return set(), set()
    body = text[stream[opening].end:stream[pairs(stream)[opening]].start]
    every, gated = set(), set()
    for attributes, _visibility, keyword, name, _start, _end in scope_items(body):
        if keyword == 'fn':
            (gated if whole or test_gated(attributes) else every).add(name)
    return every, gated


def macro_static(text):
    stream = tokens(text)
    for i, token in enumerate(stream[:-1]):
        if token.value == 'static' and stream[i + 1].kind == 'ident':
            return stream[i + 1].value
    return None


class Surface:
    """Every name resolved to its defining item, and the definitions public modules export."""

    def __init__(self, tree, index, family_roots):
        self.index = index
        self.family_roots = family_roots
        self.scopes = {names: declarations(node.own_text()) for names, node in index.items()
                       if not self.in_family(names)}
        self.public_modules = reachable_modules(tree)
        ungated = set().union(*(s.members for s in self.scopes.values()))
        self.gated_members = set().union(*(s.gated_members for s in self.scopes.values())) - ungated
        self._origins, self._exports = {}, {}
        self.public_origins = set()
        for module in sorted(self.public_modules):
            self.public_origins |= set(self.exports(module).values())

    def in_family(self, names):
        return any(names[:len(root)] == root for root in self.family_roots)

    def target(self, segments, module):
        resolved = resolve(segments, module, self.index)
        if resolved is None or self.in_family(resolved[0]):
            return None
        return resolved

    def origin(self, module, item, seen=frozenset()):
        """(defining (module, name) or None, whether any hop is test-gated) for `item` in `module`.

        An external re-export resolves to ('<external>', path); a name no scope declares
        (macro-generated, or a variant import) resolves to None and stays unresolved.
        """
        key = (module, item)
        if key in self._origins:
            return self._origins[key]
        scope = self.scopes.get(module)
        result = (None, False)
        if scope is not None and key not in seen:
            seen = seen | {key}
            gated = item in scope.gated
            if item in scope.declared:
                result = (key, gated)
            elif item in scope.imports:
                resolved = resolve(scope.imports[item], module, self.index)
                if resolved is None:
                    result = (('<external>', '::'.join(scope.imports[item])), gated)
                elif not self.in_family(resolved[0]):
                    target, rest = resolved
                    found, inner = self.origin(target, rest[0], seen) if rest else ((target, None), False)
                    result = (found, gated or inner)
            else:
                for segments in scope.globs:
                    resolved = self.target(segments, module)
                    if resolved is None or resolved[1]:
                        continue
                    if resolved[0] + (item,) in self.index:
                        # A glob also brings the target's child modules into scope.
                        result = ((resolved[0] + (item,), None), False)
                        break
                    found, inner = self.origin(resolved[0], item, seen)
                    if found is not None:
                        result = (found, inner)
                        break
        self._origins[key] = result
        return result

    def exports(self, module, seen=frozenset()):
        """Names a module exports with bare `pub`, mapped to their defining items."""
        if module in self._exports:
            return self._exports[module]
        scope = self.scopes.get(module)
        result = {}
        if scope is not None and module not in seen:
            seen = seen | {module}
            for name in scope.public - scope.gated:
                result[name] = (module, name)
            for name in scope.public_imports - scope.gated:
                found, gated = self.origin(module, name)
                if found is not None and not gated:
                    result[name] = found
            for segments in scope.public_globs:
                resolved = self.target(segments, module)
                if resolved is not None and not resolved[1]:
                    for name, found in self.exports(resolved[0], seen).items():
                        result.setdefault(name, found)
        self._exports[module] = result
        return result

    def classify(self, module, item):
        """'public', 'hook', 'private' or 'unresolved' for one production reference."""
        if item is None:
            return 'public' if module in self.public_modules else 'private'
        found, gated = self.origin(module, item)
        if gated:
            return 'hook'
        if found is None:
            return 'unresolved'
        declaring, name = found
        if declaring == '<external>' or found in self.public_origins:
            return 'public'
        if name is None:
            return 'public' if declaring in self.public_modules else 'private'
        return 'private'


def reachable_modules(tree):
    result, stack = {()}, [tree]
    while stack:
        node = stack.pop()
        for child in node.children:
            if bare_pub_module(node, child) and not test_gated(child.declaration.attributes):
                result.add(child.names)
                stack.append(child)
    return result


def bare_pub_module(parent, child):
    stream = tokens(parent.text[child.declaration.start:child.declaration.end])
    index = next(i for i, t in enumerate(stream) if t.value == 'mod')
    return index >= 1 and stream[index - 1].value == 'pub'
