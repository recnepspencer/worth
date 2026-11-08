"""Index actual invocation headers; unit-graph inventories are not compilations."""
from pathlib import Path
import collections
import datetime
import json
import subprocess

r = Path.cwd()
e = r / 'target/lane-evidence'
commit_time = subprocess.check_output(['git', 'show', '-s', '--format=%cI', 'e787b51b3e'], text=True).strip()
cutoff = datetime.datetime.fromisoformat(commit_time)
files = set(e.glob('pc-merge-*.log')) | set(e.glob('lane-evidence-pc-merge-*.log'))
files |= set(e.glob('execution-stack-*.log')) | set(e.glob('lane-evidence-execution-stack-*.log'))
for directory in [*e.glob('pc-merge-*'), *e.glob('execution-stack-*')]:
    if directory.is_dir():
        files.update(directory.rglob('*.log'))
records = {}
for path in sorted(files):
    start = head = None
    with path.open(encoding='utf-8', errors='replace') as stream:
        for line in stream:
            if line.startswith(('Start: ', 'START: ')):
                start = line.split(': ', 1)[1].strip()
            elif line.startswith('HEAD: '):
                head = line[6:].strip()
            elif line.startswith('{"type": "cargo-invocation"'):
                value = json.loads(line)
                command = value['command']
                key = (start, json.dumps(command), value.get('cwd'))
                record = records.setdefault(key, {'start': start, 'head': head, 'command': command,
                    'cwd': value.get('cwd'), 'logs': []})
                record['logs'].append(path.relative_to(r).as_posix())
                break
counts = collections.Counter()
post_merge = collections.Counter()
for record in records.values():
    command = record['command']
    kind = 'native-execution'
    if command[:1] == ['cargo']:
        verb = command[1]
        if '--unit-graph' in command:
            kind = 'unit-graph-inventory'
        elif verb == 'nextest':
            kind = 'focused-or-broad-build' if '--list-type' in command else 'nextest-' + command[2]
        elif verb == 'test':
            kind = ('cargo-build-test-artifacts' if '--no-run' in command else
                    'documentation-test' if '--doc' in command else 'cargo-test')
        else:
            kind = 'cargo-' + verb
    elif '--list' in command:
        kind = 'native-list'
    elif '--exact' in command:
        kind = 'native-test'
    elif any('phase_telemetry.py' in a for a in command):
        kind = 'telemetry-process'
    else:
        kind = 'native-control'
    record['kind'] = kind
    if kind in ('cargo-build','cargo-build-test-artifacts','focused-or-broad-build'):
        record['build_scope'] = 'tier2-or-prerequisite' if any('tier2' in name for name in record['logs']) else 'focused'
    counts[kind] += 1
    if record['start'] and datetime.datetime.fromisoformat(record['start']) >= cutoff:
        post_merge[kind] += 1
payload = {'head': subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip(),
           'merge_commit_time': commit_time, 'whole_merge_episode': counts,
           'after_merge_commit': post_merge, 'invocations': list(records.values()),
           'legacy_fixture_body_chunks': len(list((e / 'pc-merge-fixture-tier2').glob('fixture-chunk-*.log'))),
           'note': 'Build-only cargo test --no-run, list audits, telemetry and body runs are separate. Legacy fixture chunks are body executions recorded by their parent; do not invent child timestamps. Counts include started invocations; inspect exit logs for completion.'}
post = [row for row in records.values() if row['start'] and datetime.datetime.fromisoformat(row['start']) >= cutoff]
payload['post_checkpoint_build_scopes'] = collections.Counter(row['build_scope'] for row in post if 'build_scope' in row)
payload['post_checkpoint_harness_invocations'] = sum(post_merge[key] for key in ['nextest-run','native-test','cargo-test','documentation-test']) + payload['legacy_fixture_body_chunks']
payload['post_checkpoint_focused_test_invocations'] = sum(1 for row in post if row['kind']=='native-test' or (row['kind']=='nextest-run' and not any('tier2' in path for path in row['logs'])))
(e / 'pc-merge-operation-census.json').write_text(json.dumps(payload, indent=2))
print(json.dumps({'whole_merge_episode': counts, 'after_merge_commit': post_merge}, indent=2))
