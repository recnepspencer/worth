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
            values = [t.value for t in stream[i:matching[body] + 1]]
            result.append((stream[i + 1].value, digest(values)))
    return result


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
