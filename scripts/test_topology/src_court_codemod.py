#!/usr/bin/env python3
"""Plan or apply the move of a library's court-shaped source tests into court crates.

Usage:
    python scripts/test_topology/src_court_codemod.py plan --out <dir> [--overrides <json>]
    python scripts/test_topology/src_court_codemod.py apply <dir>/src-court-plan.json

`plan` never changes the tree. Its JSON and Markdown are sorted, carry no timestamps or
absolute paths, and hold every effect: moves with source hashes, writes with before
hashes, the production visibility widenings, compiled hooks and configuration edits.
`apply` checks every precondition first, moves with `git mv`, refuses drift, accepts its
own final tree on replay, and checks the test-name and qualifier-normalized body multisets.
"""
import argparse
from collections import Counter
import json
from pathlib import Path
import subprocess
import sys

from plan_effects import apply
from src_court_effects import effects_for
from src_multiset import multiset_of_files

LIB = 'workspaces/worth-query/crates/worth-query-execution/src/lib.rs'
WORKSPACE = 'workspaces/worth-query/Cargo.toml'
CONFIG = 'tools/boundary-check/config/'


def plan_command(args):
    root = args.root.resolve()
    overrides = json.loads(args.overrides.read_text(encoding='utf-8')) if args.overrides else {}
    plan = effects_for(root, root / LIB, root / (CONFIG + 'src-court-baseline.json'),
                       root / (CONFIG + 'src-court-private-logic-audit.json'), root / WORKSPACE, overrides)
    args.out.mkdir(parents=True, exist_ok=True)
    (args.out / 'src-court-plan.json').write_text(json.dumps(plan, indent=1, sort_keys=True) + '\n',
                                                  encoding='utf-8', newline='\n')
    (args.out / 'src-court-plan.md').write_text(report(plan), encoding='utf-8', newline='\n')
    print(json.dumps(plan['totals'], sort_keys=True))
    return 0


def apply_command(args):
    root = args.root.resolve()
    plan = json.loads(args.plan.read_text(encoding='utf-8'))
    if plan['version'] != 1:
        raise ValueError('unsupported plan version')
    if plan['blockers']:
        raise ValueError('plan has apply blockers: ' + json.dumps(plan['blockers'])[:2000])
    result = apply(plan, root)
    after = multiset_of_files(root, plan['court_test_files'])
    before = Counter({tuple(k.split('\t')): v for k, v in plan['test_multiset'].items()})
    if after != before:
        missing, extra = before - after, after - before
        raise ValueError(f'test multiset changed: missing {sorted(missing)[:20]} extra {sorted(extra)[:20]}')
    result['tests'] = sum(after.values())
    print(json.dumps(result, sort_keys=True))
    return 0


def report(plan):
    t = plan['totals']
    lines = ['# Source court move plan', '', '| Measure | Value |', '|---|---:|']
    lines += [f'| {k} | {v} |' for k, v in sorted(t.items())]
    lines += ['', '## Crates', '', '| Crate | Kind | Frontier | Lines |', '|---|---|---|---:|']
    for row in plan['crates']:
        lines.append(f"| {row['name']} | {row['kind']} | {', '.join(row['frontier'])} | {row['lines']} |")
    for title, key in (('Widened modules', 'widened_modules'), ('Widened items', 'widened_items'),
                       ('Compiled hooks', 'compiled_hooks'), ('Unresolved references', 'unresolved'),
                       ('Category decisions', 'categories'), ('Blockers', 'blockers')):
        lines += ['', f'## {title}', '']
        lines += [f'- `{row}`' if isinstance(row, str) else f'- `{json.dumps(row, sort_keys=True)}`'
                  for row in plan[key]] or ['- none']
    return '\n'.join(lines) + '\n'


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = parser.add_subparsers(dest='command', required=True)
    planning = sub.add_parser('plan')
    planning.add_argument('--out', type=Path, required=True)
    planning.add_argument('--overrides', type=Path)
    planning.add_argument('--root', type=Path, default=Path.cwd())
    applying = sub.add_parser('apply')
    applying.add_argument('plan', type=Path)
    applying.add_argument('--root', type=Path, default=Path.cwd())
    args = parser.parse_args(argv)
    try:
        return plan_command(args) if args.command == 'plan' else apply_command(args)
    except (ValueError, OSError, subprocess.CalledProcessError) as error:
        print(str(error), file=sys.stderr)
        return 1


if __name__ == '__main__':
    raise SystemExit(main())
