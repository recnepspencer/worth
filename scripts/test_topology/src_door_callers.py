"""Exact callers of worth-query-execution's sealed test doors, for boundary-check's seal rule.

The seal rule lets a `*_for_test` / `*ForTest` name be referenced only by its owner and by
the files `tools/boundary-check/config/control-callers.txt` lists for that owner. Every court
or testkit file naming a door the library declares (the `court_doors_for_test` module, its
member doors, and the library's existing test doors) gets an entry with the library as owner.
An entry whose caller file no longer exists and lay in this crate family (the library or one
of its courts and testkits) is moved to the file's new home when exactly one moved file has the
same module path, and dropped otherwise. Every other entry is left as it is.
"""
from pathlib import Path
import re

from rust_source import tokens

CONFIG = 'tools/boundary-check/config/control-callers.txt'
OWNER = 'workspaces/worth-query/crates/worth-query-execution/'
CRATES = 'workspaces/worth-query/crates'
TEST_NAME = re.compile(r'^\w+(?:_for_test|ForTest)$')
DECLARING = ('fn', 'struct', 'enum', 'trait', 'type', 'const', 'static', 'mod', 'union', 'macro_rules')


def declared_doors(root):
    names = set()
    for path in sorted((root / OWNER / 'src').rglob('*.rs')):
        stream = tokens(path.read_text(encoding='utf-8'))
        for i in range(len(stream) - 1):
            if stream[i].value in DECLARING and TEST_NAME.match(stream[i + 1].value):
                names.add(stream[i + 1].value)
            if stream[i].value == 'macro_rules' and i + 2 < len(stream) and TEST_NAME.match(stream[i + 2].value):
                names.add(stream[i + 2].value)
    return names


def moved_files(root):
    crates = root / CRATES
    return sorted(f for d in sorted(crates.iterdir())
                  if d.name.startswith(('worth-query-court-', 'worth-query-testkit-')) and 'execution' in d.name
                  for f in d.rglob('*.rs'))


def in_family(path):
    name = path.split('/')[3] if path.startswith(CRATES + '/') and path.count('/') > 3 else ''
    return name == 'worth-query-execution' or (
        name.startswith(('worth-query-court-', 'worth-query-testkit-')) and 'execution' in name)


def module_key(path):
    tail = path.split('/domain_computation/', 1)
    return 'domain_computation/' + tail[1] if len(tail) == 2 else None


def updated(root, lines):
    """New control-callers lines: stale callers moved or dropped, door callers added."""
    doors = declared_doors(root)
    files = moved_files(root)
    by_key = {}
    for f in files:
        rel = str(f.relative_to(root))
        by_key.setdefault(module_key(rel), []).append(rel)
    out, present = [], set()
    for line in lines:
        parts = line.split()
        if len(parts) != 2 or line.startswith('#') or (root / parts[1]).exists() or not in_family(parts[1]):
            out.append(line)
            present.add(tuple(parts))
            continue
        candidates = by_key.get(module_key(parts[1]), [])
        if len(candidates) == 1:
            entry = (parts[0], candidates[0])
            if entry not in present:
                out.append(' '.join(entry))
                present.add(entry)
    added = []
    for f in files:
        rel = str(f.relative_to(root))
        names = {t.value for t in tokens(f.read_text(encoding='utf-8')) if t.kind == 'ident'}
        if names & doors and (OWNER, rel) not in present:
            added.append(f'{OWNER} {rel}')
            present.add((OWNER, rel))
    if added:
        out.append('# Moved worth-query-execution courts and testkits reach the library only through its sealed doors.')
        out.extend(sorted(added))
    return out


def update_callers(root):
    path = root / CONFIG
    lines = path.read_text(encoding='utf-8').splitlines()
    new = updated(root, lines)
    path.write_text('\n'.join(new) + '\n', encoding='utf-8', newline='\n')
    return len(new) - len(lines)
