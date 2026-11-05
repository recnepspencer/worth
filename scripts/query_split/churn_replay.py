#!/usr/bin/env python3
"""Churn replay: how many production lines a compiler cache could reuse per historical edit.

Usage:
    python scripts/query_split/churn_replay.py commits --ref master --count 400 \
        --threshold 150 --out <commits.txt>
    python scripts/query_split/churn_replay.py score <commits.txt> <candidate.json>... --out <dir>

`commits` lists, newest first, the last non-merge commits on `--ref` that touch the crate,
skipping mass edits that change more than `--threshold` files. Each line is the commit and
its changed crate paths, so `score` never reads history and is deterministic for a list.

`score` maps each commit's files onto a candidate partition of the current library into
crates, takes the reverse-dependency closure of the touched crates in the partition's
crate graph, and counts the production lines outside it: the crates a cache could serve.
"""
import argparse
import json
from pathlib import Path
import statistics
import subprocess
import sys

sys.path.insert(0, str(Path(__file__).resolve().parent))

from crate_partition import Partition, file_modules  # noqa: E402
from module_graph import LibraryModel, flattened_nodes, node_graph  # noqa: E402

CRATE = 'workspaces/worth-query/crates/worth-query-execution'
LIB = CRATE + '/src/lib.rs'
EXPANDED = (('domain_computation',), ('domain_computation', 'primary_graph'))


def git(*args):
    return subprocess.run(['git', *args], check=True, stdout=subprocess.PIPE, text=True,
                          encoding='utf-8').stdout


def list_commits(ref, count, threshold, crate):
    rows, skipped = [], []
    for commit in git('log', '--no-merges', '--format=%H', ref, '--', crate).split():
        changed = [p for p in git('diff-tree', '--no-commit-id', '--name-only', '-r', '--root', commit).splitlines() if p]
        if len(changed) > threshold:
            skipped.append(commit)
            continue
        rows.append((commit, sorted(p for p in changed if p.startswith(crate + '/'))))
        if len(rows) == count:
            break
    return rows, skipped


def read_commits(path):
    rows = []
    for line in Path(path).read_text(encoding='utf-8').splitlines():
        if line and not line.startswith('#'):
            commit, *files = line.split('\t')
            rows.append((commit, files))
    return rows


def score(model, commits, candidate, nodes, edges, lines):
    partition = Partition(candidate, nodes, edges, lines)
    mapping = file_modules(model, CRATE)
    per_commit, unmapped = [], set()
    for commit, files in commits:
        touched = set()
        for path in files:
            crates = partition.crates_for_file(path, mapping, CRATE)
            if crates is None:
                unmapped.add(path)
                crates = partition.crates_for_unknown()
            touched |= crates
        closure = partition.reverse_closure(touched)
        reusable = sum(partition.lines[c] for c in partition.names if c not in closure)
        per_commit.append({'commit': commit, 'touched': sorted(touched), 'recompiled': len(closure),
                           'reusable_crates': len(partition.names) - len(closure),
                           'reusable_lines': reusable})
    reusable_lines = [r['reusable_lines'] for r in per_commit]
    reusable_crates = [r['reusable_crates'] for r in per_commit]
    return {'name': candidate['name'], 'description': candidate.get('description', ''),
            'valid': not partition.cycles and not partition.hypothetical, 'cycles': partition.cycles,
            'hypothetical': partition.hypothetical, 'references_to_break': partition.broken_references,
            'crates': partition.summary(),
            'commits': len(per_commit), 'unmapped_paths': len(unmapped),
            'mean_reusable_lines': round(statistics.fmean(reusable_lines), 1) if per_commit else 0,
            'median_reusable_lines': statistics.median(reusable_lines) if per_commit else 0,
            'mean_reusable_crates': round(statistics.fmean(reusable_crates), 2) if per_commit else 0,
            'median_reusable_crates': statistics.median(reusable_crates) if per_commit else 0,
            'mean_recompiled_crates': round(statistics.fmean(r['recompiled'] for r in per_commit), 2) if per_commit else 0,
            'parallel_width': partition.parallel_width(),
            'per_commit': per_commit}


def markdown(results, commit_count, threshold):
    lines = ['# Churn replay', '', f'Commits: {commit_count} (mass-edit threshold: more than {threshold} files).', '',
             '| Candidate | Valid | Crates | Mean reusable lines | Median | Mean reusable crates | Mean recompiled crates | Parallel width |',
             '|---|---|---:|---:|---:|---:|---:|---:|']
    for r in results:
        validity = 'hypothetical' if r['hypothetical'] else 'yes' if r['valid'] else 'no: cycle'
        lines.append(f"| {r['name']} | {validity} | {len(r['crates'])} | "
                     f"{r['mean_reusable_lines']} | {r['median_reusable_lines']} | {r['mean_reusable_crates']} | "
                     f"{r['mean_recompiled_crates']} | {r['parallel_width']} |")
    for r in results:
        lines += ['', f"## {r['name']}", '', r['description'], '']
        if r['hypothetical']:
            lines += [f"Hypothetical: assumes {r['references_to_break']} upward references are removed first.", '']
        lines += [
                  '| Crate | Production lines | Transitive dependents | Fan-in x size | Touched by commits |',
                  '|---|---:|---:|---:|---:|']
        for c in r['crates']:
            touches = sum(1 for row in r['per_commit'] if c['name'] in row['touched'])
            lines.append(f"| {c['name']} | {c['lines']} | {c['dependents']} | {c['fan_in_size']} | {touches} |")
        for cycle in r['cycles']:
            lines.append(f"\nCycle: {' -> '.join(cycle)}")
    return '\n'.join(lines) + '\n'


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = parser.add_subparsers(dest='command', required=True)
    listing = sub.add_parser('commits')
    listing.add_argument('--ref', default='master')
    listing.add_argument('--count', type=int, default=400)
    listing.add_argument('--threshold', type=int, default=150)
    listing.add_argument('--out', type=Path, required=True)
    scoring = sub.add_parser('score')
    scoring.add_argument('commits', type=Path)
    scoring.add_argument('candidates', type=Path, nargs='+')
    scoring.add_argument('--out', type=Path, required=True)
    scoring.add_argument('--root', type=Path, default=Path.cwd())
    args = parser.parse_args(argv)
    if args.command == 'commits':
        rows, skipped = list_commits(args.ref, args.count, args.threshold, CRATE)
        header = [f'# churn replay commits: ref={args.ref} count={len(rows)} threshold={args.threshold} '
                  f'skipped_mass_edits={len(skipped)}', '# commit<TAB>changed crate paths...']
        body = ['\t'.join([c] + f) for c, f in rows]
        args.out.parent.mkdir(parents=True, exist_ok=True)
        args.out.write_text('\n'.join(header + body) + '\n', encoding='utf-8', newline='\n')
        print(json.dumps({'commits': len(rows), 'skipped_mass_edits': len(skipped)}))
        return 0
    commits = read_commits(args.commits)
    threshold = next((part.split('=')[1] for part in args.commits.read_text(encoding='utf-8').split()
                      if part.startswith('threshold=')), '?')
    model = LibraryModel(args.root / LIB, args.root)
    nodes = flattened_nodes(model, EXPANDED)
    edges, _kinds, lines = node_graph(model, nodes)
    results = [score(model, commits, json.loads(c.read_text(encoding='utf-8')), nodes, edges, lines)
               for c in args.candidates]
    args.out.mkdir(parents=True, exist_ok=True)
    (args.out / 'churn-replay.json').write_text(json.dumps(results, indent=1, sort_keys=True) + '\n',
                                                encoding='utf-8', newline='\n')
    (args.out / 'churn-replay.md').write_text(markdown(results, len(commits), threshold),
                                              encoding='utf-8', newline='\n')
    print(json.dumps([{k: r[k] for k in ('name', 'valid', 'mean_reusable_lines', 'median_reusable_lines',
                                         'mean_reusable_crates')} for r in results]))
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
