#!/usr/bin/env python3
"""Prebuild once, prove partitions, and run governed tests without nested Cargo."""
import argparse
import json
import os
from pathlib import Path
import sys
import time

from cargo_capture import CargoRunner, capture_record
from census import collect, markdown
from compiler_prebuild import CompilerScope
from coverage import inventory
from logs import read_lane
from measured_slots import load as load_slots, retain as retain_slots
from nextest_build import LANES, build, coverage, list_tests
from prebuild import prepare
from ratchet import build_violations, lane_budget, source_violations
from reservations import configuration, original_reservations
from standalone import execute as execute_standalone
from workspace import WORKSPACES, repository_root


LANDING = tuple(name for name in LANES if name != 'scale')


def selected_lanes(names):
    names = names.split(',') if isinstance(names, str) else names
    unknown = set(names) - set(LANDING)
    if unknown:
        raise ValueError(f'unknown or scheduled-only lanes: {sorted(unknown)}')
    if len(set(names)) != len(names):
        raise ValueError('lane repeats in execution plan')
    return names


def child_environment():
    value = os.environ.copy()
    value.update(CARGO_PROFILE_DEV_DEBUG='0', CARGO_PROFILE_TEST_DEBUG='0',
                 CARGO_PROFILE_CI_TEST_DEBUG='0', CARGO_PROFILE_DIAGNOSTIC_TEST_DEBUG='0',
                 WORTH_IMPLEMENTATION_BUILD_ID='00000000-0000-4000-8000-000000000001',
                 NEXTEST_EXPERIMENTAL_LIBTEST_JSON='1')
    return value


def execute(repository, workspace, lanes, output, profile='dev', expression=None,
            partition=None, nextest_profile='ci', cargo_runner=None, queries=None, build_only=False):
    repository, output = Path(repository).resolve(), Path(output).resolve()
    output.mkdir(parents=True, exist_ok=True)
    environment = (cargo_runner.environment if cargo_runner is not None and
                   cargo_runner.environment is not None else child_environment())
    # A gate may already own its build identity and complete child environment.
    # Retain that dictionary so published prebuild paths reach its CargoRunner.
    for key, value in child_environment().items():
        environment.setdefault(key, value)
    for key in ('CARGO_PROFILE_DEV_DEBUG', 'CARGO_PROFILE_TEST_DEBUG',
                'CARGO_PROFILE_CI_TEST_DEBUG', 'CARGO_PROFILE_DIAGNOSTIC_TEST_DEBUG'):
        environment[key] = '0'
    runner = cargo_runner or CargoRunner(repository, environment)
    manifest = repository / WORKSPACES[workspace]
    budgets = json.loads(Path(__file__).with_name('budgets.json').read_text())
    budget = lane_budget(budgets, workspace, lanes, profile)
    environment.setdefault('WORTH_REPO', str(repository))
    environment.setdefault('RUST_MIN_STACK', '16777216')
    if profile == 'ci-test':
        environment['CARGO_INCREMENTAL'] = '0'
    # Refuse the source boundary before any expensive build.
    before = collect(manifest, repository)
    forbidden = source_violations(before['source_sites'])
    if forbidden:
        raise ValueError('tests invoke compilers: ' + json.dumps(forbidden))
    started = time.perf_counter()
    reuse = build(manifest, output / 'build', runner, profile)
    discovered, selections, proof = coverage(reuse, output / 'build', runner)
    if any(not row['ignored'] for row in inventory(selections['scale']) if row.get('selected', True)):
        raise ValueError('scheduled scale tests must be explicitly ignored; no scale lane is executed')
    if build_only:
        return {'coverage': proof, 'reuse': reuse}
    if callable(queries):
        queries = queries(discovered)
    programs = json.loads((output / 'build/standalone.json').read_text(encoding='utf-8'))
    if workspace == 'bank':
        binaries = runner(['cargo', 'build', '--workspace', '--bins', '--profile', profile,
                           '--manifest-path', str(manifest)], manifest.parent, output / 'bank-binaries.log')
        if binaries.returncode:
            raise ValueError('Bank product executable prebuild failed')
    scope = CompilerScope.ALL if 'compile-fail' in lanes or expression or queries else (
        CompilerScope.UNITS if 'owner-unit' in lanes else CompilerScope.NONE)
    environment.update(prepare(repository, workspace, output / 'prebuilt', runner,
                               include_proof=workspace == 'root', compiler_scope=scope, profile=profile))
    config = manifest.parent / '.config/nextest.toml'
    reservations = original_reservations(config, nextest_profile, discovered, lambda index, query:
        list_tests(reuse, output / 'build' / f'reservation-{index}.json', runner, query))
    reservation_path = output / 'reservations.json'
    reservation_path.write_text(json.dumps(reservations, indent=2) + '\n', encoding='utf-8')
    generated = output / 'nextest.toml'
    previous_path = output / 'recommendations.json'
    slot_directory = repository / 'scripts/test_cost/slots' / workspace
    previous = load_slots(slot_directory)
    configuration(config, generated, nextest_profile, previous, Path(__file__).with_name('account.py'))
    reservations.update(previous)
    reservation_path.write_text(json.dumps(reservations, indent=2) + '\n', encoding='utf-8')
    accounting = output / 'cpu' / str(time.time_ns())
    setup = output / 'setup' / str(time.time_ns())
    environment.update(WORTH_TEST_COST_RESERVATIONS=str(reservation_path),
                       WORTH_TEST_COST_ACCOUNTING_DIRECTORY=str(accounting),
                       WORTH_TEST_COST_SETUP_DIRECTORY=str(setup))
    build_seconds = time.perf_counter() - started
    failures = []
    jobs = queries if queries is not None else [(lane, expression or LANES[lane], partition, nextest_profile,
                       [program['binary'] for program in programs if not expression and
                        lane == ('ordinary' if program['kind'] == 'test' else 'owner-unit')])
                       for lane in (['selection'] if expression else lanes)]
    executed_mains = set()
    for job in jobs:
        lane, query, shard, runtime_profile = job[:4]
        mains = job[4] if len(job) == 5 else []
        command = ['cargo', 'nextest', 'run', *reuse, '--profile', runtime_profile,
                   '--config-file', str(generated), '--ignore-default-filter',
                   '--no-fail-fast', '--no-tests=' + ('fail' if (expression or queries) and not mains else 'pass'),
                   '--message-format', 'libtest-json', '-E', query]
        if shard:
            command += ['--partition', shard]
        if query is not None:
            observed = runner(command, manifest.parent, output / f'run-{lane}.log')
            if observed.returncode:
                failures.append(lane)
        for program in programs:
            if program['binary'] not in mains:
                continue
            if program['binary'] in executed_mains:
                raise ValueError('standalone program selected by more than one lane: ' + program['binary'])
            executed_mains.add(program['binary'])
            observed = execute_standalone(program, runner, output / f'run-{lane}-{program["target"]}-main.log', environment)
            if observed.returncode:
                failures.append(lane + ':' + program['target'])
    observations = []
    telemetry = output / 'telemetry.jsonl'
    with telemetry.open('w', encoding='utf-8') as sink:
        for directory, pattern in ((accounting, '*.json'), (setup, '*.jsonl')):
            for path in sorted(directory.glob(pattern)):
                sink.write(path.read_text(encoding='utf-8').rstrip() + '\n')
        sink.write(json.dumps({'type': 'test-cost-phases', 'build_seconds': build_seconds,
                               'wall_seconds': time.perf_counter() - started}) + '\n')
    observed = read_lane(telemetry)
    capture_record(environment, telemetry)
    baseline = original_reservations(config, nextest_profile, discovered, lambda index, query:
        json.loads((output / 'build' / f'reservation-{index}.json').read_text(encoding='utf-8')))
    known = {row['binary'] + '\n' + row['test'] for row in inventory(discovered)}
    retain_slots(slot_directory, observed['cpu'], read_lane(runner.logs[0])['headers']['HEAD'],
                 str(telemetry.relative_to(repository)), baseline, known)
    previous_path.write_text(json.dumps(load_slots(slot_directory), indent=2) + '\n', encoding='utf-8')
    logs = [(path.stem + '-' + str(index), path) for index, path in enumerate(runner.logs)]
    logs.append(('telemetry', telemetry))
    result = collect(manifest, repository, logs, discovered, selections)
    (output / 'census.json').write_text(json.dumps(result, indent=2) + '\n', encoding='utf-8')
    (output / 'census.md').write_text(markdown(result), encoding='utf-8')
    # Count actual compiler units across all invocations in this lane, including
    # executable and fixture prebuilds. A warm graph remains a measured zero.
    compiled = [artifact for lane in result['lanes'].values() for artifact in lane['artifacts']]
    build_observation = 'json' if any(lane['build_observation'] == 'json'
                                     for lane in result['lanes'].values()) else 'unmeasured'
    violations = build_violations({'build_observation': build_observation, 'artifacts': compiled,
                                  'log': str(output)}, budget)
    (output / 'build-ratchet.json').write_text(json.dumps(violations, indent=2) + '\n', encoding='utf-8')
    if violations:
        raise ValueError('build budget exceeded: ' + json.dumps(violations))
    if failures:
        raise ValueError(f'test lanes failed: {failures}')
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--workspace', choices=WORKSPACES, required=True)
    parser.add_argument('--lanes', default=','.join(LANDING))
    parser.add_argument('--output', type=Path)
    parser.add_argument('--cargo-profile', default='dev')
    parser.add_argument('--nextest-profile', default='ci')
    parser.add_argument('--filter')
    parser.add_argument('--partition')
    parser.add_argument('--build-only', action='store_true', help='build and prove coverage without executing tests')
    args = parser.parse_args()
    root = repository_root(Path.cwd())
    output = args.output or root / 'target/test-cost' / args.workspace
    execute(root, args.workspace, selected_lanes(args.lanes), output, args.cargo_profile,
            args.filter, args.partition, args.nextest_profile, build_only=args.build_only)
    return 0


if __name__ == '__main__':
    sys.exit(main())
