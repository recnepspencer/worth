"""Retain measured scheduling policy in small, generated per-binary shards."""
import hashlib
import json
from pathlib import Path

from telemetry import slot_use


def load(directory):
    directory = Path(directory).resolve()
    manifest = directory / 'index.json'
    if not manifest.exists():
        return {}
    value = json.loads(manifest.read_text(encoding='utf-8'))
    if value['schema'] != 1:
        raise ValueError('unsupported measured-reservation schema')
    result = {}
    for name in value['files']:
        path = (directory / name).resolve()
        if path.parent != directory:
            raise ValueError('reservation shard escapes its owner')
        shard = json.loads(path.read_text(encoding='utf-8'))
        for test, slots in shard['tests'].items():
            identity = shard['binary'] + '\n' + test
            if identity in result:
                raise ValueError('reservation identity repeats across generated shards')
            result[identity] = slots
    return result


def retain(directory, events, head, log, baseline=None, known=None):
    directory = Path(directory).resolve()
    observation = slot_use([event for event in events if event.get('exit', 0) == 0])
    if not observation['tests']:
        return {'status': 'unmeasured'}
    policy = load(directory)
    if known is not None:
        policy = {key: slots for key, slots in policy.items() if key in known}
    peaks, actual = {}, {}
    for row in observation['tests']:
        key = row['binary'] + '\n' + row['test']
        peaks[key] = max(peaks.get(key, 1), row['recommended_slots'])
        actual[key] = row['reserved_slots']
    for key, slots in peaks.items():
        declared = baseline.get(key, actual[key]) if baseline is not None else actual[key]
        if slots != declared or slots > 1:
            policy[key] = slots
        else:
            policy.pop(key, None)
    by_binary = {}
    for key, slots in sorted(policy.items()):
        binary, test = key.split('\n', 1)
        by_binary.setdefault(binary, {})[test] = slots
    directory.mkdir(parents=True, exist_ok=True)
    previous = directory / 'index.json'
    old = json.loads(previous.read_text()) if previous.exists() else {}
    old_files = old.get('files', [])
    files = []
    for binary, tests in sorted(by_binary.items()):
        digest = hashlib.sha256(binary.encode()).hexdigest()[:20]
        rows = sorted(tests.items())
        for offset in range(0, len(rows), 100):
            name = f'{digest}-{offset // 100:03}.json'
            files.append(name)
            (directory / name).write_text(json.dumps({'binary': binary, 'tests': dict(rows[offset:offset + 100])},
                                                    indent=2) + '\n', encoding='utf-8')
    provenance = old.get('observations', []) if old_files else []
    current = {'head': head, 'telemetry_log': str(log)}
    if current not in provenance:
        provenance.append(current)
    value = {'schema': 1, 'observations': provenance,
             'status': 'partial' if observation['unmeasured'] else 'measured',
             'complete_samples': len(observation['tests']), 'unmeasured_samples': len(observation['unmeasured']),
             'files': files}
    previous.write_text(json.dumps(value, indent=2) + '\n', encoding='utf-8')
    for name in sorted(set(old_files) - set(files)):
        path = (directory / name).resolve()
        if path.parent != directory or not path.name.endswith('.json'):
            raise ValueError('old reservation shard escapes its owner')
        path.unlink(missing_ok=True)
    return value
