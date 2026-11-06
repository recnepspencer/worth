"""Sequential post-review validation; never overlap workspace builds."""
from pathlib import Path
import subprocess
import sys

root = Path.cwd().resolve()
capture = root / 'target/lane-evidence/capture_lane.py'
def lane(name, *command):
    return subprocess.run([sys.executable, str(capture), name, *command], cwd=root).returncode

for name, command in [
    ('funding-merge-root-clippy-04', [sys.executable, 'target/lane-evidence/merge_workspace_compile.py', 'root']),
    ('funding-merge-root-review', [sys.executable, 'target/lane-evidence/merge_funding_proofs.py', 'root-review']),
    ('funding-merge-boundary', ['cargo', 'run', '--manifest-path', 'tools/boundary-check/Cargo.toml', '--', '--root', '.']),
    ('funding-merge-agent-context', ['cargo', 'run', '--manifest-path', 'tools/agent-context/Cargo.toml', '--', 'check']),
    ('funding-merge-final-line-caps', ['C:/Program Files/Git/bin/bash.exe', 'scripts/ci/check_workspace_rust_line_caps.sh', 'dirty']),
]:
    code = lane(name, *command)
    if name == 'funding-merge-boundary' and code == 1:
        # This ordered task lands the merge before its six queued BC9002 sites.
        # Keep the raw failure. Every other boundary code/finding blocks here.
        output = (root / 'target/lane-evidence/funding-merge-boundary.log').read_text(encoding='utf-8')
        findings = [line for line in output.splitlines() if line.startswith('BC')]
        expected = {
            'crates/worth-relational/src/tests/durability/medium/cold_source.rs',
            'crates/worth-relational/src/tests/durability/medium/cold_source/protected_reads.rs',
            'crates/worth-relational/src/tests/support/durable_medium/runtime_attachment.rs',
            'crates/worth-relational/src/tests/durability/contracts/native_checkpoint/allocation/encoded_payload.rs',
            'crates/worth-relational/src/tests/durability/contracts/semantic_result/execution_posture.rs',
            'crates/worth-relational/src/runtime/state/subsystems/record_identity/partition_frontier/tests.rs',
        }
        actual = {line.split(' ', 2)[1].split('::', 1)[0] for line in findings}
        if (findings and actual <= expected
                and all(line.startswith('BC9002_TEST_TOPOLOGY ') for line in findings)):
            print('Queued boundary findings retained:', len(findings), flush=True)
            continue
    if code:
        sys.exit(code)
