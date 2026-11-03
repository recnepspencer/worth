"""Classify the `cfg(test)` production items that moved tests need.

From the gated items the moved code names, walk the gated closure inside production:
paths and bare sibling names in each item, gated `impl` blocks of a gated type, and gated
child modules. An item is test-dependent when its closure reaches test-family code (a
moving family, or any other `cfg(test)` module): production can never depend on a testkit,
so a test-dependent item is test support that only lived in production for private access
and moves to the testkit. Every other item in the closure is compiled into production.
"""
from dataclasses import dataclass, field

from rust_source import crate_uses, tokens
from src_families import expanded_text, test_gated
from src_paths import Reference, references
from src_scope import scope_items

ITEM_KEYWORDS = ('fn', 'struct', 'enum', 'const', 'static', 'type', 'trait', 'union')


@dataclass
class HookItem:
    module: tuple
    name: str
    spans: list = field(default_factory=list)


@dataclass
class HookClasses:
    compiled: set = field(default_factory=set)
    extracted: dict = field(default_factory=dict)
    family_reads: set = field(default_factory=set)
    compiled_families: set = field(default_factory=set)
    spans: dict = field(default_factory=dict)
    kept: set = field(default_factory=set)
    orphaned: dict = field(default_factory=dict)
    anchored: set = field(default_factory=set)


class _TextNode:
    def __init__(self, names, text):
        self.names, self._text = names, text
        self.children = []

    def own_text(self):
        return self._text


def gated_items(model, module):
    """name -> [(start, end)] own-text spans of gated items (impls join their self type)."""
    node = model.index[module]
    own = node.own_text()
    result = {}
    for attributes, _vis, keyword, name, start, end in scope_items(own):
        if not test_gated(attributes):
            continue
        if keyword in ITEM_KEYWORDS or keyword == 'macro_rules':
            result.setdefault(name, []).append((start, end))
        elif keyword == 'use':
            from src_paths import use_trees
            for segments, glob, alias in use_trees(own[start:end]):
                if segments and not glob:
                    result.setdefault(alias or segments[-1], []).append((start, end))
        elif keyword == 'impl':
            target = impl_target(own[start:end])
            if target:
                result.setdefault(target, []).append((start, end))
        elif keyword not in ('use', 'mod') and '!' in own[start:end][:120]:
            stream = tokens(own[start:end])
            for i, token in enumerate(stream[:-1]):
                if token.value in ('static', 'const') and stream[i + 1].kind == 'ident':
                    result.setdefault(stream[i + 1].value, []).append((start, end))
    for attributes, _vis, keyword, name, start, end in scope_items(own):
        if keyword == 'impl' and not test_gated(attributes) and not trait_impl(own[start:end]):
            for method, ms, me, gated in impl_methods(own[start:end]):
                if gated:
                    result.setdefault(method, []).append((start + ms, start + me))
    return result


def impl_methods(text):
    """(method name, start, end, gated) for each fn in an impl body, offsets in `text`."""
    from rust_source import pairs
    stream = tokens(text)
    opening = next((i for i, t in enumerate(stream) if t.value == '{'), None)
    if opening is None:
        return []
    body_start = stream[opening].end
    body = text[body_start:stream[pairs(stream)[opening]].start]
    return [(name, body_start + start, body_start + end, test_gated(attributes))
            for attributes, _v, keyword, name, start, end in scope_items(body) if keyword == 'fn']


def method_seeds(model, called):
    """Gated production methods and support-module impls that moved code calls with `.name`."""
    items, families = set(), set()
    moving = set(model.moving)
    for names, node in model.index.items():
        if model.production.in_family(names):
            continue
        own = node.own_text()
        for attributes, _vis, keyword, name, start, end in scope_items(own):
            if keyword != 'impl':
                continue
            target = impl_target(own[start:end])
            for method, _ms, _me, gated in impl_methods(own[start:end]):
                if method in called and (gated or test_gated(attributes)):
                    items.add((names, method if not test_gated(attributes) else target))
    for family, record in model.families.items():
        if family in moving or has_tests(model, family):
            continue
        for node in record.node.walk():
            own = node.own_text()
            for _attributes, _vis, keyword, _name, start, end in scope_items(own):
                if keyword == 'impl' and any(m in called for m, *_ in impl_methods(own[start:end])):
                    families.add(family)
    return items, families


def trait_impl(text):
    """`impl Trait for Type`: its methods take the trait's visibility."""
    stream = tokens(text)
    depth = 0
    for token in stream:
        if token.value == '{' and depth == 0:
            return False
        depth += {'<': 1, '>': -1}.get(token.value, 0)
        if token.value == 'for' and depth == 0:
            return True
    return False


def impl_target(text):
    """Self type of an `impl` header: the first type after `for`, else after the generics."""
    path = impl_self_path(text)
    return path[-1] if path else None


def impl_self_path(text):
    """The self type's path segments of an `impl` item."""
    stream = tokens(text)
    start = next((k for k, t in enumerate(stream) if t.value == 'impl'), None)
    if start is None:
        return ()
    header, depth = [], 0
    for token in stream[start + 1:]:
        if token.value in ('{', 'where') and depth == 0:
            break
        if token.value == '<':
            depth += 1
        elif token.value == '>':
            depth -= 1
        elif depth == 0:
            header.append(token)
    return impl_target_path(header)


def impl_target_path(header):
    """Segments of the self type path among an impl header's depth-0 tokens (`crate::a::T`)."""
    values = [t.value for t in header]
    if 'for' in values:
        header = header[values.index('for') + 1:]
    path, k = [], 0
    while k < len(header) and not (header[k].kind == 'ident' and header[k].value not in ('dyn', 'mut', 'const')):
        k += 1
    while k < len(header) and header[k].kind == 'ident':
        path.append(header[k].value)
        if k + 2 < len(header) and header[k + 1].value == ':' and header[k + 2].value == ':':
            k += 3
        else:
            break
    return tuple(path)


def classify(model, seeds, family_seeds=(), referenced=(), dev_aliases=(), method_edges=None):
    """Partition the gated production closure of `seeds` and `family_seeds`.

    referenced: items the moved code names (paths and called methods); only test-dependent
    items it reaches (and their test-dependent closure) are extracted, the rest stay gated.
    dev_aliases: crates only the library's tests may use; reaching one is test-dependent.
    """
    gated, nodes = {}, {}
    moving = set(model.moving)
    candidates = set()
    pending = sorted(seeds) + [('<family>', f) for f in sorted(family_seeds)]
    seen = set()
    while pending:
        key = pending.pop()
        if key in seen:
            continue
        seen.add(key)
        if key[0] == '<family>':
            family = key[1]
            if has_tests(model, family):
                nodes[key] = ([], {('<test>', family)})
                continue
            if family in moving:
                # Moving support that production test code reaches: it stays if it is self-contained.
                candidates.add(family)
            texts = [(n.names, expanded_text(n)) for n in model.families[family].node.walk()]
            nodes[key] = ([], set())
        else:
            module, name = key
            if module not in gated:
                gated[module] = gated_items(model, module) if module in model.index else {}
            spans = gated[module].get(name, [])
            own = model.index[module].own_text() if module in model.index else ''
            texts = [(module, '\n'.join(own[s:e] for s, e in spans))]
            nodes[key] = (spans, set())
            words = {t.value for t in tokens(texts[0][1]) if t.kind == 'ident'}
            for sibling in sorted(words & set(gated[module])):
                if sibling != name:
                    nodes[key][1].add((module, sibling))
                    pending.append((module, sibling))
            for child in model.index[module].children if module in model.index else []:
                if child.names[-1] in words and child.names in model.families:
                    nodes[key][1].add(('<family>', child.names))
                    pending.append(('<family>', child.names))
        for names, text in texts:
            if dev_aliases and crate_uses(text, set(dev_aliases)):
                nodes[key][1].add(('<test>', 'dev-dependency'))
            # Compiler evidence: code in this module calls a method a test-only module defines.
            for module, families in (method_edges or {}).items():
                if names[:len(module)] == module:
                    for family in families:
                        target = ('<family>', family)
                        nodes[key][1].add(target)
                        pending.append(target)
            for dependency in dependencies(model, names, text, key):
                nodes[key][1].add(dependency)
                pending.append(dependency)
    reverse = {}
    for key, (_spans, deps) in nodes.items():
        for dep in deps:
            reverse.setdefault(dep, set()).add(key)
    dependent, because = set(), {}
    stack = [(d, None) for deps in (v[1] for v in nodes.values()) for d in deps if d[0] == '<test>']
    while stack:
        key, via = stack.pop()
        if key in dependent:
            continue
        dependent.add(key)
        because[key] = via
        stack.extend((r, key) for r in reverse.get(key, ()))

    def test_dependent(key):
        return key in dependent

    moving_dependent, stack = set(), [('<test>', f) for f in moving if f not in candidates or has_tests(model, f)]
    while stack:
        key = stack.pop()
        if key in moving_dependent:
            continue
        moving_dependent.add(key)
        stack.extend(reverse.get(key, ()))

    classes = HookClasses()
    classes.because = because
    referenced = set(referenced)
    extract, stack = set(), [k for k in sorted(nodes, key=str) if k in referenced and k[0] not in ('<test>', '<family>')
                             and test_dependent(k)]
    while stack:
        key = stack.pop()
        if key in extract or key[0] in ('<test>', '<family>') or not test_dependent(key) or is_import(model, key):
            continue
        extract.add(key)
        stack.extend(sorted(nodes.get(key, ((), set()))[1], key=str))
    for key in sorted(nodes, key=str):
        if key[0] == '<test>':
            continue
        if key[0] == '<family>':
            if test_dependent(key):
                classes.family_reads.add(key[1])
            elif key[1] in candidates:
                classes.anchored.add(key[1])
            else:
                classes.compiled_families.add(key[1])
        elif key in extract:
            classes.extracted.setdefault(key[0], []).append((key[1], nodes[key][0]))
            classes.family_reads |= {d[1] for d in nodes[key][1] if d[0] == '<family>'}
        elif key in moving_dependent:
            classes.orphaned.setdefault(key[0], []).append((key[1], nodes[key][0]))
        elif test_dependent(key):
            classes.kept.add(key)
        else:
            classes.compiled.add(key)
            classes.spans[key] = nodes[key][0]
    return classes


def is_import(model, key):
    module, name = key
    scope = model.production.scopes.get(module)
    return scope is not None and name in scope.imports and name not in scope.declared


def all_gated_seeds(model):
    """Every gated item of every production module."""
    seeds = set()
    for names in model.index:
        if model.production.in_family(names):
            continue
        seeds |= {(names, name) for name in gated_items(model, names)}
    return seeds


def staying_support(model):
    """Staying `cfg(test)` modules that hold no tests."""
    moving = set(model.moving)
    return {f for f in model.families if f not in moving and not has_tests(model, f)}


def dependencies(model, names, text, key):
    """Families and gated production items one text reaches, resolved from module `names`."""
    found = set()
    own_family = key[1] if key[0] == '<family>' else None
    for reference in references(_TextNode(names, text), model.index):
        target = family_target(model, reference)
        if target is None and reference.item is not None:
            through, _ = model.everything.origin(reference.module, reference.item)
            if through and through[0] != '<external>':
                target = family_target(model, Reference(through[0], through[1]))
        if target is not None:
            if target != own_family:
                found.add(('<family>', target))
            continue
        if reference.item is None:
            continue
        origin, is_gated = model.production.origin(reference.module, reference.item)
        if origin and origin[0] != '<external>' and is_gated:
            found.add(origin)
    return found


def has_tests(model, family):
    cache = model.__dict__.setdefault('_has_tests', {})
    if family not in cache:
        cache[family] = _has_tests(model, family)
    return cache[family]


def _has_tests(model, family):
    from src_families import function_bodies, macro_generated
    return any(function_bodies(expanded_text(n)) or macro_generated(expanded_text(n))
               for n in model.families[family].node.walk())


def family_target(model, reference):
    if not isinstance(reference.module, tuple):
        return None
    for family in model.families:
        if reference.module[:len(family)] == family:
            return family
    return None

