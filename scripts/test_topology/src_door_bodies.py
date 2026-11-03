"""Body check across the door rewrites: moved tests still have their library bodies.

The test-name and body multiset of the library at the base revision must equal the multiset
of the library and every court and testkit now. Bodies are normalized as for the move
(src_multiset) and, on top, only the member door rewrites are reversed:
`m_for_test` back to `m` for generated method doors, `(*expr.f_for_test())` and
`(*expr.f_mut_for_test())` back to `expr.f` for generated field doors. Any other change to a
body still changes its digest. Item doors only change paths, which the move's normalization
already reduces to their last segment.
"""
from collections import Counter
import json
from pathlib import Path
import subprocess

from rust_source import tokens
from src_multiset import normalized_bodies
import src_multiset

LIBRARY = 'workspaces/worth-query/crates/worth-query-execution/src'
CRATES = 'workspaces/worth-query/crates'


def door_renames(reports):
    """{generated door identifier: (original name, 'method' | 'field')} from member reports."""
    renames = {}
    for report in reports:
        data = json.loads(Path(report).read_text(encoding='utf-8'))
        for entry in data.get('method_doors', []):
            name = entry.rsplit('::', 1)[-1]
            renames[name + '_for_test'] = (name, 'method')
        for entry in data.get('field_doors', []):
            field = entry.split('.')[-1].split(' ')[0]
            renames[field + ('_mut_for_test' if entry.endswith('(mut)') else '_for_test')] = (field, 'field')
    return renames


def without_doors(values, renames):
    """Token values with the generated member door rewrites reversed."""
    values, drop = list(values), set()
    for i, value in enumerate(values):
        if value not in renames:
            continue
        name, kind = renames[value]
        values[i] = name
        if kind != 'field' or values[i + 1:i + 4] != ['(', ')', ')']:
            continue
        depth, j = 0, i + 3
        while j >= 0:
            depth += {')': 1, '(': -1}.get(values[j], 0)
            if depth == 0:
                break
            j -= 1
        if j >= 0 and j + 1 < len(values) and values[j + 1] == '*':
            drop |= {j, j + 1, i + 1, i + 2, i + 3}
    return [v for k, v in enumerate(values) if k not in drop]


def multiset(sources, renames):
    original = src_multiset.digest

    def door_digest(values):
        return original(without_doors(values, renames))
    src_multiset.digest = door_digest
    try:
        counts = Counter()
        for text, source in sources:
            counts.update(normalized_bodies(text, source))
        return counts
    finally:
        src_multiset.digest = original


def base_sources(root, base):
    files = subprocess.run(['git', 'ls-tree', '-r', '--name-only', base, LIBRARY], cwd=root,
                           capture_output=True, text=True, check=True).stdout.split()
    for path in files:
        if path.endswith('.rs'):
            text = subprocess.run(['git', 'show', f'{base}:{path}'], cwd=root, capture_output=True,
                                  check=True).stdout.decode('utf-8')
            yield text, path


def current_sources(root):
    crates = root / CRATES
    roots = [crates / 'worth-query-execution/src'] + [
        d for d in sorted(crates.iterdir())
        if d.name.startswith(('worth-query-court-', 'worth-query-testkit-')) and 'execution' in d.name]
    for directory in roots:
        for path in sorted(directory.rglob('*.rs')):
            yield path.read_text(encoding='utf-8'), str(path.relative_to(root))


def body_check(root, base, reports):
    renames = door_renames(reports)
    before = multiset(base_sources(root, base), renames)
    after = multiset(current_sources(root), renames)
    missing, extra = before - after, after - before
    return {'tests_before': sum(before.values()), 'tests_after': sum(after.values()),
            'missing': sorted('\t'.join(k) for k in missing), 'extra': sorted('\t'.join(k) for k in extra)}
