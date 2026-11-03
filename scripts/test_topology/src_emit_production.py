"""Production edits for a library whose court-shaped source tests left it.

- every moved family's module declaration (with its attributes) is removed;
- every module on a canonical path the moved code names is declared `pub`;
- every item the moved code names by its defining path is declared `pub`;
- every `cfg(test)` hook the moved code needs is compiled, together with the test-gated
  items and statements of the same file that mention it (recorders and their writers).
All edits to one file are applied together from the original text.
"""
from collections import defaultdict
import re

from rust_source import pairs, replace_spans, tokens
from src_families import test_gated
from src_scope import scope_items

ITEM_KEYWORDS = ('fn', 'struct', 'enum', 'const', 'static', 'type', 'trait', 'union')


def absolute(node, start, end):
    return node.offset + start, node.offset + end


def file_of(node):
    return node.source


def parent_of(model, names):
    return model.index[names[:-1]]


def visibility_span(text, start, end):
    """(start, end) of the visibility before an item's keyword, or an empty span at the keyword."""
    stream = tokens(text[start:end])
    matching = pairs(stream)
    i = 0
    while i + 1 < len(stream) and stream[i].value == '#':
        i = matching[i + 1] + 1
    if i < len(stream) and stream[i].value == 'pub':
        close = matching[i + 1] if i + 1 < len(stream) and stream[i + 1].value == '(' else i
        return start + stream[i].start, start + stream[close].end
    return start + stream[i].start, start + stream[i].start


def widen(text, start, end):
    vs, ve = visibility_span(text, start, end)
    current = text[vs:ve]
    if current == 'pub':
        return None
    return (vs, ve, 'pub' if current else 'pub ')


def production_edits(model, families, widen_modules, widen_items, hooks):
    """{source path: [(start, end, replacement)]} over original file text."""
    edits = defaultdict(list)
    for family in families:
        node = model.index[family]
        parent = parent_of(model, family)
        start, end = absolute(parent, node.declaration.start, node.declaration.end)
        edits[file_of(parent)].append((start, end, ''))
    for module in sorted(widen_modules):
        node = model.index[module]
        parent = parent_of(model, module)
        change = widen(parent.text, node.declaration.start, node.declaration.end)
        if change:
            s, e = absolute(parent, change[0], change[1])
            edits[file_of(parent)].append((s, e, change[2]))
    for module, name in sorted(widen_items | hooks):
        node = model.index.get(module)
        if node is None:
            continue
        own = node.own_text()
        for attributes, _vis, keyword, item, start, end in scope_items(own):
            if item != name or keyword not in ITEM_KEYWORDS:
                continue
            offset = own_offset(node, start)
            change = widen(node.text, offset, offset + (end - start))
            if change:
                s, e = absolute(node, change[0], change[1])
                edits[file_of(node)].append((s, e, change[2]))
            if (module, name) in hooks and test_gated(attributes):
                for gate in gate_spans(node.text, offset, offset + (end - start)):
                    edits[file_of(node)].append(absolute(node, *gate) + ('',))
    return edits


def own_offset(node, own_position):
    """Map an offset in `node.own_text()` back to `node.text` (child declarations removed)."""
    removed = 0
    for child in sorted(node.children, key=lambda c: c.declaration.start):
        span = child.declaration.end - child.declaration.start
        if child.declaration.start - removed <= own_position:
            removed += span
        else:
            break
    return own_position + removed


def gate_spans(text, start, end):
    """Spans of exact `#[cfg(test)]` attributes heading the item at [start, end)."""
    result = []
    for match in re.finditer(r'#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]\s*', text[start:end]):
        result.append((start + match.start(), start + match.end()))
        break
    return result


def ungate_recorders(text, names):
    """Spans of `#[cfg(test)]` on statements/items that mention a compiled hook, to a fixpoint."""
    spans, active, changed = [], set(names), True
    while changed:
        changed = False
        for match in re.finditer(r'#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]\s*', text):
            following = text[match.end():match.end() + 400]
            statement = following.split(';', 1)[0].split('{', 1)[0]
            words = set(re.findall(r'\w+', statement))
            span = (match.start(), match.end())
            if span not in spans and words & active:
                spans.append(span)
                declared = re.search(r'\b(?:fn|static|const|struct|enum|type)\s+(\w+)', statement)
                if declared:
                    active.add(declared[1])
                changed = True
    return spans


def apply_edits(text, changes):
    unique = sorted(set(changes))
    return replace_spans(text, unique)
