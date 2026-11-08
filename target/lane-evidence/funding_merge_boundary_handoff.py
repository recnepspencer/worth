"""Record exact unresolved enforcement sites and their owner-change evidence."""
from pathlib import Path
import json
import subprocess

root = Path.cwd().resolve()
def git(*args):
    result = subprocess.run(['git', *args], cwd=root, capture_output=True, text=True)
    return {'command': ['git', *args], 'exit_code': result.returncode,
            'stdout': result.stdout.strip(), 'stderr': result.stderr.strip()}

base = '7d0e5189bc'
incoming = '14b16fa292'
ours = '25619ac5d1'
lifecycle = 'codex/m3-p3-lifecycle'
lines = [line for line in (root / 'target/lane-evidence/funding-merge-boundary.log')
         .read_text(encoding='utf-8').splitlines() if line.startswith('BC')]
rows = []
for line in lines:
    path = line.split(' ', 2)[1].split('::', 1)[0].rstrip(':')
    owner = ('integration/Bank' if '/worth-query-bank-world/' in path
             else 'P-A queued migration' if path in {
                 'crates/worth-relational/src/tests/durability/medium/cold_source.rs',
                 'crates/worth-relational/src/tests/durability/medium/cold_source/protected_reads.rs',
                 'crates/worth-relational/src/tests/support/durable_medium/runtime_attachment.rs',
                 'crates/worth-relational/src/tests/durability/contracts/native_checkpoint/allocation/encoded_payload.rs',
                 'crates/worth-relational/src/tests/durability/contracts/semantic_result/execution_posture.rs',
                 'crates/worth-relational/src/runtime/state/subsystems/record_identity/partition_frontier/tests.rs',
             } else 'Lifecycle/integration handoff required')
    row = {'finding': line, 'path': path, 'owner': owner,
           'ours_blob': git('rev-parse', ours + ':' + path),
           'incoming_blob': git('rev-parse', incoming + ':' + path),
           'current_blob': git('hash-object', '--path=' + path, path),
           'lifecycle_changed_since_safe_point': git('diff', '--quiet', base, lifecycle, '--', path)}
    rows.append(row)
    print(line.split(': {', 1)[0])
    print('  disposition:', owner,
          '; Lifecycle quiet-diff exit:', row['lifecycle_changed_since_safe_point']['exit_code'])
record = {'head': git('rev-parse', 'HEAD'), 'lifecycle_ref': git('rev-parse', lifecycle),
          'findings': rows,
          'constructor_caller_diff_from_incoming': git('diff', incoming, '--',
              'tools/boundary-check/config/certification-construction-callers.txt')}
(root / 'target/lane-evidence/funding-merge-boundary-handoff.json').write_text(
    json.dumps(record, indent=2) + '\n', encoding='utf-8')
print('Total unresolved:', len(rows))
