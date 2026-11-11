"""Reuse a completed owned prebuild for one corrected selected nextest run.

usage: python target/lane-evidence/claude_lane_tests.py <kind> <label> <cases.json> [name...]
kind: root (relational certification court) | native (worth-relational lib units)
cases.json: JSON list of test function names (last path segment, unique).
Never selects ignored bodies: an ignored selected case fails the assertion.
"""
from pathlib import Path
import json
import sys

root = Path.cwd().resolve()
sys.path.insert(0, str(root / 'scripts/test_cost'))
sys.path.insert(0, str(root / 'scripts'))
from application_build import application_build_environment
from run_lane import child_environment
from cargo_capture import CargoRunner
from nextest_build import build, list_tests
from coverage import inventory
from logs import json_objects
from reservations import configuration

kind, label = sys.argv[1], sys.argv[2]
cases = json.loads(Path(sys.argv[3]).read_text(encoding='utf-8'))
reuse_label = sys.argv[4]
if len(sys.argv) > 5:
    requested = set(sys.argv[5:])
    assert requested <= set(cases), requested - set(cases)
    cases = [name for name in cases if name in requested]
assert len(cases) == len(set(cases))
out = root / 'target/test-cost' / label
out.mkdir(parents=True, exist_ok=True)
manifest, packages, targets = {
    'root': ('Cargo.toml', ['worth-court-relational-certification'],
             {'test': ['worth_court_relational_certification']}),
    'native': ('Cargo.toml', ['worth-relational'], {'lib': True}),
    'query': ('workspaces/worth-query/Cargo.toml', ['worth-query-court-query-decl-host'],
              {'test': ['worth_query_court_query_decl_host']}),
    'store': ('workspaces/worth-store/Cargo.toml', ['worth-store-court-application'],
              {'test': ['worth_store_court_application']}),
}[kind]
manifest = root / manifest
build_directory = root / 'target/test-cost' / reuse_label / 'build'
# Restore the identity actually issued for the successful owned prebuild.
contract = next(event for event in json_objects((build_directory / 'binaries-build.log').read_text(encoding='utf-8'))
                if event.get('type') == 'cargo-invocation')
environment = child_environment()
environment.update(contract['compiler_contract']['compiler_inputs']['environment'])
runner = CargoRunner(root, environment)
reuse = ['--cargo-metadata', str(build_directory / 'cargo-metadata.json'),
         '--binaries-metadata', str(build_directory / 'binaries.json')]
assert all(Path(reuse[index]).is_file() for index in [1, 3])
print('REUSED_OWNED_PREBUILD:', build_directory, flush=True)
expression = ' | '.join('test(' + name + ')' for name in cases)
selected = [row for row in inventory(list_tests(reuse, out / 'selection.json', runner, expression))
            if row['selected']]
selected = [row for row in selected if row['test'].rsplit('::', 1)[-1] in set(cases)]
missing = set(cases) - {row['test'].rsplit('::', 1)[-1] for row in selected}
assert not missing, ('missing cases', sorted(missing))
assert len(selected) == len(cases), ('ambiguous case names', len(selected), len(cases))
assert not any(row['ignored'] for row in selected), [r['test'] for r in selected if r['ignored']]
slots = 3
reservations = {row['binary'] + '\n' + row['test']: slots for row in selected}
(out / 'reservations.json').write_text(json.dumps(reservations, indent=2) + '\n', encoding='utf-8')
environment.update(WORTH_REPO=str(root), WORTH_TEST_COST_RESERVATIONS=str(out / 'reservations.json'),
                   WORTH_TEST_COST_ACCOUNTING_DIRECTORY=str(out / 'cpu'),
                   WORTH_TEST_COST_SETUP_DIRECTORY=str(out / 'setup'))
config = manifest.parent / '.config/nextest.toml'
if not config.exists():
    config = root / '.config/nextest.toml'
configuration(config, out / 'nextest.toml', 'ci', reservations, root / 'scripts/test_cost/account.py')
exact = ' | '.join('test(=' + row['test'] + ')' for row in selected)
observed = runner(['cargo', 'nextest', 'run', *reuse, '--profile', 'ci', '--config-file',
                   str(out / 'nextest.toml'), '--ignore-default-filter', '--no-fail-fast',
                   '--no-tests=fail', '--message-format', 'libtest-json', '-E', exact],
                  manifest.parent, out / 'run.log')
counts = [0, 0, 0]
failed = []
for event in json_objects(observed.stdout):
    if event.get('type') == 'test' and event.get('event') in ('ok', 'failed'):
        counts[0 if event['event'] == 'ok' else 1] += 1
    if event.get('type') == 'test' and event.get('event') == 'failed':
        failed.append(event.get('name', '?'))
        print('FAILED_CASE:', event.get('name', '?'), flush=True)
        print(event.get('stdout', ''), flush=True)
for path in runner.logs:
    print('FULL_OWNED_LOG_PATH:', path)
print('FAILED_NAMES: ' + json.dumps(failed), flush=True)
print('EVIDENCE_TEST_COUNTS: ' + ','.join(map(str, counts)), flush=True)
sys.exit(observed.returncode)
