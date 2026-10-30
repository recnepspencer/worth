"""Nextest wrapper: execute the exact child once and retain non-gating CPU data."""
from enum import Enum
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time

from telemetry import SchedulerKind


class ObservationFailureKind(Enum):
    RESERVATION = 'reservation-unavailable'
    POOL = 'scheduler-pool-unavailable'
    RETENTION = 'accounting-output-unavailable'


class ObservationPosture(Enum):
    COMPLETE = 'complete'
    UNAVAILABLE = 'observation-unavailable'


def process(command):
    if os.name == 'nt':
        from account_windows import run
        return run(command)
    child = subprocess.Popen(command)
    _, status, usage = os.wait4(child.pid, 0)
    child.returncode = os.waitstatus_to_exitcode(status)
    return child.returncode, usage.ru_utime + usage.ru_stime, 'complete'


def reservation(environment, binary, test):
    path = environment.get('WORTH_TEST_COST_RESERVATIONS')
    if not path:
        return None
    reservations = json.loads(Path(path).read_text(encoding='utf-8'))
    if not isinstance(reservations, dict):
        raise ValueError('reservation file must contain a map')
    slots = reservations[binary + '\n' + test]
    if slots != 'num-test-threads' and (type(slots) is not int or slots < 1):
        raise ValueError('reservation must be a positive integer or num-test-threads')
    return slots


def unavailable(event, kind, error):
    event['observation_posture'] = ObservationPosture.UNAVAILABLE.value
    event['observation_errors'].append({'kind': kind.value, 'error': type(error).__name__,
                                       'detail': str(error)})


def execute(command, environment=None):
    environment = os.environ if environment is None else environment
    start = time.perf_counter()
    # Launch/wait failures retain their real exception. Optional telemetry starts
    # only after the exact child has finished and must never change its outcome.
    code, cpu, posture = process(command)
    wall = time.perf_counter() - start
    binary = environment.get('NEXTEST_BINARY_ID', command[0])
    test = environment.get('NEXTEST_TEST_NAME', '')
    standalone = test == '<main>' or environment.get('WORTH_TEST_COST_SCHEDULER') == SchedulerKind.STANDALONE.value
    kind = SchedulerKind.STANDALONE if standalone else SchedulerKind.NEXTEST
    event = {'type': 'test-cost-cpu', 'binary': binary, 'test': test,
             'cpu_seconds': cpu, 'wall_seconds': wall, 'reserved_slots': None,
             'declared_slots': None, 'scheduler_pool_size': None, 'scheduler': kind.value,
             'accounting': posture, 'process_accounting': posture, 'exit': code,
             'observation_posture': ObservationPosture.COMPLETE.value, 'observation_errors': []}
    if kind is SchedulerKind.NEXTEST:
        try:
            event['declared_slots'] = reservation(environment, binary, test)
        except (OSError, UnicodeError, ValueError, TypeError, KeyError) as error:
            unavailable(event, ObservationFailureKind.RESERVATION, error)
        if event['declared_slots'] is not None:
            try:
                pool = int(environment['NEXTEST_TEST_THREADS'])
                if pool < 1:
                    raise ValueError('nextest pool must be a positive integer')
                event['scheduler_pool_size'] = pool
                requested = pool if event['declared_slots'] == 'num-test-threads' else event['declared_slots']
                event['reserved_slots'] = min(requested, pool)
            except (ValueError, TypeError, KeyError) as error:
                unavailable(event, ObservationFailureKind.POOL, error)
    if directory := environment.get('WORTH_TEST_COST_ACCOUNTING_DIRECTORY'):
        try:
            directory = Path(directory)
            directory.mkdir(parents=True, exist_ok=True)
            identity = hashlib.sha256((binary + '\n' + test).encode()).hexdigest()
            destination = directory / f'{identity}-{os.getpid()}.json'
            temporary = destination.with_suffix('.tmp')
            temporary.write_text(json.dumps(event, sort_keys=True) + '\n', encoding='utf-8')
            os.replace(temporary, destination)
        except (OSError, UnicodeError, ValueError, TypeError) as error:
            unavailable(event, ObservationFailureKind.RETENTION, error)
    if event['observation_errors']:
        try:
            diagnostic = {**event, 'type': 'test-cost-observation-error'}
            sys.stderr.write(json.dumps(diagnostic, sort_keys=True) + '\n')
        except (OSError, UnicodeError, ValueError, TypeError):
            # Losing the optional diagnostic must preserve the child outcome too.
            pass
    return code, event


if __name__ == '__main__':
    if len(sys.argv) < 2:
        raise SystemExit('accounting wrapper needs the child executable and its arguments')
    code, _ = execute(sys.argv[1:])
    # Preserve Windows exception status bits through Python's signed exit API.
    sys.exit(code if code < 2**31 else code - 2**32)
