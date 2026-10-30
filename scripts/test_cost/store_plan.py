#!/usr/bin/env python3
"""Execute the Store runner's structured plan through one prebuilt graph."""
import json
import os
from pathlib import Path
import sys

from cargo_capture import CargoRunner
from evidence import run
from run_lane import child_environment, execute
from workspace import repository_root


def option(arguments, name, default=None):
    return arguments[arguments.index(name) + 1] if name in arguments else default


def nextest_query(arguments):
    expression = option(arguments, '--filterset', option(arguments, '-E', 'all()'))
    packages, kinds, binaries = [], [], []
    for index, argument in enumerate(arguments):
        if argument in ('-p', '--package'):
            packages.append(f'package(={arguments[index + 1]})')
        if argument in ('--lib', '--bins', '--examples', '--benches', '--tests'):
            kind = {'--lib': 'lib', '--bins': 'bin', '--examples': 'example',
                    '--benches': 'bench', '--tests': 'test'}[argument]
            kinds.append(f'kind({kind})')
        if argument == '--test':
            binaries.append(f'binary(={arguments[index + 1]})')
        if argument in ('--include-ignored', '--ignored', '--run-ignored'):
            raise ValueError('scheduled ignored tests cannot run in this plan')
    for clauses in (packages, kinds, binaries):
        if clauses:
            expression = f'({expression}) & (' + ' + '.join(clauses) + ')'
    return expression


def execute_plan(repository, plan, output):
    queries, commands, profiles = [], [], set()
    for identity, directory, program, arguments in plan:
        if program == 'cargo' and arguments[:2] == ['nextest', 'run']:
            profiles.add(option(arguments, '--cargo-profile', 'dev'))
            queries.append((identity, nextest_query(arguments), option(arguments, '--partition'),
                            option(arguments, '--profile', 'ci')))
        else:
            commands.append((identity, directory, program, arguments))
    if len(profiles) > 1:
        raise ValueError(f'one Store plan declares multiple build profiles: {sorted(profiles)}')
    if queries:
        execute(repository, 'store', (), output, next(iter(profiles)), queries=queries)
    environment = child_environment()
    environment_file = output / 'prebuilt/environment.json'
    if environment_file.exists():
        environment.update(json.loads(environment_file.read_text(encoding='utf-8')))
    runner = CargoRunner(repository, environment)
    for identity, directory, program, arguments in commands:
        destination = output / f'{identity.replace(":", "-")}.log'
        command = [program, *arguments]
        if program == 'cargo':
            if runner(command, directory, destination).returncode:
                raise ValueError(f'{identity} failed')
        elif run(command, destination, cwd=directory, environment=environment):
            raise ValueError(f'{identity} failed')


def main():
    root = repository_root(Path.cwd())
    execute_plan(root, json.load(sys.stdin), root / 'target/test-cost/store')


if __name__ == '__main__':
    main()
