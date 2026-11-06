#!/usr/bin/env python3
"""Return the library's declarations to their base-revision visibility.

Usage:
    python scripts/test_topology/src_visibility_revert.py --base <rev> [--root .] [--check]

Moving the in-crate tests out widened library modules, items, methods and fields to `pub` so
the courts could name them. Sealed test doors replace that widening: every library line whose
only change since the base revision is a visibility marker gets its base text back. Lines that
changed in any other way (gate removal, door rewrites, new code) are left alone, and so are
files the base revision does not have. Deterministic and idempotent: a second run changes
nothing. `--check` reports without writing and exits 1 when a line would change.
"""
import argparse
import difflib
from pathlib import Path
import re
import subprocess

LIBRARY = 'workspaces/worth-query/crates/worth-query-execution/src'
VISIBILITY = re.compile(r'\bpub(?:\s*\((?:crate|super|self|in\s+[\w:]+)\))?\s+')


def without_visibility(line):
    return VISIBILITY.sub('', line)


def reverted(base, current):
    """`current` with every visibility-only change against `base` undone, and the count."""
    base_lines, lines = base.split('\n'), current.split('\n')
    out, count = [], 0
    matcher = difflib.SequenceMatcher(None, base_lines, lines, autojunk=False)
    for tag, b1, b2, c1, c2 in matcher.get_opcodes():
        if tag != 'replace':
            out.extend(lines[c1:c2])
            continue
        # Pair replaced lines by their visibility-free text, in order.
        pool = {}
        for index in range(b1, b2):
            pool.setdefault(without_visibility(base_lines[index]), []).append(index)
        for line in lines[c1:c2]:
            candidates = pool.get(without_visibility(line))
            if candidates and base_lines[candidates[0]] != line:
                out.append(base_lines[candidates.pop(0)])
                count += 1
            else:
                if candidates:
                    candidates.pop(0)
                out.append(line)
    return '\n'.join(out), count


def base_text(root, base, path):
    result = subprocess.run(['git', 'show', f'{base}:{path}'], cwd=root, capture_output=True)
    return result.stdout.decode('utf-8') if result.returncode == 0 else None


def run(root, base, check=False):
    files = subprocess.run(['git', 'ls-files', LIBRARY], cwd=root, capture_output=True, text=True,
                           check=True).stdout.split()
    changed = {}
    for path in sorted(f for f in files if f.endswith('.rs')):
        old = base_text(root, base, path)
        if old is None:
            continue
        current = (root / path).read_text(encoding='utf-8')
        new, count = reverted(old, current)
        # Undoing one change can realign the diff and expose another: run to a fixed point.
        while count:
            again, more = reverted(old, new)
            if not more:
                break
            new, count = again, count + more
        if count:
            changed[path] = count
            if not check:
                (root / path).write_text(new, encoding='utf-8', newline='\n')
    return changed


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('--base', required=True)
    parser.add_argument('--root', type=Path, default=Path.cwd())
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args(argv)
    changed = run(args.root.resolve(), args.base, args.check)
    print(f'{sum(changed.values())} declarations in {len(changed)} files')
    return 1 if args.check and changed else 0


if __name__ == '__main__':
    raise SystemExit(main())
