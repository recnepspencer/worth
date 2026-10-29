"""Lossless top-level Rust item boundaries for test/support separation."""
from dataclasses import dataclass

from module_tree import test_names
from rust_source import pairs, tokens


@dataclass(frozen=True)
class Item:
    start: int
    end: int
    keyword: str
    name: str
    visibility: tuple
    body: tuple | None
    test: bool


def items(text):
    stream, result = tokens(text), []
    matching = pairs(stream)
    i = 0
    while i < len(stream):
        first = i
        while stream[i].value == '#':
            bracket = i + 2 if stream[i + 1].value == '!' else i + 1
            end = matching[bracket]
            if bracket == i + 2:
                result.append(Item(stream[i].start, stream[end].end, 'inner-attribute', '', (0, 0), None, False))
                i, first = end + 1, end + 1
                if i >= len(stream):
                    break
            else:
                i = end + 1
        if i >= len(stream):
            break
        vis = (stream[i].start, stream[i].start)
        if stream[i].value == 'pub':
            start = i
            i += 1
            if stream[i].value == '(':
                i = matching[i] + 1
            vis = (stream[start].start, stream[i].start)
        while stream[i].value in ('async', 'unsafe', 'default') or (stream[i].value == 'const' and i + 1 < len(stream) and stream[i + 1].value == 'fn'):
            i += 1
        keyword = stream[i].value
        name = stream[i + 1].value if i + 1 < len(stream) else ''
        body, j, angle = None, i + 1, 0
        semicolon = keyword in ('use', 'const', 'static', 'type') or (keyword == 'extern' and name == 'crate')
        while j < len(stream):
            value = stream[j].value
            if value == ';':
                break
            if value == '<':
                angle += 1
            elif value == '>':
                angle = max(0, angle - 1)
            if value in ('(', '[', '{'):
                close = matching[j]
                if value == '{' and not semicolon and not angle:
                    body = (stream[j].end, stream[close].start)
                    j = close
                    if j + 1 < len(stream) and stream[j + 1].value == ';':
                        j += 1
                    break
                if keyword == 'struct' and value == '(' and not angle:
                    body = (stream[j].end, stream[close].start)
                j = close
            j += 1
        if j >= len(stream):
            raise ValueError('unterminated Rust item near ' + text[stream[first].start:][:100])
        start, end = stream[first].start, stream[j].end
        opaque = keyword not in ('use', 'const', 'static', 'type', 'fn', 'struct', 'enum', 'trait', 'impl', 'mod', 'extern')
        result.append(Item(start, end, 'macro' if opaque else keyword, name, vis, body,
                           keyword == 'fn' and bool(test_names(text[start:end]))))
        i = j + 1
    return result


def partition(text):
    """Keep test bodies/macros and module mounts in the test source; extract meaning."""
    support, tests, pos = [], [], 0
    for item in items(text):
        fragment = text[pos:item.end]
        if item.keyword == 'inner-attribute':
            support.append(fragment)
            tests.append(fragment)
        elif item.test or item.keyword in ('mod', 'macro'):
            tests.append(fragment)
        else:
            support.append(fragment)
            if item.keyword == 'use':
                tests.append(fragment)
        pos = item.end
    tests.append(text[pos:])
    return ''.join(support), ''.join(tests)
