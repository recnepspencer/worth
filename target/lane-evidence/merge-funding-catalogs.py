"""Merge immutable scheduling records, then use their owner to regenerate shards."""
import io
import json
from pathlib import Path
import subprocess
import sys
import tarfile

root = Path.cwd().resolve()
sys.path.insert(0, str(root / 'scripts/test_cost'))
import policy_catalog as catalog
import slot_policy

other = '14b16fa292'
base = subprocess.check_output(['git', 'merge-base', 'HEAD', other], text=True).strip()

def archive(ref):
    raw = subprocess.check_output(['git', 'archive', ref, 'scripts/test_cost/slots'])
    with tarfile.open(fileobj=io.BytesIO(raw)) as tar:
        return {member.name: tar.extractfile(member).read()
                for member in tar.getmembers() if member.isfile()}

trees = [archive(ref) for ref in (base, 'HEAD', other)]

def rows(tree, directory, descriptor):
    raw = tree[directory + '/' + descriptor['file']]
    assert catalog.digest(raw) == descriptor['sha256']
    header, *entries = map(json.loads, raw.splitlines())
    assert header['count'] == len(entries)
    if header['kind'] == 'leaf':
        return entries
    return [row for child in entries for row in rows(tree, directory, child)]

def key(collection, row):
    if collection == 'sources': return row['sha256']
    if collection == 'samples': return row['capture_sha256'], row['sample_index']
    if collection == 'pending': return row['claim']
    return row['binary'], row['test']

report = {}
for workspace in ('root', 'store'):
    directory = 'scripts/test_cost/slots/' + workspace
    indexes = [json.loads(tree[directory + '/index.json']) for tree in trees]
    merged = {}
    for collection in catalog.COLLECTIONS:
        maps = [{key(collection, row): row for row in rows(tree, directory, index['catalogs'][collection])}
                for tree, index in zip(trees, indexes)]
        result = {}
        for identity in sorted(set(maps[1]) | set(maps[2])):
            before, ours, theirs = [entries.get(identity) for entries in maps]
            if collection in ('sources', 'samples'):
                if ours and theirs:
                    assert ours == theirs, (collection, identity)
                selected = ours or theirs
            elif ours == theirs: selected = ours
            elif ours == before: selected = theirs
            elif theirs == before: selected = ours
            else:
                raise ValueError(('conflicting logical policy record', workspace, collection, identity))
            if selected is not None: result[identity] = selected
        merged[collection] = list(result.values())
    owned = root / directory
    seed = dict(indexes[2], catalogs={collection: catalog.write(owned, collection, entries)
                                    for collection, entries in merged.items()})
    (owned / 'index.json').write_text(json.dumps(seed, indent=2) + '\n', encoding='utf-8')
    result = slot_policy.generate(owned, [])
    retained = set().union(*(catalog.files(owned, value) for value in result['catalogs'].values()))
    for path in owned.glob('*.jsonl'):
        assert path.resolve().parent == owned.resolve() and path.suffix == '.jsonl'
        if path.name not in retained: path.unlink()
    report[workspace] = {name: result[name] for name in ('sample_count', 'source_count', 'measured_identities', 'pending_claim_count')}
    report[workspace]['parent_sample_counts'] = [index['sample_count'] for index in indexes[1:]]
print(json.dumps(report, indent=2))
(root / 'target/lane-evidence/merge-funding-catalogs.json').write_text(
    json.dumps(report, indent=2) + '\n', encoding='utf-8')
