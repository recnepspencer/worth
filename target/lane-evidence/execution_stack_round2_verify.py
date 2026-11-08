"""One serial focused verification batch, only after the stack source checks."""
from pathlib import Path
import json
import os
import re
import shutil
import struct
import sys

r = Path.cwd()
e = r / 'target/lane-evidence'
out = e / 'execution-stack-round2-verification'
out.mkdir(exist_ok=True)
sys.path[:0] = [str(r / 'scripts'), str(r / 'scripts/test_cost'), str(r / 'scripts/ci')]
from application_build import application_build_environment
from cargo_capture import CargoRunner
from coverage import inventory
from nextest_build import build, list_tests
from query_gate_environment import environment as fixture_environment
from run_lane import child_environment

ready = json.loads((e / 'execution-stack-check-results.json').read_text())
assert set(ready) == {'root', 'query', 'store', 'bank', 'fixture'}
assert not any(ready.values()), ready
env = application_build_environment(child_environment())
for key in ('RUST_MIN_STACK', 'WORTH_BANK_ASSESSMENT_WIDTH', 'WORTH_BANK_HISTORY_WIDTH'):
    env.pop(key, None)
env.setdefault('WORTH_REPO', str(r))
results = []


def remember(label, code, **facts):
    results.append({'lane': label, 'exit': code, **facts})
    (out / 'results.json').write_text(json.dumps(results, indent=2))
    print('RESULT', label, code, flush=True)


def floor():
    free = shutil.disk_usage('C:/').free
    print('C_FREE_BYTES', free, flush=True)
    assert free >= 25 * 1024**3, 'Build not started: disk floor'


def main_stack(executable):
    with Path(executable).open('rb') as stream:
        stream.seek(0x3c)
        pe = struct.unpack('<I', stream.read(4))[0]
        stream.seek(pe + 24)
        assert struct.unpack('<H', stream.read(2))[0] == 0x20b
        stream.seek(pe + 24 + 72)
        reserve = struct.unpack('<Q', stream.read(8))[0]
    assert reserve == 1024 * 1024, (executable, reserve)
    return reserve


def exact_tests(workspace, manifest, package, names, target=None):
    floor()
    directory = out / workspace
    directory.mkdir(exist_ok=True)
    runner = CargoRunner(r, env)
    selection = {'packages': [package]}
    if target:
        selection['targets'] = {'test': [target]}
    reuse = build(r / manifest, directory / 'build', runner, profile='dev', selection=selection)
    expression = ' + '.join('test(=' + name + ')' for name in names)
    rows = [row for row in inventory(list_tests(reuse, directory / 'selection.json', runner, expression))
            if row['selected']]
    assert len(rows) == len(names) and {row['test'] for row in rows} == set(names)
    assert all(not row['ignored'] for row in rows)
    assert all(not re.search(r'(^|::)(exhaustive|scale|query_scale|scenario|scenarios)::', n) for n in names)
    (directory / 'selection-audit.json').write_text(json.dumps(rows, indent=2))
    binaries = json.loads((directory / 'build/binaries.json').read_text())['rust-binaries']
    choices = [b for b in binaries.values() if b['binary-name'] == (target or package.replace('-', '_'))]
    assert len(choices) == 1, choices
    binary = choices[0]
    listed = runner([binary['binary-path'], '--list', '--exact', *names],
                    (r / manifest).parent, directory / 'direct-list.log')
    assert listed.returncode == 0
    assert all(name + ': test' in listed.stdout.splitlines() for name in names)
    result = runner([binary['binary-path'], '--exact', *names, '--nocapture'],
                    (r / manifest).parent, directory / 'direct-run.log')
    remember(workspace + '-tests', result.returncode, selected=names,
             rust_min_stack_present='RUST_MIN_STACK' in env)


# All four genuine mains belong to the consumer fixture workspace. Build the
# exact three examples and consumer binary together, with its owned environment.
floor()
manifest = r / 'workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/Cargo.toml'
consumer_env = {**env, **fixture_environment(r)}
consumer_env.pop('RUST_MIN_STACK', None)
runner = CargoRunner(r, consumer_env)
examples = ['invariant_reads', 'required_feature', 'shared_execution']
result = runner(['cargo', 'build', '--manifest-path', str(manifest),
                 '-p', 'worth-query-topology-entry', '-p', 'worth-query-consumer-root',
                 '--bins', '--examples', '--tests', '--message-format=json'],
                manifest.parent, out / 'consumer-mains-build.log')
remember('consumer-mains-build', result.returncode)
assert result.returncode == 0
artifacts = {}
consumer_test = None
for line in result.stdout.splitlines():
    try:
        value = json.loads(line)
    except ValueError:
        continue
    if value.get('reason') == 'compiler-artifact' and value.get('executable'):
        name = value['target']['name']
        kind = value['target']['kind']
        is_test = value.get('profile', {}).get('test', False)
        if name == 'worth-query-consumer-root' and 'bin' in kind and is_test:
            consumer_test = value['executable']
        elif not is_test and ((name in examples and 'example' in kind) or (name == 'worth-query-consumer-root' and 'bin' in kind)):
            artifacts[name] = value['executable']
assert set(artifacts) == set(examples + ['worth-query-consumer-root']), artifacts
for name in examples + ['worth-query-consumer-root']:
    reserve = main_stack(artifacts[name])
    result = runner([artifacts[name]], manifest.parent, out / (name + '-main.log'))
    remember(name + '-main', result.returncode, pe_stack_reserve_bytes=reserve)

# The first census's interrupted growth chunk could not flush this failed
# retention assertion. Its default-stack diagnostic overflowed before the
# repair; recover the real outcome once at the final-source default stack.
assert consumer_test, 'Consumer libtest artifact was not built'
retention = 'application_invariant_acceptance::proof::application_program::discovered::retention::discovered_committed_retention_refuses_twice_then_promotes_original_carrier'
listed = runner([consumer_test, '--list', '--exact', retention], manifest.parent / 'consumer_root', out / 'retention-list.log')
assert listed.returncode == 0 and retention + ': test' in listed.stdout.splitlines()
result = runner([consumer_test, '--exact', retention, '--nocapture'], manifest.parent / 'consumer_root', out / 'retention-default-run.log')
remember('retention-default', result.returncode, selected=[retention], cleanup_assertion_may_remain=True)

plan = json.loads((e / 'head-closure-final-bank-plan.json').read_text())
exact_tests('bank', 'workspaces/worth-query-bank-world/Cargo.toml',
            plan['packages'][0], plan['names'], 'worth_query_bank_court_external_rail_server')
raise SystemExit(int(any(row['exit'] for row in results)))
