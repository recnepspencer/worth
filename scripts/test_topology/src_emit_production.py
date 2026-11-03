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
from src_paths import use_trees
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


def production_edits(model, families, widen_modules, widen_items, hooks, widen_imports=()):
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
    for module, name in sorted(widen_imports):
        node = model.index.get(module)
        if node is None:
            continue
        own = node.own_text()
        for attributes, _vis, keyword, _item, start, end in scope_items(own):
            if keyword != 'use' or not any((alias or (segments[-1] if segments else None)) == name
                                           for segments, _glob, alias in use_trees(own[start:end])):
                continue
            offset = own_offset(node, start)
            change = widen(node.text, offset, offset + (end - start))
            if change:
                s, e = absolute(node, change[0], change[1])
                edits[file_of(node)].append((s, e, change[2]))
            if test_gated(attributes):
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
            if re.match(r'(?:#\s*\[[^\]]*\]\s*)*(?:pub(?:\s*\([^)]*\))?\s+)?(?:mod|use)\b', following):
                continue
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


def apply_edits(text, changes, source=''):
    unique = sorted(set(changes))
    removals = [(a, b) for a, b, replacement in unique if replacement == '' and b > a]
    # An edit inside a removed span is moot: the whole span leaves production.
    unique = [(a, b, x) for a, b, x in unique
              if not any(ra <= a and b <= rb and (ra, rb) != (a, b) for ra, rb in removals)]
    for (a, b, x), (c, d, y) in zip(unique, unique[1:]):
        if c < b:
            raise ValueError(f'overlapping production edits in {source}: {(a, b, x)!r} {(c, d, y)!r}: '
                             f'{text[a:b][:120]!r} / {text[c:d][:120]!r}')
    return replace_spans(text, unique)


def hook_edits(model, classes, referenced, called):
    """Compile non-test-dependent hooks, compile support modules in place, remove extracted items.

    referenced: (module, name) items moved code names; called: method names it calls with `.`.
    Compiled items lose their exact `#[cfg(test)]`; those moved code names, and methods it calls,
    become `pub`. Extracted items leave production (they were test-only code).
    """
    from src_hooks import impl_methods
    edits = defaultdict(list)
    for key in sorted(classes.compiled, key=str):
        module, name = key
        node = model.index[module]
        for start, end in classes.spans.get(key, []):
            offset = own_offset(node, start)
            stop = offset + (end - start)
            for gate in gate_spans(node.text, offset, stop):
                edits[file_of(node)].append(absolute(node, *gate) + ('',))
            body = node.text[offset:stop]
            is_impl = re.search(r'^\s*(?:#\s*\[[^\]]*\]\s*)*(?:unsafe\s+)?impl\b', body) is not None
            is_use = re.search(r'^\s*(?:#\s*\[[^\]]*\]\s*)*(?:pub(?:\s*\([^)]*\))?\s+)?use\b', body) is not None
            if not is_impl and not is_use and (key in referenced or name in called):
                change = widen(node.text, offset, stop)
                if change:
                    edits[file_of(node)].append(absolute(node, change[0], change[1]) + (change[2],))
            if is_impl and not trait_impl(body):
                edits[file_of(node)].extend(method_widenings(node, offset, body, called, impl_methods))
    for family in sorted(classes.compiled_families):
        node = model.index[family]
        parent = model.index[family[:-1]]
        start, end = node.declaration.start, node.declaration.end
        for gate in gate_spans(parent.text, start, end):
            edits[file_of(parent)].append(absolute(parent, *gate) + ('',))
        for member in node.walk():
            for attributes, _vis, keyword, _name, s, e in scope_items(member.own_text()):
                if keyword == 'impl':
                    offset = own_offset(member, s)
                    body = member.text[offset:offset + (e - s)]
                    if not trait_impl(body):
                        edits[file_of(member)].extend(method_widenings(member, offset, body, called, impl_methods))
    for removed in (classes.extracted, classes.orphaned):
        for module, items in sorted(removed.items()):
            node = model.index[module]
            for _name, spans in items:
                for start, end in spans:
                    offset = with_docs(node.text, own_offset(node, start))
                    edits[file_of(node)].append(absolute(node, offset, own_offset(node, start) + (end - start)) + ('',))
    return edits


def with_docs(text, start):
    """Extend a removal start back over the item's preceding `///` doc-comment lines."""
    line_start = text.rfind('\n', 0, start) + 1
    if text[line_start:start].strip():
        return start
    earliest = start
    while line_start > 0:
        previous = text.rfind('\n', 0, line_start - 1) + 1
        if not text[previous:line_start].strip().startswith('///'):
            break
        earliest = line_start = previous
    return earliest


GATE = re.compile(r'#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]\s*')


def residual_gates(text, skip_spans, kept_names):
    """Remaining exact `#[cfg(test)]` attributes to remove, outside test code and kept items."""
    result = []
    for match in GATE.finditer(text):
        if any(a <= match.start() < b for a, b in skip_spans):
            continue
        following = text[match.end():]
        if re.match(r'(?:#\s*\[[^\]]*\]\s*)*(?:pub(?:\s*\([^)]*\))?\s+)?mod\b', following):
            continue
        if set(re.findall(r'\w+', following[:unit_extent(following)])) & kept_names:
            continue
        result.append((match.start(), match.end(), ''))
    for start, end, inner in test_attributes(text):
        if any(a <= start < b for a, b in skip_spans):
            continue
        following = text[end:]
        if set(re.findall(r'\w+', following[:unit_extent(following)])) & kept_names:
            continue
        result.append((start, end, '#[' + inner + ']'))
    return result


def test_attributes(text):
    """(start, end, attributes) for each `#[cfg_attr(test, attributes)]`."""
    result = []
    for match in re.finditer(r'#\s*\[\s*cfg_attr\s*\(\s*test\s*,', text):
        stream = tokens(text[match.start():match.start() + 4000])
        depth, comma, close, end = 0, None, None, None
        for i, token in enumerate(stream):
            if token.value in ('(', '[', '{'):
                depth += 1
            elif token.value in (')', ']', '}'):
                depth -= 1
                if depth == 1 and token.value == ')' and close is None:
                    close = i
                if depth == 0:
                    end = i
                    break
            elif token.value == ',' and depth == 2 and comma is None:
                comma = i
        if comma is None or close is None or end is None:
            continue
        inner = text[match.start() + stream[comma].end:match.start() + stream[close].start].strip()
        result.append((match.start(), match.start() + stream[end].end, inner))
    return result
