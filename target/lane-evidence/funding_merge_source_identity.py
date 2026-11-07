"""Bind dirty-tree lane evidence to the reviewed merge's exact source blobs."""
from pathlib import Path
import hashlib
import json
import subprocess

root = Path.cwd().resolve()
def git(*args):
    return subprocess.check_output(['git', *args], cwd=root, text=True).strip()

tree = git('write-tree')
compile_tree = 'c7e6413c053ef82c98dbccf8bfb3bdad930c6bde'
changed = git('diff', '--name-only', compile_tree, tree, '--',
              '*.rs', '*.toml', '*.json', '*.txt', '*.py', '*.sh').splitlines()
changed = [path for path in changed if not path.startswith('target/')]
expected = {
    'crates/worth-relational/src/durability/authority/checkpoint_complete/source.rs',
    'crates/worth-relational/src/durability/authority/checkpoint_complete/source/tests.rs',
}
assert set(changed) == expected, changed
assert git('diff', '--name-only', '--', '*.rs', '*.toml', '*.json', '*.txt', '*.py', '*.sh') == ''
record = {
    'head': git('rev-parse', 'HEAD'),
    'merge_head': git('rev-parse', 'MERGE_HEAD'),
    'compilation_tree': compile_tree,
    'reviewed_staged_tree': tree,
    'only_source_changes_after_initial_compilation': changed,
    'source_blobs': {
        path: {'blob': git('rev-parse', tree + ':' + path),
               'sha256_working_bytes': hashlib.sha256((root / path).read_bytes()).hexdigest()}
        for path in changed
    },
    'unstaged_source_empty': True,
    'generated_lock_propagation': json.loads((root / 'target/lane-evidence/funding-merge-ui-lock-audit.json').read_text(encoding='utf-8')),
    'ui_lock_blob': git('rev-parse', tree + ':workspaces/worth-ui/Cargo.lock'),
}
(root / 'target/lane-evidence/funding-merge-source-identity.json').write_text(
    json.dumps(record, indent=2) + '\n', encoding='utf-8')
print(json.dumps(record, indent=2))
