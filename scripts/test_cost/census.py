#!/usr/bin/env python3
"""Workspace-generic test cost census. Timings are telemetry and never gate."""
import argparse
from collections import Counter
import json
from pathlib import Path
import sys

from coverage import prove
from evidence import run
from logs import llvm_lines, read_lane
from source_scan import scan_workspace
from telemetry import setup_counts, slot_use, timing_summary
from workspace import WORKSPACES, repository_root


def named_path(value):
    name, separator, path = value.partition('=')
    if not separator or not name or not path:
        raise argparse.ArgumentTypeError('expected NAME=PATH')
    return name, Path(path)


def collect(manifest, repository, lane_logs=(), discovered=None, selections=None, llvm=()):
    repository, manifest = repository.resolve(), manifest.resolve()
    sites = scan_workspace(manifest, repository)
    lanes = {}
    for name, path in lane_logs:
        if name in lanes:
            raise ValueError(f'duplicate lane name: {name}')
        lane = read_lane(path)
        lanes[name] = {'log': str(path), 'headers': lane['headers'],
                       'build_observation': lane['build_observation'],
                       'artifacts': lane['artifacts'], 'duplicate_graphs': lane['duplicates'],
                       'compiler_progress': lane['compiler_progress'],
                       'timing_telemetry': timing_summary(lane),
                       'slot_use_telemetry': slot_use(lane['cpu']),
                       'repeated_setup': setup_counts(lane['setup'])}
    coverage = {'status': 'unmeasured', 'reason': 'no complete discovered inventory supplied'}
    if discovered is not None:
        coverage = prove(discovered, selections or {})
    return {'schema': 1, 'manifest': str(manifest.relative_to(repository)),
            'source_sites': sites, 'source_counts': dict(sorted(Counter(s['kind'] for s in sites).items())),
            'lanes': lanes, 'coverage': coverage,
            'llvm_lines': {name: llvm_lines(path.read_text(encoding='utf-8')) for name, path in llvm}}


def markdown(result):
    lines = [f'# Test cost: {result["manifest"]}', '',
             'Structural observations gate; CPU and wall times are telemetry. '
             'Unmeasured means unknown, never zero.', '', '## Source census', '',
             '| Site | Count |', '|---|---:|']
    lines += [f'| {kind} | {count} |' for kind, count in result['source_counts'].items()]
    lines += ['', '## Lanes', '', '| Lane | Build census | Duplicates | Pass / fail / ignored | Wall seconds |',
              '|---|---|---:|---|---:|']
    for name, lane in result['lanes'].items():
        timing = lane['timing_telemetry']
        duplicate_count = len(lane['duplicate_graphs']) if lane['build_observation'] == 'json' else 'unknown'
        lines.append(f'| {name} | {lane["build_observation"]} | {duplicate_count} | '
                     f'{timing["pass"]} / {timing["fail"]} / {timing["ignored"]} | '
                     f'{timing.get("wall_seconds", "unknown")} |')
        lines += ['', f'### {name}', '', f'Log: `{lane["log"]}`', '',
                  'Slowest observed tests (summed work is not lane wall time):', '']
        lines += [f'- {r["seconds"]:.3f}s: `{r["name"]}` ({r["event"]})' for r in timing['slowest']]
    lines += ['', '## Lane coverage', '', f'Status: **{result["coverage"]["status"]}**.', '',
              'Ignored tests are scheduled-only; this tool never executes them.', '',
              '## Source locations', '']
    lines += [f'- `{s["path"]}:{s["line"]}`: {s["kind"]} ({s["function"]})'
              for s in result['source_sites']]
    return '\n'.join(lines) + '\n'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    scope = parser.add_mutually_exclusive_group(required=True)
    scope.add_argument('--manifest', type=Path)
    scope.add_argument('--all', action='store_true')
    parser.add_argument('--workspace', help='output directory name, inferred for governed workspaces')
    parser.add_argument('--log', type=named_path, action='append', default=[])
    parser.add_argument('--inventory', type=Path)
    parser.add_argument('--selection', type=named_path, action='append', default=[])
    parser.add_argument('--llvm', type=named_path, action='append', default=[])
    parser.add_argument('--output', type=Path)
    parser.add_argument('command', nargs=argparse.REMAINDER)
    args = parser.parse_args()
    repository = repository_root(Path.cwd())
    manifests = WORKSPACES.items() if args.all else [(args.workspace or next(
        (name for name, path in WORKSPACES.items() if (repository / path).resolve() == args.manifest.resolve()),
        args.manifest.resolve().parent.name), args.manifest)]
    if args.all and (args.log or args.inventory or args.selection or args.llvm or args.command):
        parser.error('--all is a source census; supply measured inputs separately per workspace')
    for name, path in manifests:
        manifest = (repository / path).resolve()
        destination = args.output or repository / 'target/test-cost' / name
        destination.mkdir(parents=True, exist_ok=True)
        command = args.command[1:] if args.command[:1] == ['--'] else args.command
        logs = list(args.log)
        if command:
            if any(flag in command for flag in ('--include-ignored', '--ignored', 'query-scale')) or any(
                    a == '--run-ignored' and command[i + 1:i + 2] != ['default']
                    for i, a in enumerate(command)):
                parser.error('ignored tests are scheduled-only and cannot be measured here')
            if command[0] == 'cargo':
                if command[1:2] in (['test'], ['check'], ['build']):
                    command.insert(2, '--message-format=json')
                elif command[1:2] == ['nextest']:
                    command.insert(3, '--cargo-message-format=json')
            path = destination / 'lane.log'
            code = run(command, path)
            logs.append(('command', path))
        else:
            code = 0
        result = collect(manifest, repository, logs,
                         json.loads(args.inventory.read_text()) if args.inventory else None,
                         {lane: json.loads(p.read_text()) for lane, p in args.selection}, args.llvm)
        (destination / 'census.json').write_text(json.dumps(result, indent=2, sort_keys=True) + '\n', encoding='utf-8')
        (destination / 'census.md').write_text(markdown(result), encoding='utf-8')
        print(f'{name}: {result["source_counts"]}; coverage={result["coverage"]["status"]}; {destination}')
        if code:
            return code
    return 0


if __name__ == '__main__':
    sys.exit(main())
