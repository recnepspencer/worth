"""Names a module's expanded glob imports must bind for the children that glob the module.

`use super::*` in a child reads its parent's imports too. When the parent's own glob of
production code is expanded to explicit names, the names its globbing descendants use must
be among them, or the children lose them.
"""
from rust_source import tokens
from src_paths import resolve, use_trees


def inherited_words(model, names, cache):
    """Identifiers of descendant modules that reach `names` through `use super::*` chains."""
    if names not in cache:
        cache[names] = set()
        for child in model.index[names].children:
            record = model.records.get(child.names)
            text = record.node.own_text() if record is not None else child.own_text()
            for segments, glob, _alias in use_trees(text):
                resolved = resolve(segments, child.names, model.index) if glob else None
                if resolved is not None and resolved[0] == names and not resolved[1]:
                    cache[names] |= {t.value for t in tokens(text) if t.kind == 'ident'}
                    cache[names] |= inherited_words(model, child.names, cache)
                    break
    return cache[names]
