"""Stage this merge's exact evidence and every referenced full owned log."""
from pathlib import Path
import re
import subprocess

root = Path.cwd().resolve()
directory = root / 'target/lane-evidence'
paths = set(directory.glob('funding-merge*'))
paths.update(directory.glob('merge-funding*'))
for name in [
    'merge-14b16-plan.md', 'merge_workspace_compile.py', 'merge_funding_proofs.py',
    'resolve-funding-source-merge.py', 'finish_funding_merge_checks.py',
    'funding_merge_source_identity.py', 'funding_merge_boundary_handoff.py',
    'funding_merge_commit_identity.py',
    'merge_ui_lock_audit.py', 'write_funding_merge_lane_index.py',
    'stage_funding_merge_evidence.py', 'p3-reopen-progress.md', 'turn-end-state.md',
]:
    paths.add(directory / name)
for path in list(paths):
    if path.suffix != '.log':
        continue
    for linked in re.findall(r'^FULL_OWNED_LOG_PATH: (.+)$', path.read_text(encoding='utf-8'), re.M):
        paths.add(Path(linked.strip()).resolve())
for path in paths:
    assert path.is_file() and path.resolve().is_relative_to(directory), path
    path.read_text(encoding='utf-8')
relative = sorted(str(path.relative_to(root)).replace('\\', '/') for path in paths)
for start in range(0, len(relative), 60):
    subprocess.run(['git', 'add', '-f', '--', *relative[start:start + 60]], check=True)
print('Staged current-turn UTF-8 evidence files:', len(relative))
