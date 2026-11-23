"""Rust source projections shared by the Relational DAG checks."""
from collections import namedtuple
from pathlib import Path
import re
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "query_split"))
from module_graph import LibraryModel
from rust_source import pairs, tokens
from src_families import test_gated
from src_paths import flatten, following_colons, qualified_paths, resolve
from src_scope import scope_items
from src_surface import Surface, bare_pub_module

Contact = namedtuple("Contact", "segments glob alias public")


def section(text, spans, keep=False):
    """Mask the other half of a projection, preserving whitespace and token separation."""
    pieces = []
    previous = 0
    for start, end in spans:
        pieces.extend((text[previous:start], text[start:end]))
        previous = end
    pieces.append(text[previous:])
    return ''.join(piece if bool(i % 2) == keep else re.sub(r"\S", " ", piece)
                   for i, piece in enumerate(pieces))


def test_spans(text):
    """Find test-only items, associated items, local statements and macro tokens."""
    if 'cfg' not in text or 'test' not in text:
        return []
    stream = tokens(text)
    matching = pairs(stream)
    result, i = [], 0
    while i + 2 < len(stream):
        if stream[i].value != '#':
            i += 1
            continue
        start = stream[i].start
        bracket = i + 1 + (stream[i + 1].value == '!')
        if stream[bracket].value != '[':
            i += 1
            continue
        end = matching[bracket]
        if not test_gated(text[start:stream[end].end]):
            i = end + 1
            continue
        if bracket == i + 2:
            return [(0, len(text))]
        item = end + 1
        if item >= len(stream):
            break
        while item + 1 < len(stream) and stream[item].value == '#' and stream[item + 1].value == '[':
            item = matching[item + 1] + 1
        end = gated_item_end(stream, matching, item)
        result.append((start, stream[end].end))
        i = end + 1
    return result


def gated_item_end(stream, matching, start):
    """The next item or statement after a positive test predicate."""
    i = start
    head = [t.value for t in stream[start:start + 5]]
    through_semicolon = any(word in head for word in ('use', 'let', 'type', 'static'))
    while i < len(stream):
        token = stream[i].value
        if token in (';', ',', '}', ']', ')'):
            return i if token == ';' or i == start else i - 1
        if i in matching:
            i = matching[i]
            if token == '{' and not through_semicolon:
                return i
        i += 1
    return len(stream) - 1


class RelationalProductionView:
    def __init__(self, node):
        self.names, self.source = node.names, node.source
        text = node.own_text()
        self._text = section(text, test_spans(text))

    def own_text(self):
        return self._text


def identifiers(text):
    return {token.value for token in tokens(text) if token.kind == "ident"}


def contacts(text, aliases=None):
    """Expand use leaves and collect qualified paths, including tokens inside macros."""
    stream = tokens(text)
    matching = pairs(stream)
    use_spans, next_external = {}, 0
    for i, token in enumerate(stream):
        if token.value != 'use' or token.kind != 'ident':
            continue
        if i and stream[i - 1].value in ('.', '::', ':'):
            continue
        end = i + 1
        while end < len(stream) and stream[end].value != ';':
            end = matching[end] + 1 if end in matching else end + 1
        use_spans[i] = end
        public = i > 0 and stream[i - 1].value == 'pub'
        for segments, glob, alias in flatten(stream, matching, i + 1, end, ()):
            if segments:
                yield Contact(segments, glob, alias, public)
    for segments, glob, alias in qualified_paths(text):
        yield Contact(segments, glob, alias, False)
    # Package references matter as soon as the first sibling is physically extracted.
    for i, token in enumerate(stream):
        next_external = use_spans.get(i, next_external)
        if i < next_external or token.kind != 'ident' or not following_colons(stream, i):
            continue
        if i > 1 and stream[i - 1].value == stream[i - 2].value == ':':
            continue
        if token.value.startswith(('worth_relational', 'worth_testkit_relational')):
            yield Contact(path_at(stream, i), False, None, False)
        elif aliases is not None and token.value in aliases:
            yield Contact(aliases[token.value] + path_at(stream, i)[1:], False, None, False)


def path_at(stream, i):
    path = [stream[i].value]
    while following_colons(stream, i) and i + 3 < len(stream) and stream[i + 3].kind == 'ident':
        i += 3
        path.append(stream[i].value)
    return tuple(path)


def resolved_origin(surface, current, segments):
    """Resolve a type/import path through module or item aliases and globs."""
    target = resolve(segments, current, surface.index)
    if target is None:
        found = surface.origin(current, segments[0])[0]
        rest = segments[1:]
    else:
        module, rest = target
        found = (module, None)
    while found is not None and found[0] != '<external>' and found[1] is None and rest:
        module = found[0]
        if module + (rest[0],) in surface.index:
            found = (module + (rest[0],), None)
        else:
            found = surface.origin(module, rest[0])[0]
        rest = rest[1:]
    return found


def alias_target(text, name):
    """Read a nominal RHS of a Rust type alias."""
    for _, _, keyword, item, start, end in scope_items(text):
        if keyword != 'type' or item != name:
            continue
        stream = tokens(text[start:end])
        matching = pairs(stream)
        i = next(j for j, token in enumerate(stream) if token.value == 'type') + 2
        if i < len(stream) and stream[i].value == '<':
            i = angle_end(stream, matching, i) + 1
        while i < len(stream) and stream[i].value != '=':
            i = matching[i] + 1 if i in matching else i + 1
        i += 1
        if i < len(stream) and stream[i].kind == 'ident':
            return path_at(stream, i)
    return None


def self_types(text):
    """Nominal inherent impl targets, without trait impls or generic arguments."""
    stream = tokens(text)
    matching = pairs(stream)
    for i, token in enumerate(stream[:-1]):
        if token.value != 'impl' or token.kind != 'ident':
            continue
        if i and stream[i - 1].value not in (';', '{', '}', ']', 'unsafe'):
            continue
        start = i + 1
        if stream[start].value == '<':
            start = angle_end(stream, matching, start) + 1
        if start >= len(stream) or stream[start].kind != 'ident':
            continue
        path = path_at(stream, start)
        end = start + 3 * (len(path) - 1) + 1
        if end < len(stream) and stream[end].value == '<':
            end = angle_end(stream, matching, end) + 1
        if end < len(stream) and stream[end].value in ('{', 'where'):
            yield path


def angle_end(stream, matching, start):
    depth, i = 1, start + 1
    while i < len(stream):
        value = stream[i].value
        if i in matching:
            i = matching[i]
        elif value == '<':
            depth += 1
        elif value == '>' and stream[i - 1].value != '-':
            depth -= 1
            if not depth:
                return i
        i += 1
    return len(stream)
