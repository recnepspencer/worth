"""Path occurrences in Rust source with exact offsets: `use` declarations and rooted paths.

A `use` declaration is reported whole (visibility through `;`) with its flattened leaves,
because one braced tree can name items that end up in different crates. A rooted path
(`crate::`, `super::`, `self::`, or a leading child-module or imported-module name) is
reported from its first to its last identifier segment; generic arguments end it.
"""
from dataclasses import dataclass

from rust_source import pairs, tokens
from src_paths import flatten


@dataclass(frozen=True)
class UseDeclaration:
    start: int
    end: int
    visibility: str
    leaves: tuple


@dataclass(frozen=True)
class PathOccurrence:
    start: int
    end: int
    segments: tuple


def scan(text, local_heads=()):
    """(use declarations, path occurrences); `local_heads` are names that root a local path."""
    stream = tokens(text)
    matching = pairs(stream)
    uses, skipped = [], set()
    for i, token in enumerate(stream):
        if token.value != 'use' or token.kind != 'ident' or (i and stream[i - 1].value in ('.', ':')) \
                or precise_capture(stream, i):
            continue
        start, visibility = token.start, ''
        if i and stream[i - 1].value == ')' and i >= 2:
            opening = next((k for k, v in matching.items() if v == i - 1), None)
            if opening is not None and opening >= 1 and stream[opening - 1].value == 'pub':
                start, visibility = stream[opening - 1].start, text[stream[opening - 1].start:stream[i - 1].end]
        elif i and stream[i - 1].value == 'pub':
            start, visibility = stream[i - 1].start, 'pub'
        end = i + 1
        while end < len(stream) and stream[end].value != ';':
            end = matching[end] + 1 if stream[end].value in ('{', '(', '[') else end + 1
        if end >= len(stream):
            continue
        leaves = tuple(flatten(stream, matching, i + 1, end, ()))
        uses.append(UseDeclaration(start, stream[end].end, visibility, leaves))
        skipped.update(range(i, end + 1))
    for i, token in enumerate(stream):
        if token.value == 'pub' and i + 2 < len(stream) and stream[i + 1].value == '(' and stream[i + 2].value == 'in':
            skipped.update(range(i, matching[i + 1] + 1))
    paths = []
    i = 0
    while i < len(stream):
        token = stream[i]
        rooted = token.kind == 'ident' and (token.value in ('crate', 'super', 'self') or token.value in local_heads)
        preceded = i and (stream[i - 1].value in ('.', '$')
                          or (i > 1 and stream[i - 1].value == ':' and stream[i - 2].value == ':'))
        if i in skipped or not rooted or preceded or not colons(stream, i):
            i += 1
            continue
        if token.value in local_heads and i and stream[i - 1].value in ('fn', 'struct', 'enum', 'mod'):
            i += 1
            continue
        segments, j = [token.value], i
        while colons(stream, j) and j + 3 < len(stream) and stream[j + 3].kind == 'ident':
            segments.append(stream[j + 3].value)
            j += 3
        paths.append(PathOccurrence(token.start, stream[j].end, tuple(segments)))
        i = j + 1
    return uses, paths


def precise_capture(stream, i):
    """`use<..>` in a return type's bounds (precise capturing) is not a `use` declaration."""
    return i + 1 < len(stream) and stream[i + 1].value == '<'


def colons(stream, i):
    return i + 2 < len(stream) and stream[i + 1].value == ':' and stream[i + 2].value == ':'


def visibility_markers(text):
    """(start, end, inner) for every `pub(...)` restriction, so moved code can re-scope it."""
    stream = tokens(text)
    matching = pairs(stream)
    result = []
    for i, token in enumerate(stream):
        if token.value == 'pub' and i + 1 < len(stream) and stream[i + 1].value == '(':
            close = matching[i + 1]
            result.append((token.start, stream[close].end, text[stream[i + 1].end:stream[close].start].strip()))
    return result
