"""One serial check pass for the complete public entry-point batch."""
from pathlib import Path
import json
import subprocess
import sys

r = Path.cwd()
e = r / 'target/lane-evidence'
py = sys.executable
checks = {}
manifests = {
    'root': 'Cargo.toml',
    'query': 'workspaces/worth-query/Cargo.toml',
    'store': 'workspaces/worth-store/Cargo.toml',
    'bank': 'workspaces/worth-query-bank-world/Cargo.toml',
    'fixture': 'workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/Cargo.toml',
}
for workspace, manifest in manifests.items():
    selection = ['--workspace'] if workspace != 'fixture' else [
        item for package in ('worth-query-topology-entry', 'worth-query-parameter-entry', 'worth-query-consumer-root')
        for item in ('-p', package)]
    checks[workspace] = subprocess.call([py, str(e / 'run_evidence.py'),
        'execution-stack-' + workspace + '-check-driver', py, str(e / 'pa_merge_owned_cargo.py'),
        'execution-stack-' + workspace + '-check', manifest, 'check',
        *selection, '--tests', '--message-format=json', '--keep-going'])
    (e / 'execution-stack-check-results.json').write_text(json.dumps(checks, indent=2))
    print('RESULT', workspace, checks[workspace], flush=True)
    if checks[workspace]:
        raise SystemExit('Stop dependent checks; fix the complete diagnostic batch first')
if not any(checks.values()):
    wasm = subprocess.call([py, str(e / 'run_evidence.py'), 'execution-stack-wasm-check-driver',
        py, str(e / 'pa_merge_owned_cargo.py'), 'execution-stack-wasm-check', 'Cargo.toml',
        'check', '-p', 'worth-signal-wasm', '--target', 'wasm32-unknown-unknown', '--message-format=json'])
    (e / 'execution-stack-wasm-result.json').write_text(json.dumps({'exit': wasm}, indent=2))
raise SystemExit(int(any(checks.values()) or ('wasm' in locals() and wasm)))
