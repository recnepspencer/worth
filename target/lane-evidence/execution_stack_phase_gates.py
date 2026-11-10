"""Scoped lint and phase gates after the single public-entry verification round."""
from pathlib import Path
import json
import subprocess
import sys

r = Path.cwd()
e = r / 'target/lane-evidence'
py = sys.executable
# All selected functional outcomes must be recorded before phase gates. A held
# protected-owner refusal is reported as red, never converted into a pass.
actual = json.loads((e / 'execution-stack-round2-verification/results.json').read_text())
observed = {row['lane']: row['exit'] for row in actual}
for critical in ['consumer-mains-build','invariant_reads-main','required_feature-main',
                 'worth-query-consumer-root-main','bank-tests']:
    assert observed.get(critical) == 0, (critical, observed)
# The sole held genuine-main refusal must remain the recorded checkpoint failure.
if observed.get('shared_execution-main'):
    assert observed['shared_execution-main'] == 101
    raw = (e/'execution-stack-round2-verification/shared_execution-main.log').read_text(encoding='utf8')
    assert 'NestedStopped' in raw and 'shared_execution.rs:62' in raw
assert json.loads((e / 'execution-stack-wasm-result.json').read_text())['exit'] == 0
results = []

def lane(label, *command):
    code = subprocess.call([py, str(e / 'run_evidence.py'), label, *command])
    results.append([label, code])
    (e / 'execution-stack-phase-results.json').write_text(json.dumps(results, indent=2))
    return code

for label, manifest, packages in [
    ('root', 'Cargo.toml', ['worth-execution','worth-relational','worth-runtime-bridge','worth-signal']),
    ('query', 'workspaces/worth-query/Cargo.toml', ['worth-query','worth-query-execution','worth-query-host']),
    ('store', 'workspaces/worth-store/Cargo.toml', ['worth-store-court-application','worth-store-court-recovery-runtime-replication']),
    ('fixture', 'workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/Cargo.toml', ['worth-query-consumer-root']),
]:
    lane('execution-stack-'+label+'-fmt', 'cargo','fmt','--manifest-path',manifest,
         *[item for package in packages for item in ('-p',package)], '--','--check')

# The Native dead-code exception is the existing committed owner-service policy
# in scripts/ci/check_owner_service_clippy.sh. Full strict evidence stays red.
for workspace, manifest, packages, kinds, exception in [
    ('root', 'Cargo.toml', ['worth-execution', 'worth-relational', 'worth-runtime-bridge', 'worth-signal'],
     ['--lib', '--tests'], ['-A', 'dead-code']),
    ('query', 'workspaces/worth-query/Cargo.toml', ['worth-query-execution', 'worth-query-host', 'worth-query'],
     ['--lib', '--tests'], []),
    ('store', 'workspaces/worth-store/Cargo.toml', ['worth-store-court-application', 'worth-store-court-recovery-runtime-replication'],
     ['--all-targets'], []),
    ('fixture', 'workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/Cargo.toml',
     ['worth-query-topology-entry', 'worth-query-parameter-entry', 'worth-query-consumer-root'], ['--all-targets'], []),
]:
    lane('execution-stack-' + workspace + '-lint-driver', py, str(e / 'pa_merge_owned_cargo.py'),
         'execution-stack-' + workspace + '-lint', manifest, 'clippy',
         *[item for package in packages for item in ('-p', package)],
         *kinds, '--keep-going', '--', '-D', 'warnings', *exception)

lane('execution-stack-line-caps', 'C:/Program Files/Git/bin/bash.exe',
     'scripts/ci/check_workspace_rust_line_caps.sh', 'dirty')
lane('execution-stack-lock-alignment', py, 'scripts/ci/check_lockfile_alignment.py')
lane('execution-stack-boundary-driver', py, str(e / 'pa_merge_owned_cargo.py'),
     'execution-stack-boundary', 'tools/boundary-check/Cargo.toml', 'run',
     '--target-dir', str(r / 'target/constitution-tools'), '--', '--root', str(r))
lane('execution-stack-context-driver', py, str(e / 'pa_merge_owned_cargo.py'),
     'execution-stack-context', 'tools/agent-context/Cargo.toml', 'run',
     '--target-dir', str(r / 'target/constitution-tools'), '--', 'check')
raise SystemExit(int(any(code for _, code in results)))
