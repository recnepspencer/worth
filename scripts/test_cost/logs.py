"""Cargo artifacts and libtest/nextest observations from retained lane logs."""
from collections import defaultdict
from datetime import datetime
import json
from pathlib import Path, PurePosixPath
import re


def json_objects(text):
    """Accept JSONL amidst progress output; inspect captured test output as well."""
    for line in text.splitlines():
        if not line.lstrip().startswith('{'):
            continue
        try:
            value = json.loads(line)
        except json.JSONDecodeError:
            continue
        if isinstance(value, dict):
            yield value
            for field in ('stdout', 'stderr', 'output'):
                if isinstance(value.get(field), str):
                    yield from json_objects(value[field])


def read_lane(path):
    text = Path(path).read_text(encoding='utf-8-sig', errors='replace')
    result = {'log': str(path), 'artifacts': [], 'artifact_messages': 0, 'tests': [], 'setup': [],
              'cpu': [], 'compiler_progress': [], 'telemetry': {}, 'headers': {}}
    context = {}
    for event in json_objects(text):
        if event.get('type') == 'cargo-invocation':
            context = event
        if event.get('reason') == 'compiler-artifact':
            result['artifact_messages'] += 1
            if not event['fresh']:
                result['artifacts'].append(artifact(event, context))
        if event.get('type') == 'test' and event.get('event') in ('ok', 'failed', 'ignored'):
            result['tests'].append(event)
        if event.get('type') == 'test-cost-setup':
            result['setup'].append(event)
        if event.get('type') == 'test-cost-cpu':
            result['cpu'].append(event)
    result['compiler_progress'] = re.findall(r'^\s*(?:Compiling|Checking) (\S+) v(\S+)', text, re.M)
    for field in ('Command', 'HEAD', 'Git status --short empty', 'Start', 'End', 'Exit code'):
        found = re.findall(rf'^{re.escape(field)}: (.+)$', text, re.M)
        result['headers'][field] = found[-1].strip() if found else None
    h = result['headers']
    if h['Start'] and h['End']:
        result['telemetry']['wall_seconds'] = (datetime.fromisoformat(h['End']) -
                                               datetime.fromisoformat(h['Start'])).total_seconds()
    durations = [e['exec_time'] for e in result['tests'] if 'exec_time' in e]
    result['telemetry']['test_work_seconds'] = sum(durations) if durations else None
    # A textual Compiling line has no features/profile/target or fresh bit.
    result['build_observation'] = 'json' if result['artifact_messages'] else 'unmeasured'
    result['duplicates'] = duplicates(result['artifacts'])
    return result


def artifact(event, context):
    files = event.get('filenames', [])
    filename = files[0].replace('\\', '/') if files else ''
    path = PurePosixPath(filename)
    target_dir = context.get('target_dir')
    if target_dir is None and 'deps' in path.parts:
        target_dir = str(PurePosixPath(*path.parts[:path.parts.index('deps') - 1]))
    return {'package': event['package_id'], 'target': event['target']['name'],
            'kind': sorted(event['target']['kind']), 'features': sorted(event['features']),
            'profile': event['profile'], 'target_dir': target_dir,
            'check_mode': context.get('mode', 'check' if any(f.endswith('.rmeta') for f in files)
                                      and not any(f.endswith('.rlib') for f in files) else 'build'),
            'filenames': files}


def duplicates(artifacts):
    packages = defaultdict(list)
    for value in artifacts:
        packages[value['package']].append(value)
    result = []
    for package, values in sorted(packages.items()):
        if len(values) < 2:
            continue
        causes = [key for key in ('features', 'profile', 'target_dir', 'check_mode', 'kind', 'target')
                  if len({json.dumps(v[key], sort_keys=True) for v in values}) > 1]
        result.append({'package': package, 'compiles': len(values),
                       'causes': causes or ['repeated identical graph'], 'artifacts': values})
    return result


def llvm_lines(text, helpers=()):
    """Read cargo llvm-lines' stable table, retaining totals and instantiations."""
    rows = []
    for line in text.splitlines():
        match = re.match(r'\s*(\d+)\s+(?:\([^)]*\)\s+)?(\d+)\s+(?:\([^)]*\)\s+)?(.+?)\s*$', line)
        if match:
            name = match[3]
            rows.append({'lines': int(match[1]), 'copies': int(match[2]), 'name': name,
                         'testkit_helpers': sorted(h for h in helpers if h in name)})
    if not rows or not any(r['name'] == 'TOTAL' for r in rows):
        raise ValueError('LLVM lines input has no TOTAL row')
    return sorted(rows, key=lambda r: (-r['lines'], r['name']))
