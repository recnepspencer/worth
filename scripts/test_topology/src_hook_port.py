"""Port the codemod's compiled hooks for a later library tree into the merged library.

Usage:
    python scripts/test_topology/src_hook_port.py LATER_REV CODEMOD_ROOT CODEMOD_REV

The codemod compiles the `#[cfg(test)]` items its moved tests call: in its output for the later
tree (CODEMOD_ROOT at CODEMOD_REV), those gates are gone from the library while LATER_REV still
has them. For every library file, each such removed gate (keyed by the line it gated) is removed
here too, where this tree still gates that same line once. Reports gates it could not place.
"""
import argparse
import difflib
import json
from pathlib import Path
import subprocess

LIBRARY = 'workspaces/worth-query/crates/worth-query-execution/src/'
GATE = '#[cfg(test)]'


def show(rev, path, cwd=None):
    result = subprocess.run(['git', 'show', f'{rev}:{path}'], capture_output=True, text=True, cwd=cwd)
    return result.stdout if result.returncode == 0 else None


def compiled_gates(original, compiled):
    """Stripped lines that `original` gates with #[cfg(test)] and `compiled` no longer does."""
    a, b = original.splitlines(), compiled.splitlines()
    removed = []
    for tag, i1, i2, j1, j2 in difflib.SequenceMatcher(a=a, b=b, autojunk=False).get_opcodes():
        if tag in ('delete', 'replace'):
            for i in range(i1, i2):
                if a[i].strip() == GATE and i + 1 < len(a):
                    removed.append(a[i + 1].strip())
    return removed


def port(root, later, codemod_root, codemod_rev):
    files = subprocess.run(['git', 'ls-tree', '-r', '--name-only', later, '--', LIBRARY],
                           capture_output=True, text=True, check=True).stdout.split()
    applied, unplaced = 0, []
    for path in files:
        if not path.endswith('.rs') or '/court_doors_for_test' in path:
            continue
        original, compiled = show(later, path), show(codemod_rev, path, cwd=codemod_root)
        here = root / path
        if original is None or compiled is None or not here.exists():
            continue
        gated = compiled_gates(original, compiled)
        if not gated:
            continue
        lines = here.read_text(encoding='utf-8').splitlines(keepends=True)
        for target in gated:
            spots = [i for i in range(len(lines) - 1) if lines[i].strip() == GATE and lines[i + 1].strip() == target]
            if len(spots) == 1:
                del lines[spots[0]]
                applied += 1
            elif spots or not any(line.strip() == target for line in lines):
                unplaced.append(f'{path}: {target}')
        here.write_text(''.join(lines), encoding='utf-8')
    return applied, unplaced


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('later')
    parser.add_argument('codemod_root', type=Path)
    parser.add_argument('codemod_rev')
    parser.add_argument('--root', type=Path, default=Path('.'))
    args = parser.parse_args(argv)
    applied, unplaced = port(args.root.resolve(), args.later, args.codemod_root, args.codemod_rev)
    print(json.dumps({'gates_removed': applied, 'unplaced': unplaced}, indent=1))


if __name__ == '__main__':
    main()
