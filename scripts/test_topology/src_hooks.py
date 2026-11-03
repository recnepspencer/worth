"""Classify the `cfg(test)` production items that moved tests need.

From the gated items the moved code names, walk the gated closure inside production:
paths and bare sibling names in each item, gated `impl` blocks of a gated type, and gated
child modules. An item is test-dependent when its closure reaches test-family code (a
moving family, or any other `cfg(test)` module): production can never depend on a testkit,
so a test-dependent item is test support that only lived in production for private access
and moves to the testkit. Every other item in the closure is compiled into production.
"""
from dataclasses import dataclass, field

from rust_source import tokens
from src_families import test_gated
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
        if keyword == 'impl' and not test_gated(attributes):
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


def impl_target(text):
    """Self type of an `impl` header: the first type after `for`, else after the generics."""
    stream = tokens(text)
    start = next((k for k, t in enumerate(stream) if t.value == 'impl'), None)
    if start is None:
        return None
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
    values = [t.value for t in header]
    if 'for' in values:
        header = header[values.index('for') + 1:]
    return next((t.value for t in header if t.kind == 'ident' and t.value not in ('dyn', 'mut', 'const')), None)


def classify(model, seeds, family_seeds=()):
    """seeds: gated production items moved code names; family_seeds: support modules it calls into."""
    gated, nodes = {}, {}
    moving = set(model.moving)
    pending = sorted(seeds) + [('<family>', f) for f in sorted(family_seeds)]
    seen = set()
    while pending:
        key = pending.pop()
        if key in seen:
            continue
        seen.add(key)
        if key[0] == '<family>':
            family = key[1]
            if family in moving or has_tests(model, family):
                nodes[key] = ([], {('<test>', family)})
                continue
            texts = [(n.names, n.own_text()) for n in model.families[family].node.walk()]
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
            for dependency in dependencies(model, names, text, key):
                nodes[key][1].add(dependency)
                pending.append(dependency)
    dependent = {}

    def test_dependent(key, trail=frozenset()):
        if key[0] == '<test>':
            return True
        if key in dependent:
            return dependent[key]
        if key in trail:
            return False
        dependent[key] = any(test_dependent(d, trail | {key}) for d in nodes.get(key, ((), set()))[1])
        return dependent[key]

    classes = HookClasses()
    for key in sorted(nodes, key=str):
        if key[0] == '<test>':
            continue
        if key[0] == '<family>':
            if test_dependent(key):
                classes.family_reads.add(key[1])
            else:
                classes.compiled_families.add(key[1])
        elif test_dependent(key):
            classes.extracted.setdefault(key[0], []).append((key[1], nodes[key][0]))
            classes.family_reads |= {d[1] for d in nodes[key][1] if d[0] == '<family>'}
        else:
            classes.compiled.add(key)
            classes.spans[key] = nodes[key][0]
    return classes


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
    from src_families import function_bodies, macro_generated
    return any(function_bodies(n.own_text()) or macro_generated(n.own_text())
               for n in model.families[family].node.walk())


def family_target(model, reference):
    if not isinstance(reference.module, tuple):
        return None
    for family in model.families:
        if reference.module[:len(family)] == family:
            return family
    return None

