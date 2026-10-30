"""Retain measured scheduling policy in small, generated per-binary shards."""
import json
from pathlib import Path

from logs import read_lane
from slot_policy import generate


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
    """Retain authoritative log samples; known must be the full current inventory."""
    log = Path(log).resolve()
    events = list(events)
    if read_lane(log)['cpu'] != events:
        raise ValueError('CPU events differ from the authoritative retained telemetry log')
    result = generate(directory, [log], baseline=baseline, known=known,
                      execution_heads={str(log): head})
    result['latest_observation'] = {'status': 'observed' if events else 'unmeasured', 'samples': len(events)}
    return result
