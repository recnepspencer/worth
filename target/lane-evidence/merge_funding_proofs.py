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
label = 'funding-merge-' + kind
out = root / 'target/test-cost' / label
out.mkdir(parents=True, exist_ok=True)
cases = {
    'root-review': ['complete_original_frame_content_denials_preserve_their_semantic_and_request_causes'],
    'query-extra': [
        'automatic_checkpoint_exhaustion_keeps_business_results_and_explicit_capture_uses_its_own_allowance',
        'automatic_checkpoint_progresses_after_caller_cancellation_without_replacing_the_body_value',
        'live_checkpoint_recapture_keeps_all_covered_canonical_bodies_cold',
        'automatic_capture_at_head_forty_one_preserves_ordinary_request_results',
        'close_checkpoint_exhaustion_reports_failure_and_preserves_journal_reopen'],
    'root-courts': prior_cases('funding-controls-native') + [
        'recovered_nonhead_envelope_custody_ends_with_the_last_inflight_read'],
    'root-units': prior_cases('funding-final-cdc-recovery') + [
        'inline_bytes_clone_quote_counts_length_without_spare_capacity'],
    'query': prior_cases('funding-final-host') + prior_cases('funding-final-bounded') + [
        'initializer_refusal_reopens_at_missing_world_origin_without_replaying_initializer',
        'killed_home_reopens_normalized_image_plus_nonempty_tail',
        'journal_only_killed_home_reopens_original_inputs_and_commits_without_initializing'],
    'store': prior_cases('funding-final-store'),
}[kind]
assert len(cases) == len(set(cases))
manifest, packages, targets = {
    'root-review': ('Cargo.toml', ['worth-relational'], {'lib': True}),
    'query-extra': ('workspaces/worth-query/Cargo.toml', ['worth-query-court-query-decl-host'],
                    {'test': ['worth_query_court_query_decl_host']}),
    'root-courts': ('Cargo.toml', ['worth-court-relational-certification'],
                    {'test': ['worth_court_relational_certification']}),
    'root-units': ('Cargo.toml', ['worth-relational', 'worth-foundational'], {'lib': True}),
    'query': ('workspaces/worth-query/Cargo.toml', ['worth-query-court-query-decl-host'],
              {'test': ['worth_query_court_query_decl_host']}),
    'store': ('workspaces/worth-store/Cargo.toml', ['worth-store-court-application'],
              {'test': ['worth_store_court_application']}),
}[kind]
manifest = root / manifest
environment = application_build_environment(child_environment())
runner = CargoRunner(root, environment)
if kind == 'query-extra':
    prior = root / 'target/test-cost/funding-merge-query/build'
    reuse = ['--cargo-metadata', str(prior / 'cargo-metadata.json'),
             '--binaries-metadata', str(prior / 'binaries.json')]
    print('REUSED_HOST_BINARY_SOURCE_TREE: c7e6413c053ef82c98dbccf8bfb3bdad930c6bde', flush=True)
else:
    reuse = build(manifest, out / 'build', runner, selection={'packages': packages, 'targets': targets})
expression = ' | '.join('test(' + name + ')' for name in cases)
selected = [row for row in inventory(list_tests(reuse, out / 'selection.json', runner, expression)) if row['selected']]
assert len(selected) == len(cases) and not any(row['ignored'] for row in selected), selected
assert {row['test'].rsplit('::', 1)[-1] for row in selected} == set(cases), selected
direct = 'same_directory_reopens_reads_and_commits_without_replaying_initializer_or_old_key'
ordinary = [row for row in selected if kind != 'query' or row['test'].rsplit('::', 1)[-1] != direct]
# Every Native control has one Serial caller, no joined worker. Both the Host
# and bare Store Home join one independent maintenance worker: caller 1+worker 1.
slots = 2 if kind in ('query', 'query-extra', 'store') else 1
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
if kind == 'query':
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
