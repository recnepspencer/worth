#!/usr/bin/env python3
"""Deterministically plan or apply a v3 integration-court migration (Python 3.11+)."""
import argparse
import json
from pathlib import Path
import subprocess
import sys

from court_plan import build
from plan_effects import apply
from plan_report import markdown


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest='command', required=True)
    planning = sub.add_parser('plan')
    planning.add_argument('manifest', type=Path)
    planning.add_argument('--out', type=Path, required=True)
    planning.add_argument('--root', type=Path, default=Path.cwd())
    applying = sub.add_parser('apply')
    applying.add_argument('plan', type=Path)
    applying.add_argument('--root', type=Path, default=Path.cwd())
    args = parser.parse_args(argv)
    try:
        if args.command == 'plan':
            plan = build(args.manifest, args.root)
            args.out.mkdir(parents=True, exist_ok=True)
            (args.out / 'court-plan.json').write_text(json.dumps(plan, indent=2, sort_keys=True) + '\n', encoding='utf-8', newline='\n')
            (args.out / 'court-plan.md').write_text(markdown(plan), encoding='utf-8', newline='\n')
            print(json.dumps(plan['totals'], sort_keys=True))
        else:
            plan = json.loads(args.plan.read_text(encoding='utf-8'))
            if plan['version'] != 1:
                raise ValueError('unsupported plan version')
            if plan['blockers']:
                raise ValueError('plan has apply blockers: ' + json.dumps(plan['blockers']))
            print(json.dumps(apply(plan, args.root), sort_keys=True))
    except (ValueError, OSError, subprocess.CalledProcessError) as error:
        print(str(error), file=sys.stderr)
        return 1
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
