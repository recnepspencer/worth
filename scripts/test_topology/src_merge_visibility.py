"""Resolve library merge conflicts whose only change on our side is the door work's shape.

Usage:
    python scripts/test_topology/src_merge_visibility.py FILE...

Each FILE holds diff3 conflict hunks (ours / base / theirs). The door work changes library
source in two ways that carry no behaviour: an item's visibility (widened for a door
signature) and a removed `#[cfg(test)]` (a hook compiled for the courts). When ours differs
from base only by those, the hunk resolves to theirs with the same changes replayed on the same
items (matched by kind and name). Any other difference in ours leaves the hunk unresolved;
the file is reported. Two door-work shapes resolve too: a `#[cfg(test)]` module the move took
to a court (ours empty or only the door mount) keeps ours, since master's edit to it is ported
into the court copy; and a door mount ours appended is kept after theirs.
"""
import re
import sys
from pathlib import Path

from rust_source import tokens

DECL = re.compile(r'^(\s*)(pub(?:\([^)]*\))?\s+)?((?:(?:const|async|unsafe)\s+)*(fn|struct|enum|trait|type|mod|const|static|union|use)\s+([\w:{}*, ]+?))(\b|\s|<|\(|;|:|\{)')
HUNK = re.compile(r'^<<<<<<< [^\n]*\n(.*?)^\|\|\|\|\|\|\| [^\n]*\n(.*?)^=======\n(.*?)^>>>>>>> [^\n]*\n', re.M | re.S)


def key(line):
    match = DECL.match(line)
    if not match:
        return None
    return match.group(4), match.group(5).strip(), match.group(2) or ''


def core_tokens(text):
    """Token values without visibilities and `#[cfg(test)]` attributes: formatting-blind."""
    values = [t.value for t in tokens(text)]
    out, k = [], 0
    while k < len(values):
        if values[k:k + 7] == ['#', '[', 'cfg', '(', 'test', ')', ']']:
            k += 7
            continue
        if values[k] == 'pub':
            k += 1
            if k < len(values) and values[k] == '(':
                depth = 0
                while k < len(values):
                    depth += {'(': 1, ')': -1}.get(values[k], 0)
                    k += 1
                    if depth == 0:
                        break
            continue
        out.append(values[k])
        k += 1
    return [v for i, v in enumerate(out) if not (v == ',' and i + 1 < len(out) and out[i + 1] in (')', ']', '}', '>'))]


MOUNT = ['mod', 'court_doors_for_test', ';']


def moved_test_module(text):
    """Whether `text` is exactly one `#[cfg(test)] mod NAME { .. }` block (blank lines aside)."""
    values = [t.value for t in tokens(text)]
    if values[:9] != ['#', '[', 'cfg', '(', 'test', ')', ']', 'mod', values[8] if len(values) > 8 else ''] \
            or len(values) < 10 or values[9] != '{':
        return False
    depth = 0
    for i, value in enumerate(values[9:], 9):
        depth += {'{': 1, '}': -1}.get(value, 0)
        if depth == 0:
            return i == len(values) - 1
    return False


def shape_changes(ours, base):
    """({(kind, name): visibility}, {(kind, name) whose cfg(test) ours removed}) or None when ours
    differs from base by anything else."""
    o, b = ours.splitlines(), base.splitlines()
    if core_tokens(ours) != core_tokens(base):
        return None
    visibility, uncfg = {}, set()
    for line in o:
        k = key(line)
        if k:
            visibility[k[:2]] = k[2]
    for i, line in enumerate(b):
        if line.strip() == '#[cfg(test)]':
            following = next((key(x) for x in b[i + 1:] if key(x) or not x.strip().startswith('#')), None)
            if following and following[:2] in visibility:
                uncfg.add(following[:2])
    return visibility, uncfg


def replay(theirs, visibility, uncfg):
    out = []
    for line in theirs.splitlines(keepends=True):
        k = key(line)
        if k and k[:2] in uncfg and out and out[-1].strip() == '#[cfg(test)]':
            out.pop()
        if k and k[:2] in visibility and visibility[k[:2]] != k[2]:
            match = DECL.match(line)
            line = match.group(1) + visibility[k[:2]] + line[match.end(2) if match.group(2) else len(match.group(1)):]
        out.append(line)
    return ''.join(out)


def resolve(path):
    text = Path(path).read_text(encoding='utf-8')
    unresolved = 0

    def one(match):
        nonlocal unresolved
        ours, base, theirs = match.groups()
        if core_tokens(ours) in ([], MOUNT) and moved_test_module(base) and moved_test_module(theirs):
            return ours  # the module moved to a court; master's edit to it is ported there
        if core_tokens(ours) == core_tokens(base) + MOUNT:
            mount = ''.join(line for line in ours.splitlines(keepends=True)
                            if line.startswith('#[path') or line.strip() == 'mod court_doors_for_test;')
            body = ours[:ours.rfind(mount.splitlines(keepends=True)[0])] if mount else ours
            changes = shape_changes(body, base)
            if changes is not None:
                return replay(theirs, *changes).rstrip('\n') + '\n\n' + mount
        changes = shape_changes(ours, base)
        if changes is None:
            unresolved += 1
            return match.group(0)
        return replay(theirs, *changes)
    new = HUNK.sub(one, text)
    Path(path).write_text(new, encoding='utf-8')
    return unresolved


def main(argv=None):
    left = {}
    for path in (argv or sys.argv[1:]):
        count = resolve(path)
        if count:
            left[path] = count
    for path, count in left.items():
        print(f'{count} unresolved: {path}')
    print(f'{len(left)} files with unresolved hunks')


if __name__ == '__main__':
    main()
