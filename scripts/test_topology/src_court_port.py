"""Port a later tree's library test changes into the court and testkit files the move made.

Usage:
    python scripts/test_topology/src_court_port.py BASE THEIRS_ROOT THEIRS_REV [--report DIR]

BASE is the commit of the codemod's output on the original tree (its court and testkit files
are the merge base). THEIRS_ROOT/THEIRS_REV is a checkout holding the codemod's output for the
later tree, placed with the base plan's placement (`SRC_PINS`), so a file's text differs from
BASE only by the later tree's own changes. For every court or testkit file:
- ours is where that BASE file lives at HEAD (git rename detection BASE -> HEAD, renames limited
  to court and testkit crates); `git merge-file` merges BASE -> THEIRS into it, conflicts
  marked and reported;
- a file new in THEIRS goes to the HEAD crate its BASE crate became (the crate most of its files
  moved to), its tests directory renamed to match;
- a file THEIRS deleted is deleted when ours still equals BASE, and reported otherwise.
"""
import argparse
from collections import Counter, defaultdict
import json
from pathlib import Path
import re
import subprocess
import tempfile

CRATES = 'workspaces/worth-query/crates/'


def family(path):
    name = path[len(CRATES):].split('/')[0] if path.startswith(CRATES) else ''
    return (name.startswith('worth-query-court-') and 'execution' in name) or name.startswith('worth-query-testkit-execution')


def git(*args, cwd=None, text=True):
    return subprocess.run(['git', *args], capture_output=True, text=text, cwd=cwd, check=True).stdout


def tree(rev, cwd=None):
    return {p for p in git('ls-tree', '-r', '--name-only', rev, '--', CRATES, cwd=cwd).split() if family(p)}


def head_paths(base):
    """{base path: head path} for every BASE court/testkit file that still exists at HEAD."""
    out = git('-c', 'diff.renameLimit=50000', 'diff', '--name-status', '-M50%', base, 'HEAD', '--', CRATES)
    mapping = {p: p for p in tree(base)}
    for line in out.splitlines():
        parts = line.split('\t')
        if parts[0].startswith('R') and family(parts[1]):
            mapping[parts[1]] = parts[2]
        elif parts[0] == 'D' and family(parts[1]):
            mapping.pop(parts[1], None)
    return mapping


def crate_map(mapping):
    """{base crate dir: head crate dir} by majority of its files' moves."""
    votes = defaultdict(Counter)
    for old, new in mapping.items():
        votes[old[len(CRATES):].split('/')[0]][new[len(CRATES):].split('/')[0]] += 1
    return {old: counts.most_common(1)[0][0] for old, counts in votes.items()}


def relocated(path, crates):
    old = path[len(CRATES):].split('/')[0]
    new = crates.get(old, old)
    rest = path[len(CRATES) + len(old):]
    return CRATES + new + rest.replace(f'/tests/{old.replace("-", "_")}', f'/tests/{new.replace("-", "_")}', 1)


def merge3(ours, base, theirs):
    with tempfile.TemporaryDirectory() as directory:
        files = []
        for name, text in (('ours', ours), ('base', base), ('theirs', theirs)):
            path = Path(directory) / name
            path.write_bytes(text)
            files.append(str(path))
        result = subprocess.run(['git', 'merge-file', '-p', '-L', 'ours', '-L', 'base', '-L', 'master', *files],
                                capture_output=True)
        return result.stdout, result.returncode


def port(root, base, theirs_root, theirs_rev):
    mapping = head_paths(base)
    crates = crate_map(mapping)
    base_files = tree(base)
    theirs_files = tree(theirs_rev, cwd=theirs_root)
    report = {'merged': [], 'conflicted': [], 'added': [], 'deleted': [], 'kept_changed_deletion': [],
              'unmapped': []}
    for path in sorted(base_files | theirs_files):
        a = git('show', f'{base}:{path}', text=False) if path in base_files else None
        t = git('show', f'{theirs_rev}:{path}', cwd=theirs_root, text=False) if path in theirs_files else None
        if a == t:
            continue
        if a is None:
            dest = root / relocated(path, crates)
            dest.parent.mkdir(parents=True, exist_ok=True)
            dest.write_bytes(t)
            report['added'].append(str(dest.relative_to(root)))
            continue
        ours_path = mapping.get(path)
        if ours_path is None:
            report['unmapped'].append(path)
            continue
        ours = (root / ours_path).read_bytes()
        if t is None:
            if ours == a:
                (root / ours_path).unlink()
                report['deleted'].append(ours_path)
            else:
                report['kept_changed_deletion'].append(ours_path)
            continue
        merged, conflicts = merge3(ours, a, t)
        (root / ours_path).write_bytes(merged)
        report['conflicted' if conflicts else 'merged'].append(ours_path)
    return report


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('base')
    parser.add_argument('theirs_root', type=Path)
    parser.add_argument('theirs_rev')
    parser.add_argument('--report', type=Path)
    parser.add_argument('--root', type=Path, default=Path('.'))
    args = parser.parse_args(argv)
    report = port(args.root.resolve(), args.base, args.theirs_root, args.theirs_rev)
    if args.report:
        args.report.mkdir(parents=True, exist_ok=True)
        (args.report / 'src-court-port.json').write_text(json.dumps(report, indent=1, sort_keys=True) + '\n')
    print(json.dumps({k: len(v) for k, v in report.items()}, sort_keys=True))


if __name__ == '__main__':
    main()
