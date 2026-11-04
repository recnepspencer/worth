"""Test-name and body multisets whose bodies ignore path qualification.

Moving a test rewrites `crate::a::b::Item` to its canonical path, so a raw body hash would
change for every moved test. Each maximal `seg::seg::...` path is reduced to its last
segment before hashing: the body proves identical apart from where its names now live.
"""
from collections import Counter
import hashlib

from filesystem_path import filesystem_path
from module_tree import test_names
import posixpath
import re

from rust_source import pairs, string_value, tokens
from src_paths import flatten


TEST_FN = re.compile(r'#\s*\[\s*(?:\w+\s*::\s*)*(?:test|rstest)\s*(?:\]|\()(?:[^\[\]]|\[[^\]]*\])*?'
                     r'(?:#\s*\[[^\]]*\]\s*)*\s*(?:pub(?:\s*\([^)]*\))?\s+)?(?:async\s+)?fn\s+(\w+)')


def test_function_names(text):
    """Functions under a real test attribute; `cfg(test)` is a gate, not a test."""
    return set(TEST_FN.findall(text))


def normalized_bodies(text, source=None):
    names = test_function_names(text)
    stream = tokens(text)
    matching = pairs(stream)
    result = []
    for i, token in enumerate(stream):
        if token.value == 'fn' and i + 1 < len(stream) and stream[i + 1].value in names:
            body = next((j for j in range(i + 2, len(stream)) if stream[j].value in ('{', ';')), None)
            if body is None or stream[body].value != '{':
                continue
            values = without_import_shape(stream, matching, i, matching[body] + 1, source)
            result.append((stream[i + 1].value, digest(values)))
    return result


def without_import_shape(stream, matching, start, end, source=None):
    """Token values with every `use` statement replaced by one sorted set of the names bound.

    The move splits braced imports into one explicit import per leaf; the names a body
    can use are what must stay identical, not how the imports were grouped.
    """
    values, bound_all, k = [], [], start
    while k < end:
        token = stream[k]
        if token.value == 'use' and token.kind == 'ident' and not (k and stream[k - 1].value in ('.', ':')):
            close = k + 1
            while close < end and stream[close].value != ';':
                close = matching[close] + 1 if stream[close].value in ('{', '(', '[') else close + 1
            bound = sorted('*' if glob else (alias or (segments[-1] if segments else ''))
                           for segments, glob, alias in flatten(stream, matching, k + 1, close, ()))
            if values and values[-1] in ('pub', ')'):
                while values and values[-1] != 'pub':
                    values.pop()
                if values:
                    values.pop()
            bound_all += bound
            k = close + 1
            continue
        values.append(included_target(stream, k, source) or token.value)
        k += 1
    return values + ['use'] + sorted(bound_all)


def included_target(stream, k, source):
    """A relative `include*!` literal compares as the repository file it names."""
    token = stream[k]
    if source is None or token.kind != 'string' or k < 3 or stream[k - 1].value != '(' \
            or stream[k - 2].value != '!' or not stream[k - 3].value.startswith('include'):
        return None
    literal = string_value(token)
    if literal.startswith('/'):
        return None
    return 'file:' + posixpath.normpath(posixpath.join(posixpath.dirname(source), literal))


def digest(values):
    reduced, k = [], 0
    while k < len(values):
        if k + 2 < len(values) and values[k + 1] == ':' and values[k + 2] == ':' and values[k] not in (':', '<', '>'):
            k += 3
            continue
        reduced.append(values[k])
        k += 1
    return hashlib.sha256(' '.join(reduced).encode()).hexdigest()[:16]


def multiset_of_texts(sources):
    """sources: (text, repository-relative source path) pairs."""
    counts = Counter()
    for text, source in sources:
        counts.update(normalized_bodies(text, source))
    return counts


def multiset_of_files(root, paths):
    return multiset_of_texts((filesystem_path(root / p).read_text(encoding='utf-8'), p) for p in paths)
