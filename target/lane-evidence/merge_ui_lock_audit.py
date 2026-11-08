"""Verify the boundary metadata's UI lock propagation is resolution-only."""
from pathlib import Path
import json
import subprocess
import tomllib

root = Path.cwd().resolve()
path = 'workspaces/worth-ui/Cargo.lock'
old_bytes = subprocess.check_output(['git', 'show', ':' + path])
new_bytes = (root / path).read_bytes()
key = lambda row: (row['name'], row['version'], row.get('source', ''))
old = {key(row): row for row in tomllib.loads(old_bytes.decode())['package']}
new = {key(row): row for row in tomllib.loads(new_bytes.decode())['package']}
removed = sorted(set(old) - set(new))
modified = sorted(k for k in old.keys() & new.keys() if old[k] != new[k])
added = sorted(set(new) - set(old))
assert not removed, removed
assert all(not k[2] and k[0].startswith('worth-') for k in modified), modified
def normalized(row, packages):
    result = dict(row)
    if 'dependencies' not in result:
        return result
    resolved = []
    for edge in result['dependencies']:
        parts = edge.split(' ')
        choices = [k for k in packages if k[0] == parts[0]
                   and (len(parts) == 1 or k[1] == parts[1])]
        assert len(choices) == 1, (edge, choices)
        resolved.append(choices[0])
    result['dependencies'] = sorted(resolved)
    return result

known = {}
for lock in ['Cargo.lock', 'workspaces/worth-query/Cargo.lock', 'workspaces/worth-store/Cargo.lock']:
    packages = {key(row): row for row in tomllib.loads((root / lock).read_text(encoding='utf-8'))['package']}
    for k, row in packages.items():
        known[k] = normalized(row, packages)
for k in added:
    assert k in known, k
    assert new[k].get('checksum') == known[k].get('checksum'), k
    if k[2]:
        assert normalized(new[k], new) == known[k], (k, 'external resolution differs')
record = {'removed_packages': removed, 'modified_existing_workspace_packages': modified,
          'added_packages_already_resolved_in_qualified_workspaces': added,
          'external_versions_and_checksums_unchanged_or_reused_exactly': True}
(root / 'target/lane-evidence/funding-merge-ui-lock-audit.json').write_text(
    json.dumps(record, indent=2) + '\n', encoding='utf-8')
print('Existing packages removed:', len(removed))
print('Existing workspace dependency rows updated:', [k[0] for k in modified])
print('Added packages already in qualified workspace locks:', len(added))
print('All added external resolutions exactly match qualified locks.')
