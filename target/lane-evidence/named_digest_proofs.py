"""One owned prebuild and one selected run per affected funding proof binary."""
from pathlib import Path
import json
import os
import re
import subprocess
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

def prior_cases(label):
    command = (root / 'target/lane-evidence' / (label + '.log')).read_text(encoding='utf-8').splitlines()[0]
    return re.findall(r'test\(([^)]+)\)', command)

kind = sys.argv[1]
label = 'named-digest-proof-' + kind
out = root / 'target/test-cost' / label
out.mkdir(parents=True, exist_ok=True)
cases = json.loads((root/'target/lane-evidence/named-digest-proof-selection.json').read_text())
if len(sys.argv) > 2:
    requested = set(sys.argv[2:])
    assert requested <= set(cases), requested - set(cases)
    cases = [name for name in cases if name in requested]
assert len(cases) == len(set(cases))
manifest, packages, targets = {
 'root': ('Cargo.toml', ['worth-relational','worth-court-relational-certification'], {}),
 'consumer': ('workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/Cargo.toml',
  ['worth-query-consumer-root','worth-query-topology-entry'], {}),
 'query': ('workspaces/worth-query/Cargo.toml',
  ['worth-query-court-query-decl-host'], {'test':['worth_query_court_query_decl_host']}),
 'store': ('workspaces/worth-store/Cargo.toml',['worth-store-court-application'],{'test':['worth_store_court_application']}),
}[kind]
manifest = root / manifest
environment = application_build_environment(child_environment())
runner = CargoRunner(root, environment)
reuse = build(manifest, out / 'build', runner, selection={'packages': packages, 'targets': targets})
expression = ' | '.join('test(' + name + ')' for name in cases)
selected = [row for row in inventory(list_tests(reuse, out / 'selection.json', runner, expression)) if row['selected']]
assert len(selected) == len(cases) and not any(row['ignored'] for row in selected), selected
assert {row['test'].rsplit('::', 1)[-1] for row in selected} == set(cases), selected
direct = 'same_directory_reopens_reads_and_commits_without_replaying_initializer_or_old_key'
ordinary = [row for row in selected if kind != 'query' or row['test'].rsplit('::', 1)[-1] != direct]
# Root includes a caller and two scoped registry threads. Other packets use
# the installed fixture caller plus its one actual owned worker.
slots = 3 if kind == 'root' else 2
reservations = {row['binary'] + '\n' + row['test']: slots for row in selected}
(out / 'reservations.json').write_text(json.dumps(reservations, indent=2) + '\n', encoding='utf-8')
environment.update(WORTH_REPO=str(root), WORTH_TEST_COST_RESERVATIONS=str(out / 'reservations.json'),
                   WORTH_TEST_COST_ACCOUNTING_DIRECTORY=str(out / 'cpu'),
                   WORTH_TEST_COST_SETUP_DIRECTORY=str(out / 'setup'))
config = manifest.parent / '.config/nextest.toml'
if not config.exists(): config = root / '.config/nextest.toml'
configuration(config, out / 'nextest.toml', 'ci', reservations, root / 'scripts/test_cost/account.py')
exact = ' | '.join('test(=' + row['test'] + ')' for row in ordinary)
observed = runner(['cargo', 'nextest', 'run', *reuse, '--profile', 'ci', '--config-file', str(out / 'nextest.toml'),
                   '--ignore-default-filter', '--no-fail-fast', '--no-tests=fail',
                   '--message-format', 'libtest-json', '-E', exact], manifest.parent, out / 'run.log')
counts = [0, 0, 0]
for event in json_objects(observed.stdout):
    if event.get('type') == 'test' and event.get('event') in ('ok', 'failed'):
        counts[0 if event['event'] == 'ok' else 1] += 1
    if event.get('type') == 'test' and event.get('event') == 'failed':
        print(event.get('stdout', ''), flush=True)
print(observed.stdout)
code = observed.returncode
if kind == 'query' and any(row['test'].rsplit('::', 1)[-1] == direct for row in selected):
    row = next(row for row in selected if row['test'].rsplit('::', 1)[-1] == direct)
    binaries = json.loads((out / 'build/binaries.json').read_text(encoding='utf-8'))
    binary = Path(binaries['rust-binaries'][row['binary']]['binary-path']).resolve()
    assert binary.is_relative_to(root)
    child = dict(environment)
    child.pop('RUST_MIN_STACK', None)
    command = [str(binary), '--exact', row['test'], '--nocapture']
    print('DIRECT COMMAND:', subprocess.list2cmdline(command), flush=True)
    print('RUST_MIN_STACK present:', 'RUST_MIN_STACK' in child, flush=True)
    result = subprocess.run(command, env=child, capture_output=True)
    output = result.stdout.decode('utf-8', errors='replace') + result.stderr.decode('utf-8', errors='replace')
    print(output, flush=True)
    (out / 'direct-run.log').write_text(output, encoding='utf-8')
    summary = re.search(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;', output)
    direct_counts = list(map(int, summary.groups())) if summary else [0, 1, 0]
    counts = [a + b for a, b in zip(counts, direct_counts)]
    code = code or result.returncode
for path in runner.logs: print('FULL_OWNED_LOG_PATH:', path)
print('EVIDENCE_TEST_COUNTS: ' + ','.join(map(str, counts)), flush=True)
sys.exit(code)
