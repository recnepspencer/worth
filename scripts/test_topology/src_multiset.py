"""Test-name and body multisets whose bodies ignore path qualification.

Moving a test rewrites `crate::a::b::Item` to its canonical path, so a raw body hash would
change for every moved test. Each maximal `seg::seg::...` path is reduced to its last
segment before hashing: the body proves identical apart from where its names now live.
"""
from collections import Counter
import hashlib

from filesystem_path import filesystem_path
from module_tree import test_names
from rust_source import pairs, tokens
from src_paths import flatten


def normalized_bodies(text):
    names = set(test_names(text))
    stream = tokens(text)
    matching = pairs(stream)
    result = []
    for i, token in enumerate(stream):
        if token.value == 'fn' and i + 1 < len(stream) and stream[i + 1].value in names:
            body = next((j for j in range(i + 2, len(stream)) if stream[j].value in ('{', ';')), None)
            if body is None or stream[body].value != '{':
                continue
            values = without_import_shape(stream, matching, i, matching[body] + 1)
            result.append((stream[i + 1].value, digest(values)))
    return result


def without_import_shape(stream, matching, start, end):
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
        values.append(token.value)
        k += 1
    return values + ['use'] + sorted(bound_all)


def digest(values):
    reduced, k = [], 0
    while k < len(values):
        if k + 2 < len(values) and values[k + 1] == ':' and values[k + 2] == ':' and values[k] not in (':', '<', '>'):
            k += 3
            continue
        reduced.append(values[k])
        k += 1
    return hashlib.sha256(' '.join(reduced).encode()).hexdigest()[:16]


def multiset_of_texts(texts):
    counts = Counter()
    for text in texts:
        counts.update(normalized_bodies(text))
    return counts


def multiset_of_files(root, paths):
    return multiset_of_texts(filesystem_path(root / p).read_text(encoding='utf-8') for p in paths)
