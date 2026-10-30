"""Rebase compile-time filesystem operands without relying on a test's cwd."""
import json
from functools import lru_cache
from pathlib import Path

from rust_source import pairs, replace_spans, string_value, tokens


def relative(path, parent):
    import os
    return Path(os.path.relpath(path, parent)).as_posix()


def absolute_operand(target, compiled_root):
    return 'concat!(env!("CARGO_MANIFEST_DIR"), ' + json.dumps('/' + relative(target, compiled_root)) + ')'


class Destinations:
    def __init__(self, owner, assignments):
        self.owner = owner
        self.rows = {path: row['destination'] for path, row in assignments.items()
                     if row['package'] == owner['cargo']['name']}
        self.suffixes = {}
        for path, destination in self.rows.items():
            if path.suffix != '.rs':
                continue
            parts = path.relative_to(owner['root']).parts
            for n in range(len(parts) - 1):
                key = '/'.join(parts[n:])
                previous = self.suffixes.get(key, destination)
                self.suffixes[key] = destination if previous == destination else None

    @lru_cache(maxsize=None)
    def target(self, value):
        path = (self.owner['root'] / value.lstrip('/')).resolve()
        if path in self.rows:
            return self.rows[path]
        if value in self.suffixes:
            return self.suffixes[value]
        # Preserve a directory/glob only when every moved child has the same
        # relative location under one new directory. Never choose the first reader.
        wildcard = value.find('*')
        prefix = value[:wildcard].rsplit('/', 1)[0] if wildcard >= 0 else value
        directory = (self.owner['root'] / prefix.lstrip('/')).resolve()
        candidates = []
        for source, destination in self.rows.items():
            if source.is_relative_to(directory):
                remainder = source.relative_to(directory)
                base = destination
                for _ in remainder.parts:
                    base = base.parent
                if base / remainder != destination:
                    return None
                candidates.append(base)
        if candidates and len(set(candidates)) == 1:
            base = candidates[0]
            return base / value[len(prefix):].lstrip('/') if wildcard >= 0 else base
        return None


def rewrite(text, source, destination, owner, destinations):
    compiled_root = next((p for p in destination.parents if p.parent.name == 'crates'), owner['root'])
    stream, changes, receipts, covered = tokens(text), [], [], []
    matching = pairs(stream)

    def edit(start, end, replacement, kind):
        if text[start:end] != replacement:
            changes.append((start, end, replacement))
            receipts.append({'source': str(source), 'offset': start, 'kind': kind,
                             'before': text[start:end], 'after': replacement})
            covered.append((start, end))

    # Handle concat!(env!(...), "/tests/...") as one operand; concatenating
    # two independently rebased absolute paths would silently corrupt the path.
    for i, token in enumerate(stream[:-2]):
        if token.value != 'concat' or [t.value for t in stream[i + 1:i + 3]] != ['!', '(']:
            continue
        end = matching[i + 2]
        part = stream[i + 3:end]
        strings = [t for t in part if t.kind == 'string']
        if len(strings) != 2 or string_value(strings[0]) != 'CARGO_MANIFEST_DIR':
            continue
        value = string_value(strings[1])
        target = destinations.target(value) if value.startswith(('/tests/', 'tests/')) else None
        if target is not None:
            edit(token.start, stream[end].end, absolute_operand(target, compiled_root), 'manifest-path')
    for i, token in enumerate(stream):
        if any(a <= token.start < b for a, b in covered):
            continue
        if token.value == 'env' and i + 4 < len(stream) and [t.value for t in stream[i + 1:i + 3]] == ['!', '(']:
            if stream[i + 3].kind == 'string' and string_value(stream[i + 3]) == 'CARGO_MANIFEST_DIR':
                end = matching[i + 2]
                if compiled_root != owner['root']:
                    edit(token.start, stream[end].end, absolute_operand(owner['root'], compiled_root), 'manifest-owner-root')
                continue
        if token.kind != 'string':
            continue
        value = string_value(token)
        if '/' not in value:
            continue
        # Includes retain their source-file-relative rule in source_rewrite.
        if i >= 3 and stream[i - 3].value in ('include', 'include_str', 'include_bytes'):
            continue
        # A prefix assertion is a source-identity claim, not a filesystem read.
        if i >= 2 and stream[i - 2].value in ('starts_with', 'ends_with', 'contains'):
            continue
        target = destinations.target(value)
        if target is not None:
            edit(token.start, token.end, absolute_operand(target, compiled_root), 'manifest-path')
    return replace_spans(text, changes), receipts
