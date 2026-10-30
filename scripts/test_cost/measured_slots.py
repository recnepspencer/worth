"""Load and retain measured scheduling policy through bounded catalogs."""
from pathlib import Path

from logs import read_lane
from policy_catalog import index as read_index, read as read_catalog
from slot_policy import generate


def load(directory, known=None):
    """Select current rules without migrating or discarding historical samples."""
    directory = Path(directory).resolve()
    value = read_index(directory)
    if not value:
        return {}
    result = {}
    for row in read_catalog(directory, value['catalogs']['rules']):
        identity = row['binary'] + '\n' + row['test']
        if identity in result or type(row['slots']) is not int or row['slots'] < 1:
            raise ValueError('invalid or repeated reservation identity in its catalog')
        result[identity] = row['slots']
    return result if known is None else {key: slots for key, slots in result.items() if key in known}


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
