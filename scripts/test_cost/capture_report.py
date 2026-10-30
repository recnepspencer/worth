"""Recover an initial run report from retained captures without rerunning Rust."""
import argparse
from collections import Counter
from datetime import datetime
import hashlib
import json
from pathlib import Path
import sys

from cargo_capture import CargoRunner, retention_name
from census import collect, markdown
from complete_coverage import merge
from coverage import inventory, prove as prove_selection
from executed_coverage import execution_kind, observations, prove
from logs import read_lane
from measured_slots import load as load_slots, retain as retain_slots
from ratchet import build_violations, compiler_observations, lane_budget
from residue_report import generate as generate_residue, markdown as residue_markdown
from workspace import WORKSPACES


def retained_logs(repository, source):
    locals = [path for directory in ('build', 'prebuilt') for path in (source / directory).rglob('*.log')]
    locals += list(source.glob('run-*.log')) + [source / 'documentation/doctests.log']
    retained, provenance = [], []
    for path in sorted(locals):
        lane = read_lane(path)
        headers = lane['headers']
        if any(headers[key] is None for key in ('Command', 'HEAD', 'Git status --short empty', 'Start', 'End', 'Exit code')):
            raise ValueError('incomplete original command evidence: ' + str(path))
        name = retention_name(path.relative_to(repository / 'target'), headers['HEAD'], headers['Start'])
        canonical = repository / 'target/lane-evidence' / name
        if path.read_bytes() != canonical.read_bytes():
            raise ValueError('retained capture differs from original: ' + str(path))
        retained.append(canonical)
        provenance.append({'working_log': str(path), 'retained_log': str(canonical), 'headers': headers,
                           'sha256': hashlib.sha256(canonical.read_bytes()).hexdigest()})
    if len(set(retained)) != len(retained):
        raise ValueError('initial capture identity repeats')
    return retained, provenance


def captured_selections(path, native):
    path = Path(path).resolve()
    value = json.loads(path.read_text(encoding='utf-8'))
    known = {(row['binary'], row['test']): row for row in inventory(native)}
    chosen, seen, ignored_seen = {}, set(), set()
    for suite, observation in value['suites'].items():
        rows = []
        for ignored, field, visited in ((False, 'tests', seen), (True, 'ignored', ignored_seen)):
            for identity in observation[field]:
                if not isinstance(identity, list) or len(identity) != 2 or not all(isinstance(item, str) for item in identity):
                    raise ValueError('captured gate selection has an invalid identity')
                key = tuple(identity)
                if key not in known or known[key]['ignored'] != ignored:
                    raise ValueError('captured gate selection contradicts actual inventory: ' + repr(key))
                if key in visited:
                    raise ValueError('captured gate selection repeats an identity: ' + repr(key))
                visited.add(key)
                if not ignored:
                    rows.append(dict(known[key], selected=True))
        chosen[suite] = rows
    proof = prove_selection(native, chosen)
    if value['coverage']['status'] == 'pass' and proof['status'] != 'pass':
        raise ValueError('captured complete gate selection leaves actual native claims missing')
    if value['coverage']['selected'] != len(seen) or value['coverage']['ignored'] != len(ignored_seen):
        raise ValueError('captured gate selection counts contradict its identity lists')
    return chosen, {'kind': 'captured-gate-selection', 'path': str(path),
                    'sha256': hashlib.sha256(path.read_bytes()).hexdigest(), 'proof': proof}


def original_parent(path):
    if path is None:
        return None
    path = Path(path).resolve()
    observed = read_lane(path)
    if any(observed['headers'][field] is None for field in
           ('Command', 'HEAD', 'Git status --short empty', 'Start', 'End', 'Exit code')):
        raise ValueError('incomplete original parent evidence: ' + str(path))
    return {'path': str(path), 'headers': observed['headers'],
            'sha256': hashlib.sha256(path.read_bytes()).hexdigest(),
            'interpretation': 'Original outer command outcome is retained; offline recovery does not relabel it.'}


def telemetry_capture(path, cpu, setup):
    path = Path(path).resolve()
    observed = read_lane(path)
    headers = observed['headers']
    if any(headers[field] is None for field in
           ('Command', 'HEAD', 'Git status --short empty', 'Start', 'End', 'Exit code')):
        raise ValueError('incomplete recovered telemetry capture: ' + str(path))
    command = json.loads(headers['Command'])
    if len(command) < 2 or Path(command[1]).name != 'phase_telemetry.py' or headers['Exit code'] != '0':
        raise ValueError('not a successful captured telemetry aggregation')
    for flag, expected in (('--cpu', cpu), ('--setup', setup)):
        if flag not in command or Path(command[command.index(flag) + 1]).resolve() != Path(expected).resolve():
            raise ValueError('reused telemetry belongs to other process observation inputs')
    if any(flag not in command or command[command.index(flag) + 1] != 'nan'
           for flag in ('--build-seconds', '--wall-seconds')):
        raise ValueError('recovery cannot reuse fabricated phase clocks')
    return path, observed


def recover(repository, workspace, source, output, cpu, setup, runner, profile='dev',
            selection_manifest=None, outer_log=None, retained_telemetry=None):
    repository, source, output = (Path(value).resolve() for value in (repository, source, output))
    if source == output:
        raise ValueError('initial snapshot must not overwrite the current census')
    output.mkdir(parents=True, exist_ok=True)
    paths, provenance = retained_logs(repository, source)
    native = json.loads((source / 'build/inventory.json').read_text())
    if selection_manifest is not None:
        selections, selection_source = captured_selections(selection_manifest, native)
    else:
        selections = {path.stem.removeprefix('selection-'): json.loads(path.read_text())
                      for path in sorted((source / 'build').glob('selection-*.json'))}
        selection_source = {'kind': 'generic-partitions'}
    parent = original_parent(outer_log)
    documentation = json.loads((source / 'documentation/result.json').read_text())
    complete, chosen, structural = merge(native, selections, documentation)
    events, walls = observations(paths, native)
    executed = prove(complete, chosen, events, require_full_workspace=True)
    executed['logs'] = walls
    (output / 'executed-coverage.json').write_text(json.dumps(executed, indent=2) + '\n', encoding='utf-8')
    # The interrupted parent lost its monotonic clocks. Do not fabricate phase
    # values; the aggregate command records their absence as an observation error.
    if retained_telemetry is not None:
        telemetry_path, telemetry = telemetry_capture(retained_telemetry, cpu, setup)
    else:
        command = [sys.executable, str(Path(__file__).with_name('phase_telemetry.py').resolve()),
                   '--cpu', str(Path(cpu).resolve()), '--setup', str(Path(setup).resolve()),
                   '--build-seconds', 'nan', '--wall-seconds', 'nan']
        telemetry_log = output / 'phase-telemetry.log'
        observed = runner(command, repository, telemetry_log)
        if observed.returncode:
            raise ValueError('telemetry recovery command failed: ' + str(observed.returncode))
        telemetry_path = Path(runner.logs[-1])
        telemetry = read_lane(telemetry_path)
    baseline = json.loads((source / 'reservations.json').read_text())
    known = {row['binary'] + '\n' + row['test'] for row in complete}
    native_runs = [row for row in provenance if Path(row['working_log']).name.startswith('run-')]
    head = min(native_runs, key=lambda row: row['headers']['Start'])['headers']['HEAD']
    policy = retain_slots(output / 'measured-slots', telemetry['cpu'], head, str(telemetry_path), baseline, known)
    (output / 'recommendations.json').write_text(json.dumps(load_slots(output / 'measured-slots'), indent=2) + '\n', encoding='utf-8')
    logs = [(f'initial-{index}', path) for index, path in enumerate(paths)] + [('initial-telemetry', telemetry_path)]
    result = collect(repository / WORKSPACES[workspace], repository, logs, complete, chosen)
    budget = lane_budget(json.loads(Path(__file__).with_name('budgets.json').read_text()), workspace, ['selection'], profile)
    violations = build_violations(compiler_observations(result['lanes'].values(), output), budget)
    start = min(row['headers']['Start'] for row in provenance)
    end = max(row['headers']['End'] for row in provenance)
    wall = (datetime.fromisoformat(end) - datetime.fromisoformat(start)).total_seconds()
    native_paths = [Path(row['retained_log']) for row in native_runs]
    lane_counts = {Path(row['working_log']).stem.removeprefix('run-'): dict(Counter(
        event['event'] for event in read_lane(row['retained_log'])['tests'])) for row in native_runs}
    build_wall = sum(read_lane(row['retained_log'])['telemetry']['wall_seconds'] for row in provenance
                     if Path(row['working_log']).name in ('binaries-build.log', 'executables-build.log', 'check.log'))
    command_failures = [row for row in provenance
                        if execution_kind(json.loads(row['headers']['Command'])) is not None
                        and int(row['headers']['Exit code']) != 0]
    documentation_failed = documentation.get('status') != 'pass' or documentation.get('exit_code') != 0
    result.update(documentation=documentation, executed_coverage=executed,
                  build_ratchet={'budget_lane': 'selection', 'violations': violations},
                  status='fail' if violations or executed['status'] == 'fail' or command_failures or documentation_failed else 'pass',
                  capture_recovery={'scope': 'initial run only; no development repairs merged',
                    'captures': provenance, 'selection_source': selection_source, 'original_parent': parent,
                    'source_observation': 'current recovery source, not historical source reconstruction',
                    'cpu_directory': str(Path(cpu).resolve()), 'setup_directory': str(Path(setup).resolve()),
                    'telemetry_log': str(telemetry_path),
                    'telemetry_reused': retained_telemetry is not None,
                    'telemetry_sha256': hashlib.sha256(telemetry_path.read_bytes()).hexdigest(),
                    'setup_observation': 'unmeasured' if not telemetry['setup'] else 'observed',
                    'measured_slots': policy,
                    'execution_command_failures': command_failures, 'documentation_failed': documentation_failed,
                    'lane_counts': lane_counts, 'utc_start': start, 'utc_end': end,
                    'wall_seconds_telemetry': wall, 'build_command_wall_sum_telemetry': build_wall,
                    'phase_build_seconds': None, 'phase_run_seconds': None,
                    'timing_limitation': 'Original monotonic phase clocks lost; command wall sum is not exclusive compile time.'})
    for name, value in (('census', result), ('coverage', structural), ('build-ratchet', violations)):
        (output / f'{name}.json').write_text(json.dumps(value, indent=2) + '\n', encoding='utf-8')
    caution = (f'\n## Initial run recovery\n\nInitial captures only: {executed["executed"]} executed; '
               f'{executed["passed"]} passed; {len(executed["failed"])} failed. '
               f'Build-ratchet violations: {len(violations)}; no budget was relaxed. '
               f'UTC wall telemetry {wall:.3f}s; original monotonic phase times are unknown. '
               'Later repair executions are excluded; this report does not certify an after-fix run.\n')
    if parent is not None:
        caution += ('\nOriginal outer command exit: ' + parent['headers']['Exit code'] +
                    '; the offline proof does not relabel that command. Evidence: `' + parent['path'] + '`.\n')
    (output / 'census.md').write_text(markdown(result) + caution, encoding='utf-8')
    before_path, notes_path = source / 'before.json', source / 'residue-baseline-notes.md'
    before = json.loads(before_path.read_text()) if before_path.exists() else {}
    notes = notes_path.read_text(encoding='utf-8') if notes_path.exists() else ''
    baseline_lanes = [json.loads(path.read_text()) for path in sorted((source / 'baseline').glob('*.json'))]
    residue = generate_residue(workspace, before, result, baseline_lanes, [read_lane(path) for path in native_paths],
                               structural, notes, telemetry_lanes=[telemetry])
    residue['coverage']['runtime_executed'] = f'initial run: {executed["executed"]} observed, {len(executed["failed"])} failed; repairs excluded'
    residue['initial_run'] = {'census': str(output / 'census.json'), 'status': result['status'],
                             'build_ratchet_violations': len(violations)}
    for destination in (output, source):
        (destination / 'residue.json').write_text(json.dumps(residue, indent=2) + '\n', encoding='utf-8')
        (destination / 'residue.md').write_text(residue_markdown(residue) + caution, encoding='utf-8')
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--workspace', choices=WORKSPACES, required=True)
    parser.add_argument('--source', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--cpu', type=Path, required=True)
    parser.add_argument('--setup', type=Path, required=True)
    parser.add_argument('--profile', default='dev')
    parser.add_argument('--selection-manifest', type=Path, help='captured gate-selection.json; retain its exact suites and mains')
    parser.add_argument('--outer-log', type=Path, help='original parent evidence; preserved even when offline proof passes')
    parser.add_argument('--telemetry-capture', type=Path, help='explicit retained aggregation of these same CPU/setup inputs')
    args = parser.parse_args()
    repository = Path(__file__).resolve().parents[2]
    result = recover(repository, args.workspace, args.source, args.output, args.cpu, args.setup,
                     CargoRunner(repository), args.profile, args.selection_manifest, args.outer_log, args.telemetry_capture)
    print(json.dumps({'status': result['status'], 'executed': result['executed_coverage']['executed'],
        'passed': result['executed_coverage']['passed'], 'failed': len(result['executed_coverage']['failed']),
        'build_ratchet_violations': len(result['build_ratchet']['violations']),
        'wall_seconds_telemetry': result['capture_recovery']['wall_seconds_telemetry']}))
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
