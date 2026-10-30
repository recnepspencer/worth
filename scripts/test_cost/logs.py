"""Cargo artifacts and libtest/nextest observations from retained lane logs."""
from collections import Counter, defaultdict
from datetime import datetime
import json
from pathlib import Path, PurePosixPath
import re

from compiler_units import CompilerUnitRefusal, attribute as attribute_unit, failed_unit


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
              'cpu': [], 'compiler_progress': [], 'telemetry': {}, 'headers': {},
              'failed_compiler_attempts': [], 'compiler_attempt_refusals': [], 'compiler_error_messages': 0}
    context, failed, refused, invocation, states, state = {}, {}, set(), -1, [], None
    for event in json_objects(text):
        if event.get('type') == 'cargo-invocation':
            invocation += 1
            context = dict(event, observation_invocation=f'{Path(path).resolve()}#{invocation}')
            state = {'context': context, 'build_success': None, 'exit_code': None,
                     'failed_units': set(), 'refused': False}
            states.append(state)
        if (event.get('reason') == 'build-finished' and compiler_context(context)
                and state is not None and state['build_success'] is None):
            state['build_success'] = event.get('success')
        if event.get('type') == 'cargo-result' and state is not None:
            state['exit_code'] = event['exit_code']
        active = state is None or state['build_success'] is None
        if event.get('reason') == 'compiler-artifact' and active and (not context.get('command') or compiler_context(context)):
            result['artifact_messages'] += 1
            if not event['fresh']:
                result['artifacts'].append(artifact(event, context))
        if (event.get('reason') == 'compiler-message' and event.get('message', {}).get('level') == 'error'
                and active and compiler_context(context)):
            result['compiler_error_messages'] += 1
            try:
                attempted = denied_attempt(event, context)
                failed.setdefault((attempted['invocation'], attempted['unit']['identity']), attempted)
                state['failed_units'].add(attempted['unit']['identity'])
            except (CompilerUnitRefusal, KeyError, TypeError) as error:
                key = (invocation, event.get('package_id'), json.dumps(event.get('target'), sort_keys=True))
                if key not in refused:
                    refused.add(key)
                    result['compiler_attempt_refusals'].append({'kind': 'unattributed-failed-compiler-unit',
                        'package': event.get('package_id'), 'target': event.get('target'), 'reason': str(error)})
                state['refused'] = True
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
    if states and states[-1]['exit_code'] is None and h['Exit code'] is not None:
        states[-1]['exit_code'] = int(h['Exit code'])
    for state in states:
        unsuccessful = (state['build_success'] is False or
                        state['exit_code'] not in (None, 0) and state['build_success'] is not True)
        if (compiler_context(state['context']) and unsuccessful
                and not state['failed_units'] and not state['refused']):
            result['compiler_attempt_refusals'].append({'kind': 'unattributed-compiler-outcome',
                'invocation': state['context']['observation_invocation'],
                'build_success': state['build_success'], 'exit_code': state['exit_code'],
                'reason': 'unsuccessful compiler invocation has no attributable failed unit'})
    if h['Start'] and h['End']:
        elapsed = (datetime.fromisoformat(h['End']) - datetime.fromisoformat(h['Start'])).total_seconds()
        result['telemetry']['capture_wall_seconds'] = elapsed
        result['telemetry'].setdefault('wall_seconds', elapsed)
    durations = [e['exec_time'] for e in result['tests'] if 'exec_time' in e]
    result['telemetry']['test_work_seconds'] = sum(durations) if durations else None
    # A textual Compiling line has no features/profile/target or fresh bit.
    structured = result['artifact_messages'] or result['compiler_error_messages'] or any(
        state['build_success'] is not None for state in states if compiler_context(state['context']))
    result['build_observation'] = 'json' if structured else 'unmeasured'
    emitted = {(row['invocation'], row['unit']['identity']) for row in result['artifacts'] if row['unit']}
    result['failed_compiler_attempts'] = [row for key, row in failed.items() if key not in emitted]
    result['duplicates'] = duplicates(result['artifacts'])
    result['compiler_attempt_groups'] = duplicates(result['artifacts'] + result['failed_compiler_attempts'])
    return result


def compiler_context(context):
    command = context.get('command', [])
    if '--unit-graph' in command:
        return False
    if context.get('unit_plan') is not None:
        return True
    if command[:2] == ['cargo', 'nextest']:
        return not any(argument == '--binaries-metadata' or argument.startswith('--binaries-metadata=')
                       for argument in command)
    return command[:1] == ['cargo'] and command[1:2] in (
        ['build'], ['check'], ['test'], ['rustc'], ['clippy'], ['run'])


def denied_attempt(event, context):
    if context.get('unit_plan') is None:
        raise CompilerUnitRefusal('failed compilation has no preexecution Cargo unit declaration')
    unit = failed_unit(event, context['unit_plan'])
    profile = {field: unit['declared_profile'][field] for field in
               ('opt_level', 'debuginfo', 'debug_assertions', 'overflow_checks')}
    profile['test'] = unit['mode'] == 'test'
    return {'package': event['package_id'], 'target': event['target']['name'],
            'kind': sorted(event['target']['kind']), 'features': unit['features'], 'profile': profile,
            'target_dir': context.get('target_dir'), 'cargo_profile': context.get('cargo_profile'),
            'check_mode': 'check' if unit['output'] == 'check' else 'build',
            'invocation_mode': context.get('mode'), 'invocation': context.get('observation_invocation'),
            'filenames': [], 'unit': unit, 'unit_refusal': None, 'observation': 'compiler-error'}


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
    unit, refusal = None, None
    if context.get('unit_plan') is not None:
        try:
            unit = attribute_unit(event, context['unit_plan'])
        except (CompilerUnitRefusal, KeyError, TypeError) as error:
            refusal = str(error)
    else:
        refusal = 'no preexecution Cargo unit declaration in this capture'
    return {'package': event['package_id'], 'target': event['target']['name'],
            'kind': sorted(event['target']['kind']), 'features': sorted(event['features']),
            'profile': event['profile'], 'target_dir': target_dir,
            'cargo_profile': context.get('cargo_profile'),
            'check_mode': check_mode, 'invocation_mode': context.get('mode'),
            'invocation': context.get('observation_invocation'),
            'filenames': files, 'unit': unit, 'unit_refusal': refusal}


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
        units = [value['unit'] for value in values if value.get('unit') is not None
                 and not value.get('unit_refusal')]
        fields = {'required_dependency_graph': 'dependency_units', 'physical_role': 'roles',
                  'target_platform': 'platform', 'required_output': 'output',
                  'declared_profile': 'declared_profile', 'declared_target': 'declared_target',
                  'compiler_inputs': 'compiler_inputs'}
        causes += [cause for cause, field in fields.items()
                   if len({json.dumps(unit[field], sort_keys=True) for unit in units}) > 1]
        counts = Counter(unit['identity'] for unit in units)
        repeated = [{'identity': identity, 'compiles': count}
                    for identity, count in sorted(counts.items()) if count > 1]
        if repeated:
            causes.append('repeated identical required unit')
        if not causes and len(units) == len(values):
            causes = ['distinct declared compiler units']
        elif not causes:
            outputs = {tuple(value.get('filenames', [])) for value in values}
            causes = ['unattributed compiler inputs' if len(outputs) > 1 else 'repeated observed compiler unit']
        result.append({'package': package, 'compiles': len(values),
                       'causes': causes, 'artifacts': values,
                       'unit_observation': 'declared' if len(units) == len(values) else 'unmeasured',
                       'distinct_required_units': len(counts) if len(units) == len(values) else None,
                       'repeated_units': repeated, 'unattributed_compiles': len(values) - len(units)})
    return result


def llvm_lines(text, helpers=()):
    """Read cargo llvm-lines' stable table, retaining totals and instantiations."""
    rows = []
    for line in text.splitlines():
        match = re.match(r'\s*(\d+)\s+(?:\([^)]*\)\s+)?(\d+)\s+(?:\([^)]*\)\s+)?(.+?)\s*$', line)
        if match:
            name = match[3]
            if name == '(TOTAL)':
                name = 'TOTAL'
            inferred = re.findall(r'\b(worth_\w*(?:testkit\w*|test_support)::[^\s<>(),]+)', name)
            rows.append({'lines': int(match[1]), 'copies': int(match[2]), 'name': name,
                         'testkit_helpers': sorted(set(inferred) | {h for h in helpers if h in name})})
    if not rows or not any(r['name'] == 'TOTAL' for r in rows):
        raise ValueError('LLVM lines input has no TOTAL row')
    return sorted(rows, key=lambda r: (-r['lines'], r['name']))
