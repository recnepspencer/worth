"""Nextest wrapper: execute the exact child once and retain non-gating CPU data."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time


def process(command):
    if os.name == 'nt':
        from account_windows import run
        return run(command)
    child = subprocess.Popen(command)
    _, status, usage = os.wait4(child.pid, 0)
    child.returncode = os.waitstatus_to_exitcode(status)
    return child.returncode, usage.ru_utime + usage.ru_stime, 'complete'


def execute(command, environment=None):
    environment = os.environ if environment is None else environment
    start = time.perf_counter()
    code, cpu, posture = process(command)
    wall = time.perf_counter() - start
    binary = environment.get('NEXTEST_BINARY_ID', command[0])
    test = environment.get('NEXTEST_TEST_NAME', '')
    slots = None
    if path := environment.get('WORTH_TEST_COST_RESERVATIONS'):
        reservations = json.loads(Path(path).read_text(encoding='utf-8'))
        slots = reservations.get(binary + '\n' + test)
        if slots == 'num-test-threads':
            slots = int(environment['NEXTEST_TEST_THREADS'])
    event = {'type': 'test-cost-cpu', 'binary': binary, 'test': test,
             'cpu_seconds': cpu, 'wall_seconds': wall, 'reserved_slots': slots,
             'accounting': posture, 'exit': code}
    if directory := environment.get('WORTH_TEST_COST_ACCOUNTING_DIRECTORY'):
        directory = Path(directory)
        directory.mkdir(parents=True, exist_ok=True)
        identity = hashlib.sha256((binary + '\n' + test).encode()).hexdigest()
        (directory / f'{identity}-{os.getpid()}.json').write_text(
            json.dumps(event, sort_keys=True) + '\n', encoding='utf-8')
    return code, event


if __name__ == '__main__':
    if len(sys.argv) < 2:
        raise SystemExit('accounting wrapper needs the child executable and its arguments')
    code, _ = execute(sys.argv[1:])
    # Preserve Windows exception status bits through Python's signed exit API.
    sys.exit(code if code < 2**31 else code - 2**32)
