"""Inventory and execute declared harness-free test programs exactly once."""
from copy import deepcopy
from enum import Enum
import json
import os
from pathlib import Path
import sys
import time
import tomllib

from logs import json_objects
from telemetry import SchedulerKind


class StandaloneFailureKind(Enum):
    INVALID_ARTIFACT = 'invalid-artifact'
    AMBIGUOUS_EXECUTABLE = 'ambiguous-executable'
    MISSING_EXECUTABLE = 'missing-executable'
    CONFLICTING_IDENTITY = 'conflicting-identity'


class StandaloneFailure(ValueError):
    def __init__(self, kind, context):
        self.kind = kind
        super().__init__(f'{kind.value}: {context}')


def filesystem_path(path):
    path = Path(path).absolute()
    return Path('\\\\?\\' + str(path)) if os.name == 'nt' and not str(path).startswith('\\\\?\\') else path


def executable_key(path):
    return os.path.normcase(str(Path(path).resolve()))


def declaration(configuration, kind, name):
    if kind == 'lib':
        return configuration.get('lib', {})
    return next((row for row in configuration.get(kind, []) if row['name'] == name), {})


def discover(compiler_messages):
    """Use real executable artifacts and their manifests, including fresh ones.

    Multiple Cargo records for the same path are one program. Distinct test
    and non-test executable candidates remain visible until nextest metadata
    identifies the actual selected executable.
    """
    events = json_objects(compiler_messages) if isinstance(compiler_messages, str) else compiler_messages
    manifests, programs = {}, {}
    for event in events:
        if event.get('reason') != 'compiler-artifact' or not event.get('executable'):
            continue
        target = event['target']
        kind = next((k for k in ('test', 'example', 'bench', 'bin', 'lib') if k in target['kind']), None)
        if kind is None:
            continue
        manifest = Path(event['manifest_path']).resolve()
        if manifest not in manifests:
            manifests[manifest] = tomllib.loads(filesystem_path(manifest).read_text(encoding='utf-8'))
        configuration = manifests[manifest]
        row = declaration(configuration, kind, target['name'])
        enabled = target.get('test', row.get('test', event.get('profile', {}).get('test', False)))
        if row.get('harness', True) or not enabled:
            continue
        package = configuration['package']['name']
        executable = str(Path(event['executable']).resolve())
        identity = (package, kind, target['name'], executable_key(executable))
        programs[identity] = {
            'binary': (package if kind == 'lib' else f'{package}::{target["name"]}'
                       if kind == 'test' else f'{package}::{kind}/{target["name"]}'),
            'test': '<main>', 'ignored': False,
            'package': package, 'target': target['name'], 'kind': kind, 'lib': kind == 'lib',
            'harness': False, 'exe': executable, 'cwd': str(manifest.parent),
            'manifest': str(manifest), 'profile_test': event.get('profile', {}).get('test', False),
        }
    return sorted(programs.values(), key=lambda p: (p['binary'], p['exe']))


def separate(binaries_summary, candidates):
    """Remove mains before libtest discovery and bind them to nextest identities."""
    summary = deepcopy(binaries_summary)
    binaries = summary.get('rust-binaries')
    if not isinstance(binaries, dict):
        raise StandaloneFailure(StandaloneFailureKind.INVALID_ARTIFACT, 'missing rust-binaries map')
    by_path = {executable_key(p['exe']): p for p in candidates}
    chosen, bound = {}, set()
    for binary, record in list(binaries.items()):
        path = record.get('binary-path')
        program = by_path.get(executable_key(path)) if path else None
        if program is None:
            continue
        key = (program['package'], program['kind'], program['target'])
        if key in chosen:
            raise StandaloneFailure(StandaloneFailureKind.CONFLICTING_IDENTITY,
                                    f'{key} has multiple nextest binary identities')
        chosen[key] = dict(program, binary=binary)
        bound.add(key)
        del binaries[binary]
    unbound = {}
    for program in candidates:
        key = (program['package'], program['kind'], program['target'])
        if key not in bound:
            unbound.setdefault(key, []).append(program)
    for key, choices in unbound.items():
        test_choices = [p for p in choices if p['profile_test']]
        choices = test_choices or choices
        if len(choices) != 1:
            raise StandaloneFailure(StandaloneFailureKind.AMBIGUOUS_EXECUTABLE, str(key))
        chosen[key] = choices[0]
    return summary, sorted(chosen.values(), key=lambda p: p['binary'])


def synthetic_inventory(programs):
    """A main's successful process exit is one explicit, nonignored test claim."""
    result = {'rust-suites': {}}
    for program in programs:
        binary = program['binary']
        if binary in result['rust-suites']:
            raise StandaloneFailure(StandaloneFailureKind.CONFLICTING_IDENTITY, binary)
        result['rust-suites'][binary] = {
            'package-name': program['package'], 'binary-name': program['target'],
            'kind': program['kind'], 'status': 'listed', 'harness': False,
            'testcases': {'<main>': {'ignored': False, 'filter-match': {'status': 'matches'}}},
        }
    return result


def execute(program, runner, log_path, environment):
    """Run the independently built executable with zero libtest arguments.

    The runner owns full command evidence. CPU accounting uses the same wrapper
    as nextest. Standalone programs run sequentially outside the nextest pool
    and therefore have no nextest slot reservation.
    """
    if not filesystem_path(program['exe']).is_file():
        raise StandaloneFailure(StandaloneFailureKind.MISSING_EXECUTABLE, program['exe'])
    log_path = Path(log_path)
    log_path.parent.mkdir(parents=True, exist_ok=True)
    child = dict(environment, CARGO_MANIFEST_DIR=program['cwd'],
                 NEXTEST_BINARY_ID=program['binary'], NEXTEST_TEST_NAME='<main>',
                 WORTH_TEST_COST_SCHEDULER=SchedulerKind.STANDALONE.value)
    previous = runner.environment
    started = time.perf_counter()
    try:
        runner.environment = child
        result = runner([sys.executable, str(Path(__file__).with_name('account.py').resolve()), program['exe']],
                        program['cwd'], log_path)
    finally:
        runner.environment = previous
    event = {'type': 'test', 'event': 'ok' if result.returncode == 0 else 'failed',
             'name': program['binary'] + '$<main>', 'exec_time': time.perf_counter() - started,
             'test-cost': {'execution': 'standalone-process', 'target-kind': program['kind']}}
    with log_path.open('a', encoding='utf-8') as stream:
        stream.write('\n' + json.dumps(event) + '\n')
    if getattr(runner, 'logs', []) and Path(runner.logs[-1]).resolve() != log_path.resolve():
        with Path(runner.logs[-1]).open('a', encoding='utf-8') as stream:
            stream.write('\n' + json.dumps(event) + '\n')
    return result
