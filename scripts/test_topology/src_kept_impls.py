"""Library-type impls of moving test modules, and their closure, stay compiled in the library.

Rust's orphan rule keeps an inherent impl in its type's crate, and keeps an impl of a library
or external trait for a library type there too. A moving test module's impls whose self type
resolves to a library definition are such test doors; they stay at the same module path, with
the non-test items they reach (same-module helpers by name, other moving items by resolved
path), in `pub` inline modules that replace the moved declarations. Their tests move. A type
that resolves into test code or not at all (macro-generated fixtures) is test code: its impls
move with it.
"""
import re

from rust_source import tokens
from src_families import test_gated
from src_paths import resolve, references, use_trees
from src_scope import scope_items

ITEM_KEYWORDS = ('fn', 'struct', 'enum', 'const', 'static', 'type', 'trait', 'union')


def impl_trait(text):
    """The trait of `impl Trait for Type`, or None for an inherent impl."""
    stream = tokens(text)
    start = next((k for k, t in enumerate(stream) if t.value == 'impl'), None)
    if start is None:
        return None
    depth, last = 0, None
    for token in stream[start + 1:]:
        if token.value in ('{', 'where') and depth == 0:
            return None
        if token.value == '<':
            depth += 1
        elif token.value == '>':
            depth -= 1
        elif depth == 0 and token.value == 'for':
            return last
        elif depth == 0 and token.kind == 'ident' and token.value not in ('dyn', 'unsafe', 'const'):
            last = token.value
    return None


class _Text:
    def __init__(self, names, text):
        self.names, self._text, self.children = names, text, []

    def own_text(self):
        return self._text


def in_moved(model, module):
    return any(module[:len(n)] == n for n, r in model.records.items() if not r.extracted)


def library_definition(model, names, name):
    origin, _gated = model.everything.origin(names, name)
    return (origin is not None and origin[0] != '<external>' and not in_moved(model, origin[0])
            and not any(origin[0][:len(f)] == f for f in model.families))


def mark_kept_impls(model):
    """Set `record.kept` (item indices) and `record.kept_impls` (own-text spans); return the count."""
    from src_hooks import impl_target
    work = []
    for names, record in model.records.items():
        record.kept, record.kept_impls = set(), []
        if record.extracted or record.foreign:
            continue
        own = record.node.own_text()
        for index, item in enumerate(record.items):
            if item.keyword != 'impl':
                continue
            text = own[item.start:item.end]
            target, trait = impl_target(text), impl_trait(text)
            trait_origin = model.everything.origin(names, trait)[0] if trait else None
            # Inherent impls become door traits (src_door_traits); library-trait impls stay.
            if target and trait is not None and library_definition(model, names, target) and \
                    (trait_origin is None or not in_moved(model, trait_origin[0])):
                record.kept.add(index)
                work.append((names, index))
    # Production cfg(test) statements (inside function bodies) that call moved items keep those
    # items in the library: recorders the instrumentation writes. Gated items are classified apart.
    for names, node in model.index.items():
        if names in model.records or any(names[:len(f)] == f for f in model.families):
            continue
        statements = gated_statements(node.own_text())
        if not statements:
            continue
        for reference in references(_Text(names, statements), model.index):
            if reference.item is None:
                continue
            origin = model.everything.origin(reference.module, reference.item)[0] or (reference.module, reference.item)
            target = model.records.get(origin[0])
            if target is None or target.extracted or target.foreign:
                continue
            target_own = target.node.own_text()
            for index, other in enumerate(target.items):
                if origin[1] in declared_names(target_own, other) and not other.test \
                        and index not in target.kept:
                    target.kept.add(index)
                    work.append((origin[0], index))
    while work:
        names, index = work.pop()
        record = model.records[names]
        own = record.node.own_text()
        item = record.items[index]
        text = own[item.start:item.end]
        words = {t.value for t in tokens(text) if t.kind == 'ident'}
        for other_index, other in enumerate(record.items):
            if other_index not in record.kept and not other.test and declared_names(own, other) & words:
                record.kept.add(other_index)
                work.append((names, other_index))
        for reference in references(_Text(names, text), model.index):
            if reference.item is None:
                continue
            origin = model.everything.origin(reference.module, reference.item)[0] or (reference.module, reference.item)
            target = model.records.get(origin[0])
            if target is None or target.extracted or target.foreign:
                continue
            target_own = target.node.own_text()
            for other_index, other in enumerate(target.items):
                if origin[1] in declared_names(target_own, other) and not other.test \
                        and other_index not in target.kept:
                    target.kept.add(other_index)
                    work.append((origin[0], other_index))
    count = 0
    for record in model.records.values():
        record.kept_impls = sorted((record.items[i].start, record.items[i].end) for i in record.kept)
        own = record.node.own_text()
        record.kept_names = set().union(*(declared_names(own, record.items[i]) for i in record.kept)) \
            if record.kept else set()
        count += len(record.kept)
    return count


def gated_statements(text):
    """Text of `#[cfg(test)]` statements that sit inside function bodies (not items)."""
    from src_emit_production import GATE, unit_extent
    pieces = []
    for match in GATE.finditer(text):
        following = text[match.end():]
        if re.match(r'(?:#\s*\[[^\]]*\]\s*)*(?:pub(?:\s*\([^)]*\))?\s+)?'
                    r'(?:fn|struct|enum|const|static|type|trait|union|impl|mod|use|macro_rules|thread_local)\b',
                    following):
            continue
        line_start = text.rfind('\n', 0, match.start()) + 1
        if not text[line_start:match.start()].strip() and text[line_start:match.start()] == '':
            continue
        pieces.append(following[:unit_extent(following)])
    return '\n'.join(pieces)


def declared_names(own, item):
    """Names an item declares: its own name, or the statics an item macro (thread_local!) declares."""
    if item.keyword in ITEM_KEYWORDS:
        return {item.name}
    if item.name == '!':
        stream = tokens(own[item.start:item.end])
        return {stream[i + 1].value for i in range(len(stream) - 1)
                if stream[i].value == 'static' and stream[i + 1].kind == 'ident'}
    return set()


def has_kept(model, names):
    return any(r.kept_impls for n, r in model.records.items() if n[:len(names)] == names)


def kept_module_text(model, names):
    """`pub` inline module source that keeps a moving module's library impls and their closure."""
    if not has_kept(model, names):
        return ''
    record = model.records[names]
    own = record.node.own_text()
    body = [public(own[s:e]) for s, e in record.kept_impls]
    uses = [own[start:end] for _a, _v, keyword, _n, start, end in scope_items(own)
            if keyword == 'use' and not reaches_moved(model, names, own[start:end])]
    children = [kept_module_text(model, child.names) for child in record.node.children
                if child.names in model.records]
    attributes = kept_attributes(record.node.declaration.attributes) if record.node.declaration else ''
    inner = '\n'.join(uses + body + [c for c in children if c])
    return f'{attributes}pub mod {names[-1]} {{\n{inner}\n}}\n'


def reaches_moved(model, names, text):
    """Whether a `use` names a moved item that is not kept (it would not resolve)."""
    for segments, glob, _alias in use_trees(text):
        resolved = resolve(segments, names, model.index)
        if resolved is None:
            continue
        module, rest = resolved
        if not in_moved(model, module):
            continue
        if glob or not rest:
            if not has_kept(model, module):
                return True
            continue
        origin = model.everything.origin(module, rest[0])[0] or (module, rest[0])
        owner = model.records.get(origin[0])
        if owner is not None and origin[1] not in getattr(owner, 'kept_names', set()):
            return True
    return False


def public(text):
    """Kept items are test doors the moved courts call: top-level and inherent members become `pub`."""
    text = re.sub(r'(?m)^(\s*(?:#\s*\[[^\]]*\]\s*)*)pub\s*\([^)]*\)\s+', r'\1pub ', text)
    head = re.match(r'\s*(?:#\s*\[[^\]]*\]\s*)*', text).end()
    if re.match(r'(?:fn|struct|enum|const|static|type|trait|union)\b', text[head:]):
        text = text[:head] + 'pub ' + text[head:]
    if re.match(r'impl\b', text[head:]) and not re.search(r'\bfor\b', text[head:text.find('{')]):
        text = re.sub(r'(?m)^(\s+(?:#\s*\[[^\]]*\]\s*)*)(?=(?:const\s+|async\s+|unsafe\s+)*fn\s)', r'\1pub ', text)
    return text


def kept_attributes(attributes):
    kept = [a for a in re.findall(r'#\s*\[(?:[^\[\]]|\[[^\]]*\])*\]', attributes)
            if not re.match(r'#\s*\[\s*path\s*=', a) and not test_gated(a)]
    return ''.join(a + '\n' for a in kept)
