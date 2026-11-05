#!/usr/bin/env python3
"""Deterministic structural measures for the worth-query-execution split (law 22).

Usage:
    python scripts/query_split/structural_measures.py --out <dir>

Reads `cargo metadata --offline` for every workspace that reaches the crate and the source
tree; never builds or times anything. Records, per workspace, the crate's edit reach (the
packages that recompile when it changes), fan-in x size, and the Query workspace's test
targets plus court and testkit lines against the court ceiling.
"""
import argparse
import json
from pathlib import Path
import subprocess
import sys

sys.path.insert(0, str(Path(__file__).resolve().parent))
sys.path.insert(0, str(Path(__file__).resolve().parent.parent / 'test_topology'))

from court_partition import CEILING  # noqa: E402
from churn_replay import CRATE, LIB  # noqa: E402
from module_graph import LibraryModel  # noqa: E402

PACKAGE = 'worth-query-execution'
WORKSPACES = ('Cargo.toml', 'workspaces/worth-query/Cargo.toml', 'workspaces/worth-query-bank-world/Cargo.toml',
              'workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/Cargo.toml')


def metadata(manifest):
    command = ['cargo', 'metadata', '--offline', '--format-version', '1', '--manifest-path', str(manifest)]
    return json.loads(subprocess.run(command, check=True, stdout=subprocess.PIPE, text=True,
                                     encoding='utf-8').stdout)


def rust_lines(directory):
    return sum(p.read_text(encoding='utf-8').count('\n') for p in sorted(Path(directory).rglob('*.rs'))
               if 'target' not in p.parts)


def edit_reach(data, package):
    """Packages whose compiled targets include `package`: normal/build closure, plus dev users."""
    names = {p['id']: p['name'] for p in data['packages']}
    nodes = data['resolve']['nodes']
    normal, dev = {}, {}
    for node in nodes:
        for dep in node['deps']:
            kinds = {k['kind'] for k in dep['dep_kinds']}
            if kinds & {None, 'build'}:
                normal.setdefault(dep['pkg'], set()).add(node['id'])
            if 'dev' in kinds:
                dev.setdefault(dep['pkg'], set()).add(node['id'])
    start = [i for i, n in names.items() if n == package]
    closure, stack = set(start), list(start)
    while stack:
        for user in normal.get(stack.pop(), ()):
            if user not in closure:
                closure.add(user)
                stack.append(user)
    tests = {user for member in closure for user in dev.get(member, ())} - closure
    local = {i for i in closure | tests if next(p for p in data['packages'] if p['id'] == i)['source'] is None}
    return sorted(names[i] for i in local if names[i] != package)


def test_targets(data):
    members = set(data['workspace_members'])
    rows = []
    for package in data['packages']:
        if package['id'] not in members:
            continue
        for target in package['targets']:
            kinds = set(target['kind'])
            testable = target.get('test', True) and (kinds & {'lib', 'test', 'bin', 'proc-macro'})
            if testable:
                rows.append(package['name'] + '::' + target['name'] + ('' if 'test' in kinds else ' (' + '/'.join(sorted(kinds)) + ')'))
    return sorted(rows)


def measures(root):
    model = LibraryModel(root / LIB, root)
    production = sum(model.production_lines(names) for names in model.views)
    family_lines = sum(f.lines for f in model.families)
    result = {'version': 1, 'package': PACKAGE,
              'production_lines': production, 'in_crate_test_family_lines': family_lines,
              'in_crate_test_families': len(model.families),
              'in_crate_tests': sum(len(f.tests) for f in model.families),
              'source_lines': rust_lines(root / CRATE / 'src'), 'workspaces': {}}
    for manifest in WORKSPACES:
        data = metadata(root / manifest)
        reach = edit_reach(data, PACKAGE)
        package = next(p for p in data['packages'] if p['name'] == PACKAGE)
        lib = next(t for t in package['targets'] if 'lib' in t['kind'])
        result['workspaces'][manifest] = {'edit_reach': len(reach), 'edit_reach_packages': reach,
                                          'fan_in_x_production_lines': len(reach) * production,
                                          'library_has_test_target': lib.get('test', True)}
    query = metadata(root / WORKSPACES[1])
    targets = test_targets(query)
    crates = {}
    for package in query['packages']:
        if package['id'] in set(query['workspace_members']) and ('-court-' in package['name'] or '-testkit' in package['name']):
            lines = rust_lines(Path(package['manifest_path']).parent)
            crates[package['name']] = {'lines': lines, 'over_ceiling': lines > CEILING}
    result['query_workspace'] = {'test_targets': len(targets), 'test_target_list': targets,
                                 'court_crates': sum(1 for n in crates if '-court-' in n),
                                 'testkit_crates': sum(1 for n in crates if '-testkit' in n),
                                 'ceiling': CEILING, 'court_and_testkit_lines': dict(sorted(crates.items()))}
    return result


def markdown(result):
    lines = ['# Structural measures: ' + result['package'], '',
             f"- production lines (non-test, own text): {result['production_lines']}",
             f"- in-crate test families: {result['in_crate_test_families']} "
             f"({result['in_crate_tests']} tests, {result['in_crate_test_family_lines']} lines)",
             f"- Rust source lines under src/: {result['source_lines']}", '',
             '| Workspace | Edit reach (packages) | Fan-in x production lines | Library test target |',
             '|---|---:|---:|---|']
    for manifest, row in result['workspaces'].items():
        lines.append(f"| `{manifest}` | {row['edit_reach']} | {row['fan_in_x_production_lines']} | "
                     f"{'yes' if row['library_has_test_target'] else 'no'} |")
    query = result['query_workspace']
    lines += ['', f"Query workspace: {query['test_targets']} test targets, {query['court_crates']} court crates, "
              f"{query['testkit_crates']} testkit crates (ceiling {query['ceiling']} lines).", '',
              '| Court or testkit | Lines | Over ceiling |', '|---|---:|---|']
    for name, row in query['court_and_testkit_lines'].items():
        lines.append(f"| {name} | {row['lines']} | {'yes' if row['over_ceiling'] else ''} |")
    return '\n'.join(lines) + '\n'


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--root', type=Path, default=Path.cwd())
    args = parser.parse_args(argv)
    result = measures(args.root.resolve())
    args.out.mkdir(parents=True, exist_ok=True)
    (args.out / 'structural-measures.json').write_text(json.dumps(result, indent=1, sort_keys=True) + '\n',
                                                       encoding='utf-8', newline='\n')
    (args.out / 'structural-measures.md').write_text(markdown(result), encoding='utf-8', newline='\n')
    print(json.dumps({k: result[k] for k in ('production_lines', 'in_crate_test_family_lines', 'in_crate_tests')}))
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
