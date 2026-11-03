"""Crate-relative path references in Rust source, resolved against a module tree."""
from dataclasses import dataclass

from rust_source import pairs, tokens


@dataclass(frozen=True)
class Reference:
    module: tuple
    item: str | None
    glob: bool = False
    alias: str | None = None


def use_trees(text):
    """Every `use` declaration flattened to (segments, glob, alias) leaves."""
    stream = tokens(text)
    matching = pairs(stream)
    result = []
    for i, token in enumerate(stream):
        if token.value != 'use' or token.kind != 'ident':
            continue
        if i and stream[i - 1].value in ('.', '::'):
            continue
        end = i + 1
        while end < len(stream) and stream[end].value != ';':
            end = matching[end] + 1 if stream[end].value in ('{', '(', '[') else end + 1
        result.extend(flatten(stream, matching, i + 1, end, ()))
    return result


def flatten(stream, matching, start, end, prefix):
    leaves, segments, i = [], list(prefix), start
    while i < end:
        value = stream[i].value
        if value == '{':
            close = matching[i]
            j = i + 1
            while j < close:
                k = j
                while k < close and stream[k].value != ',':
                    k = matching[k] + 1 if stream[k].value in ('{', '(', '[') else k + 1
                leaves.extend(flatten(stream, matching, j, k, tuple(segments)))
                j = k + 1
            return leaves
        if value == '*':
            return [(tuple(segments), True, None)]
        if value == 'as' and i + 1 < end:
            return [(tuple(segments), False, stream[i + 1].value)]
        if stream[i].kind == 'ident':
            if value == 'self' and segments and len(segments) == len(prefix):
                pass
            else:
                segments.append(value)
        i += 1
    return [(tuple(segments), False, None)] if len(segments) > len(prefix) or segments else leaves


def qualified_paths(text):
    """Expression and type paths rooted at `crate`, `super` or `self`, outside `use` and `pub(in ..)`."""
    stream = tokens(text)
    matching = pairs(stream)
    skipped = set()
    for i, token in enumerate(stream):
        if token.value == 'use' and token.kind == 'ident' and not (i and stream[i - 1].value in ('.', '::')):
            end = i + 1
            while end < len(stream) and stream[end].value != ';':
                end = matching[end] + 1 if stream[end].value in ('{', '(', '[') else end + 1
            skipped.update(range(i, end))
        if token.value == 'pub' and i + 2 < len(stream) and stream[i + 1].value == '(' and stream[i + 2].value == 'in':
            skipped.update(range(i, matching[i + 1] + 1))
    result = []
    i = 0
    while i < len(stream):
        token = stream[i]
        rooted = token.value in ('crate', 'super', 'self') and token.kind == 'ident'
        if i in skipped or not rooted or (i and stream[i - 1].value == ':' ) or not following_colons(stream, i):
            i += 1
            continue
        segments, j = [token.value], i + 1
        while following_colons(stream, j - 1) and j + 2 < len(stream) + 1:
            nxt = stream[j + 2] if j + 2 < len(stream) else None
            if nxt is None or nxt.kind != 'ident':
                break
            segments.append(nxt.value)
            j += 3
        result.append((tuple(segments), False, None))
        i = j
    return result


def following_colons(stream, i):
    return i + 2 < len(stream) and stream[i + 1].value == ':' and stream[i + 2].value == ':'


def module_index(tree):
    return {node.names: node for node in tree.walk()}


def resolve(segments, current, index):
    """Absolute (module, remainder) for a path, or None for an external crate path."""
    if not segments:
        return None
    head, rest = segments[0], list(segments[1:])
    if head == 'crate':
        base = ()
    elif head in ('super', 'self'):
        base = current
        position = 0
        while position < len(segments) and segments[position] in ('super', 'self'):
            if segments[position] == 'super':
                if not base:
                    return None
                base = base[:-1]
            position += 1
        rest = list(segments[position:])
    elif current + (head,) in index:
        base, rest = current + (head,), rest
    else:
        return None
    module = base
    while rest and module + (rest[0],) in index:
        module = module + (rest.pop(0),)
    return module, tuple(rest)


def references(node, index):
    """Resolved crate references made by one module node's own text."""
    text = node.own_text()
    result = []
    for segments, glob, alias in use_trees(text) + qualified_paths(text):
        resolved = resolve(segments, node.names, index)
        if resolved is None:
            continue
        module, rest = resolved
        result.append(Reference(module, rest[0] if rest else None, glob, alias))
    return result
