"""Impl blocks of library types that a moving test module defines, which must stay in the library.

Rust's orphan rule keeps an inherent impl in its type's crate, and keeps an impl of a library
or external trait for a library type in the library too. Such impls are test doors that only
lived in a test module: they stay compiled at the same module path, in an inline module that
replaces the moved module's declaration, while the module's tests move. An impl of a trait the
moved code defines may move with that trait.
"""
from rust_source import tokens
from src_families import test_gated
from src_paths import resolve, use_trees
from src_scope import scope_items

TYPE_KEYWORDS = ('struct', 'enum', 'trait', 'type', 'union')


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


def mark_kept_impls(model):
    """Set `record.kept_impls` (own-text spans) for every moving record; return their count."""
    from src_hooks import impl_target
    moved_types = {item.name for record in model.records.values() if not record.extracted
                   for item in record.items if item.keyword in TYPE_KEYWORDS}
    count = 0
    for record in model.records.values():
        record.kept_impls = []
        if record.extracted or record.foreign:
            continue
        own = record.node.own_text()
        for item in record.items:
            if item.keyword != 'impl':
                continue
            text = own[item.start:item.end]
            target, trait = impl_target(text), impl_trait(text)
            if target and target not in moved_types and (trait is None or trait not in moved_types):
                record.kept_impls.append((item.start, item.end))
                count += 1
    return count


def has_kept(model, names):
    return any(model.records[n].kept_impls for n in model.records if n[:len(names)] == names)


def kept_module_text(model, names):
    """Inline module source that keeps a moving module's library impls at the same path."""
    if not has_kept(model, names):
        return ''
    record = model.records[names]
    node = record.node
    own = node.own_text()
    body = [own[s:e] for s, e in sorted(record.kept_impls)]
    uses = []
    for attributes, visibility, keyword, _name, start, end in scope_items(own):
        if keyword == 'use' and production_only(model, names, own[start:end]):
            uses.append(own[start:end])
    children = [kept_module_text(model, child.names) for child in node.children if child.names in model.records]
    attributes = kept_attributes(node.declaration.attributes)
    inner = '\n'.join(uses + body + [c for c in children if c])
    return f'{attributes}mod {names[-1]} {{\n{inner}\n}}\n'


def production_only(model, names, text):
    """A `use` item none of whose leaves resolves into moved code."""
    for segments, _glob, _alias in use_trees(text):
        resolved = resolve(segments, names, model.index)
        if resolved is not None and any(resolved[0][:len(n)] == n for n in model.records
                                        if not model.records[n].extracted):
            return False
    return True


def kept_attributes(attributes):
    import re
    kept = [a for a in re.findall(r'#\s*\[(?:[^\[\]]|\[[^\]]*\])*\]', attributes)
            if not re.match(r'#\s*\[\s*path\s*=', a) and not test_gated(a)]
    return ''.join(a + '\n' for a in kept)
