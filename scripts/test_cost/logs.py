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
    result = {'log': str(path), 'artifacts': [], 'artifact_messages': 0, 'tests': [], 'suites': [], 'setup': [],
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
        if event.get('type') == 'suite' and event.get('event') in ('ok', 'failed'):
            result['suites'].append(event)
        if event.get('type') == 'test-cost-setup':
            result['setup'].append(event)
        if event.get('type') == 'test-cost-cpu':
            result['cpu'].append(event)
        if event.get('type') == 'test-cost-phases':
            result['telemetry'].update(build_seconds=event['build_seconds'],
                                       run_seconds=event['wall_seconds'] - event['build_seconds'],
                                       wall_seconds=event['wall_seconds'])
    result['compiler_progress'] = re.findall(r'^\s*(?:Compiling|Checking) (\S+) v(\S+)', text, re.M)
    summaries = re.findall(r'^test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;', text, re.M)
    result['plain_test_counts'] = ({key: sum(int(row[index]) for row in summaries)
                                   for index, key in enumerate(('pass', 'fail', 'ignored'))}
                                  if summaries else None)
    build = re.findall(r'^\s*Finished .+ target\(s\) in (.+)$', text, re.M)
    run = re.findall(r'^\s*Summary \[\s*([^]]+)\]', text, re.M)
    if build:
        result['telemetry'].setdefault('build_seconds', sum(duration(value) for value in build))
    if run:
        result['telemetry'].setdefault('run_seconds', sum(duration(value) for value in run))
    for field in ('Command', 'HEAD', 'Git status --short empty', 'Start', 'End', 'Exit code'):
        found = re.findall(rf'^{re.escape(field)}: (.+)$', text, re.M)
        result['headers'][field] = found[-1].strip() if found else None
    h = result['headers']
    if h['Start'] and h['End']:
        elapsed = (datetime.fromisoformat(h['End']) - datetime.fromisoformat(h['Start'])).total_seconds()
        result['telemetry']['capture_wall_seconds'] = elapsed
        result['telemetry'].setdefault('wall_seconds', elapsed)
    durations = [e['exec_time'] for e in result['tests'] if 'exec_time' in e]
    result['telemetry']['test_work_seconds'] = sum(durations) if durations else None
    # A textual Compiling line has no features/profile/target or fresh bit.
    result['build_observation'] = 'json' if result['artifact_messages'] else 'unmeasured'
    result['duplicates'] = duplicates(result['artifacts'])
    return result


def duration(value):
    units = {'h': 3600, 'm': 60, 's': 1, 'ms': .001}
    matches = re.findall(r'(\d+(?:\.\d+)?)\s*(ms|h|m|s)', value)
    if not matches:
        raise ValueError(f'unrecognized Cargo/nextest duration: {value}')
    return sum(float(number) * units[unit] for number, unit in matches)


def artifact(event, context):
    files = event.get('filenames', [])
    filename = files[0].replace('\\', '/') if files else ''
    path = PurePosixPath(filename)
    target_dir = context.get('target_dir')
    if target_dir is None and 'deps' in path.parts:
        target_dir = str(PurePosixPath(*path.parts[:path.parts.index('deps') - 1]))
    if target_dir is None and filename:
        parent = path.parent
        if parent.name in ('examples', 'incremental'):
            parent = parent.parent
        if parent.name in ('debug', 'release', 'ci-test', 'diagnostic-test'):
            target_dir = str(parent.parent)
    linked = any(name.endswith(('.rlib', '.dll', '.so', '.dylib', '.a', '.lib', '.exe', '.wasm')) for name in files)
    check_mode = 'build' if linked else 'check' if any(name.endswith('.rmeta') for name in files) else context.get('mode', 'unmeasured')
    return {'package': event['package_id'], 'target': event['target']['name'],
            'kind': sorted(event['target']['kind']), 'features': sorted(event['features']),
            'profile': event['profile'], 'target_dir': target_dir,
            'cargo_profile': context.get('cargo_profile'),
            'check_mode': check_mode, 'invocation_mode': context.get('mode'),
            'filenames': files}


def duplicates(artifacts):
    packages = defaultdict(list)
    for value in artifacts:
        packages[value['package']].append(value)
    result = []
    for package, values in sorted(packages.items()):
        if len(values) < 2:
            continue
        causes = [key for key in ('features', 'profile', 'cargo_profile', 'target_dir', 'check_mode', 'kind', 'target')
                  if len({json.dumps(v[key], sort_keys=True) for v in values}) > 1]
        if not causes:
            outputs = {tuple(value.get('filenames', [])) for value in values}
            causes = ['unattributed compiler inputs' if len(outputs) > 1 else 'repeated observed compiler unit']
        result.append({'package': package, 'compiles': len(values),
                       'causes': causes, 'artifacts': values})
    return result


def llvm_lines(text, helpers=()):
    """Read cargo llvm-lines' stable table, retaining totals and instantiations."""
    rows = []
    for line in text.splitlines():
        match = re.match(r'\s*(\d+)\s+(?:\([^)]*\)\s+)?(\d+)\s+(?:\([^)]*\)\s+)?(.+?)\s*$', line)
        if match:
            name = match[3]
            inferred = re.findall(r'\b(worth_\w*(?:testkit\w*|test_support)::[^\s<>(),]+)', name)
            rows.append({'lines': int(match[1]), 'copies': int(match[2]), 'name': name,
                         'testkit_helpers': sorted(set(inferred) | {h for h in helpers if h in name})})
    if not rows or not any(r['name'] == 'TOTAL' for r in rows):
        raise ValueError('LLVM lines input has no TOTAL row')
    return sorted(rows, key=lambda r: (-r['lines'], r['name']))
