"""Generate nextest reservations from observed CPU use; timings never gate."""
import json
import math
from pathlib import Path
import sys
import tomllib

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'test_topology'))
from coverage import inventory
from telemetry import slot_use
from manifest_edit import toml_value


def recommended(events):
    result = {}
    for event in slot_use(events)['tests']:
        identity = event['binary'] + '\n' + event['test']
        result[identity] = max(result.get(identity, 1), event['recommended_slots'])
    return result


def configuration(original, output, profile, recommendations, wrapper):
    value = tomllib.loads(Path(original).read_text(encoding='utf-8')) if Path(original).exists() else {}
    value['experimental'] = sorted(set(value.get('experimental', [])) | {'wrapper-scripts'})
    value.setdefault('scripts', {}).setdefault('wrapper', {})['test-cost'] = {
        'command': [sys.executable, str(Path(wrapper).resolve())]}
    profiles = value.setdefault('profile', {})
    selected = profiles.setdefault(profile, {})
    selected['scripts'] = [{'filter': 'all()', 'run-wrapper': 'test-cost'}] + selected.get('scripts', [])
    overrides = []
    for identity, slots in sorted(recommendations.items()):
        binary, test = identity.split('\n', 1)
        # Exact match strings are filter-language arguments, not regexes.
        if any(character in binary + test for character in ('(', ')', '\n')):
            raise ValueError(f'unsupported exact filter identity: {identity}')
        overrides.append({'filter': f'binary_id(={binary}) & test(={test})', 'threads-required': slots})
    selected['overrides'] = overrides + selected.get('overrides', [])
    # All tables can be expressed as inline TOML tables. Preserve every timeout,
    # group and original override, including order and inherited profile rules.
    Path(output).write_text('\n'.join(f'{json.dumps(key)} = {toml_value(item)}'
                                     for key, item in value.items()) + '\n', encoding='utf-8')
    return value


def original_reservations(config, profile, discovered, select):
    """Ask nextest to evaluate its own filters, in its documented first-match order."""
    value = tomllib.loads(Path(config).read_text(encoding='utf-8')) if Path(config).exists() else {}
    profiles = value.get('profile', {})
    selected = profiles.get(profile, {})
    default = profiles.get('default', {})
    baseline = selected.get('threads-required', default.get('threads-required', 1))
    result = {row['binary'] + '\n' + row['test']: baseline for row in inventory(discovered)}
    seen = set()
    overrides = selected.get('overrides', []) + ([] if profile == 'default' else default.get('overrides', []))
    for index, override in enumerate(overrides):
        if 'threads-required' not in override:
            continue
        matches = inventory(select(index, override['filter']))
        for row in matches:
            key = row['binary'] + '\n' + row['test']
            if row.get('selected', True) and key not in seen:
                result[key] = override['threads-required']
                seen.add(key)
    return result
