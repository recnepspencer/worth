#!/usr/bin/env python3
"""Exercise compiler JSON, prebuilt discovery, standalone mains and CPU transport."""
import json
from pathlib import Path
import sys

from cargo_capture import CargoRunner
from nextest_build import build, coverage
from logs import read_lane
from reservations import configuration
from run_lane import child_environment
from standalone import execute as execute_standalone
from workspace import repository_root


def main():
    repository = repository_root(Path.cwd())
    output = repository / 'target/test-cost/transport-probe'
    output.mkdir(parents=True, exist_ok=True)
    (output / 'src').mkdir(exist_ok=True)
    (output / 'examples').mkdir(exist_ok=True)
    (output / 'Cargo.toml').write_text(
        '[package]\nname="test-cost-transport-probe"\nversion="0.0.0"\nedition="2024"\n'
        '[workspace]\n[[example]]\nname="standalone"\ntest=true\nharness=false\n', encoding='utf-8')
    (output / 'src/lib.rs').write_text(
        '#[test] fn compiler_transport_is_live() { assert_eq!(2+2,4); }\n'
        '#[test] #[ignore] fn scheduled_probe_is_never_executed() { panic!("scheduled-only"); }\n', encoding='utf-8')
    (output / 'examples/standalone.rs').write_text(
        'fn main() { assert_eq!(std::env::args().count(), 1); }\n', encoding='utf-8')
    environment = child_environment()
    environment['CARGO_TARGET_DIR'] = str(repository / 'target')
    environment['WORTH_TEST_COST_ACCOUNTING_DIRECTORY'] = str(output / 'cpu')
    runner = CargoRunner(repository, environment)
    resume = '--resume' in sys.argv[1:]
    if resume:
        proof = json.loads((output / 'build/coverage.json').read_text())
        observed = read_lane(output / 'nextest-run.log')
        if observed['headers']['Exit code'] != '0' or len(observed['tests']) != 1:
            raise ValueError('no successful native probe execution to reuse')
    else:
        reuse = build(output / 'Cargo.toml', output / 'build', runner)
        discovered, selections, proof = coverage(reuse, output / 'build', runner)
    programs = json.loads((output / 'build/standalone.json').read_text())
    if len(programs) != 1 or proof['scheduled_only'] != 1:
        raise ValueError('transport lost a main or an ignored-test inventory entry')
    generated = output / 'nextest.toml'
    configuration(output / 'no-original.toml', generated, 'ci', {}, Path(__file__).with_name('account.py'))
    reservations = output / 'reservations.json'
    reservations.write_text(json.dumps({row['binary'] + '\n' + row['test']: 1 for row in proof['tests']}))
    environment['WORTH_TEST_COST_RESERVATIONS'] = str(reservations)
    if not resume:
        observed = runner(['cargo', 'nextest', 'run', *reuse, '--profile', 'ci', '--config-file', str(generated),
                           '--ignore-default-filter', '--no-tests=fail', '--message-format', 'libtest-json'],
                          output, output / 'nextest-run.log')
        if observed.returncode:
            raise ValueError('prebuilt wrapper transport failed')
    if execute_standalone(programs[0], runner, output / 'standalone-run.log', environment).returncode:
        raise ValueError('standalone main transport failed')
    records = [json.loads(path.read_text()) for path in (output / 'cpu').glob('*.json')]
    if len(records) != 2 or any(row['cpu_seconds'] is None for row in records):
        raise ValueError('CPU transport did not account both actual child processes')
    print(json.dumps({'pass': 2, 'fail': 0, 'ignored': 1, 'coverage': proof['status'],
                      'actual_children': len(records)}, sort_keys=True))


if __name__ == '__main__':
    main()
