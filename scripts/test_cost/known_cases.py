"""Validate known cost discoveries against retained observations, never prose alone."""
import argparse
import json
import os
from pathlib import Path
import re
import tomllib

from logs import read_lane

ORACLES = ('branch_transitions_oracle', 'canonical_journal_oracle',
           'anchored_record_members::oracle', 'physical_checkpoint_oracle',
           'limit_sweep', 'process_kill', 'recovery_interruption')


def source_hits(documents, path_part, expression):
    result = []
    for path, text in sorted(documents.items()):
        if path_part not in path:
            continue
        for match in re.finditer(expression, text, re.M):
            start = text.rfind('\n', 0, match.start()) + 1
            end = text.find('\n', match.end())
            result.append({'path': path, 'line': text.count('\n', 0, match.start()) + 1,
                           'text': text[start:end if end >= 0 else len(text)][:600]})
    return result


def events(lanes, predicate):
    return [dict(event, log=lane['log']) for lane in lanes.values()
            for event in lane['tests'] if predicate(event)]


def check(name, evidence, detail=''):
    return {'name': name, 'status': 'found' if evidence else 'missing',
            'evidence': evidence, 'detail': detail}


def case(identity, checks):
    missing = [row['name'] for row in checks if row['status'] != 'found']
    return {'case': identity, 'status': 'found' if not missing else 'missing-evidence',
            'checks': checks, 'missing': missing}


def pool_match(expression, event):
    """Evaluate only the concrete selector grammar present in the retained pool rules.

    Unsupported expressions remain unknown; they never establish a reservation.
    """
    binary, separator, test = event['name'].partition('$')
    if not separator:
        return None
    binaries = re.findall(r'binary_id\(=([^()]*)\)', expression)
    tests = re.findall(r'test\((=?)([^()/]+)\)', expression)
    residue = re.sub(r'binary_id\(=[^()]*\)|test\(=?[^()/]+\)', '', expression)
    if re.sub(r'[\s+&()]', '', residue) or not tests:
        return None
    if binaries and binary not in binaries:
        return False
    return any(test == pattern if exact else pattern in test for exact, pattern in tests)


def diagnostic_hits(raw_logs, identity, expression):
    """Attribute cold-build output to the runner's named failure output block."""
    result = []
    for path, text in sorted(raw_logs.items()):
        headers = list(re.finditer(r'^\s*(?:TIMEOUT|FAIL)\s+[^\n]*' + re.escape(identity) + r'[^\n]*$', text, re.M))
        for header in headers:
            end = re.search(r'^\{"type":"test","event":"failed"', text[header.end():], re.M)
            limit = header.end() + end.start() if end else len(text)
            body = text[header.end():limit]
            for row in source_hits({path: body}, '', expression):
                row['line'] += text.count('\n', 0, header.end())
                row['attribution'] = header.group().strip()
                result.append(row)
    return result


def verified_environment(environment, documents):
    """Resolve the two named parameterized helper forms present in this corpus.

    A fixture inventory row supplies locations, never proof of a live caller.
    """
    location = environment['call_location']
    text = documents.get(location['path'], '')
    feature, profile = (environment[key] for key in ('feature_identity', 'profile_identity'))
    if any('"' + value + '"' not in text for value in (feature, profile)):
        return None
    constructor = 'run_cargo_ui_fixture_suite' in text or ('UiProofSuiteDeclaration::new' in text and 'UiProofEnvironment::cargo' in text)
    if not constructor:
        return None
    suite = environment['suite_identity']
    direct = '"' + suite + '"' in text
    origins = []
    for fixture in environment.get('fixtures', []):
        declaration = fixture.get('declaration_location')
        if not declaration:
            return None
        caller = documents.get(declaration['path'], '')
        if '"' + fixture['name'] + '"' not in caller:
            return None
        if not direct:
            forwarded = ('fn run_case(suite:' in text and re.search(r'\brun_case\s*\(\s*"' + re.escape(suite) + '"', caller))
            formatted = False
            if '"layout-indexes-{ui_dir}"' in text and suite.startswith('layout-indexes-'):
                suffix = suite[len('layout-indexes-'):]
                formatted = re.search(r'\bassert_compile_fails_in_ui_dir\s*\(\s*"' + re.escape(suffix) + '"', caller)
                if suffix == 'foundations' and 'assert_compile_fails_in_ui_dir("foundations"' in text:
                    formatted = formatted or re.search(r'\bassert_compile_fails\s*\(', caller)
            if not forwarded and not formatted:
                return None
        origins.append(declaration)
    # Direct synthetic probes may have no fixtures; real inventories validate
    # every retained fixture origin before counting their environments.
    dependency_key = json.dumps(environment.get('dependencies', []), sort_keys=True)
    return {'location': location, 'identity': [suite, feature, profile, dependency_key],
            'fixtures': len(environment.get('fixtures', [])), 'fixture_origins': origins}


def serial_executions(text):
    """Retain Cargo's actual executable target as well as the full libtest name."""
    binary = None
    result = set()
    for line in text.splitlines():
        running = re.match(r'\s*Running .+ \(([^)]+)\)', line)
        if running:
            filename = running[1].replace('\\', '/').rsplit('/', 1)[-1]
            target = re.match(r'(.+)-[0-9a-f]{16}(?:\.exe)?$', filename)
            binary = target[1] if target else None
        observed = re.match(r'^test (\S+) \.\.\. (?:ok|FAILED|ignored)', line)
        if observed and binary:
            result.add((binary, observed[1]))
    return result


def validate(before, documents, lanes, raw_logs, environments=None, configuration=None):
    """Pure validation API: all evidence inputs are already observed documents."""
    rows = []
    sites = before.get('source_sites', [])
    cargo = [row for row in sites if row['kind'] == 'cargo-command']
    helper = [row for row in cargo if 'independent_executable' in row['path']]
    cluster = events(lanes, lambda e: 'physical_record_journeys$' in e['name']
                     and 280 <= e.get('exec_time', -1) <= 295)
    timeout = events(lanes, lambda e: 'record_world_survives_fresh_processes' in e['name']
                     and e['event'] == 'failed' and e.get('reason') == 'time limit exceeded')
    rows.append(case('helper-executable-builds', [
        check('census Cargo site', helper),
        check('source compiler launch', source_hits(documents, 'independent_executable', r'Command::new\(')),
        check('record journey executable caller', source_hits(documents, 'physical_record_journeys', r'OFFLINE_ROOT_OBSERVER\.path\(')),
        check('approximately 287 second cluster', cluster, 'Historical timing telemetry; no timing gates.'),
        check('600 second timeout', [e for e in timeout if 599 <= e.get('exec_time', -1) < 602]),
    ]))
    authority = events(lanes, lambda e: 'authority_compile$' in e['name']
                       and e['event'] == 'failed' and e.get('reason') == 'time limit exceeded')
    locks = diagnostic_hits(raw_logs, '::authority_compile ', r'Blocking waiting for file lock on package cache')
    cold = diagnostic_hits(raw_logs, '::authority_compile ', r'\bChecking worth-store(?: |-)')
    rows.append(case('authority-cold-compiler-lock', [
        check('census compiler session', [r for r in sites if r['kind'] == 'trybuild' and 'authority_compile' in r['path']]),
        check('source compiler session', source_hits(documents, 'authority_compile', r'trybuild::TestCases')),
        check('900 second timeout', [e for e in authority if 899 <= e.get('exec_time', -1) < 902]),
        check('package cache lock', locks), check('cold dependency check', cold),
    ]))
    verified = []
    for environment in (environments or {}).get('environments', []):
        observed = verified_environment(environment, documents)
        if observed is not None:
            verified.append(observed)
    identities = {tuple(row['identity']) for row in verified}
    rows.append(case('compiler-boundary-per-fixture-environments', [
        check('census Cargo site', [r for r in cargo if 'compiler_boundary/cargo_environment' in r['path']]),
        check('fixture loop', source_hits(documents, 'compiler_boundary/cargo_environment', r'for fixture in declaration\.fixtures\(\)')),
        check('single-bin check', source_hits(documents, 'compiler_boundary/cargo_environment', r'"--bin"')),
        check('18 source-verified environments', verified if len(identities) == 18 else [],
              f'{len(identities)} distinct source-verified environments; declared counts are not evidence.'),
    ]))
    passes = [r for r in sites if r['kind'] == 'trybuild-pass']
    pass_sources = []
    for row in passes:
        pass_sources.extend(source_hits({row['path']: documents.get(row['path'], '')}, '', r'\b\w+\.pass\('))
    pass_sources = list({(r['path'], r['line']): r for r in pass_sources}.values())
    rows.append(case('trybuild-positive-programs', [check('census pass sites', passes),
                                                  check('source pass calls', pass_sources)]))
    formal = [(name, lane) for name, lane in lanes.items() if 'formal' in name]
    builds = [{'log': lane['log'], 'build_seconds': lane['telemetry'].get('build_seconds')}
              for _, lane in formal if lane['telemetry'].get('build_seconds', 0) >= 1500]
    runs = [{'log': lane['log'], 'run_seconds': lane['telemetry'].get('run_seconds'),
             'test_events': lane['tests']} for _, lane in formal
            if 0 < lane['telemetry'].get('run_seconds', 999) < 1 and lane['tests']]
    rows.append(case('formal-build-dominates-run', [check('26 minute build', builds),
                                                   check('subsecond test execution', runs)]))
    config = configuration or {}
    overrides = [(profile, rule) for profile, value in config.get('profile', {}).items()
                 for rule in value.get('overrides', []) if rule.get('threads-required') == 'num-test-threads']
    for name in ORACLES:
        observed = events(lanes, lambda e: name in e['name'])
        loops = source_hits(documents, name.replace('::', '/'), r'\bfor\s+[^\n]+\s+in\s+[^\n]+')
        reservations = [{'profile': profile, 'filter': rule['filter'],
                         'tests': sorted({event['name'] for event in observed if pool_match(rule['filter'], event) is True})}
                        for profile, rule in overrides]
        reservations = [row for row in reservations if row['tests']]
        rows.append(case('sequential-pool:' + name, [check('named execution', observed),
            check('source serial loop', loops, 'Lexical serial enumeration; CPU parallelism is unmeasured.'),
            check('matching whole-pool reservation', reservations,
                  'No reservation is inferred from duration or from the task description.')]))
    workspace = [(name, lane) for name, lane in lanes.items() if 'workspace' in name]
    serial = [{'log': lane['log'], 'command': text.splitlines()[0] if text else ''}
              for name, lane in workspace for text in [raw_logs.get(lane['log'], '')]
              if re.search(r'cargo test --workspace', text[:4000])]
    repeats = []
    binary_owners = {}
    for lane in lanes.values():
        for event in lane['tests']:
            binary, separator, _ = event['name'].partition('$')
            if separator:
                binary_owners.setdefault(binary.rsplit('::', 1)[-1], set()).add(binary)
    for name, lane in workspace:
        observed = serial_executions(raw_logs.get(lane['log'], ''))
        for other_name, other in lanes.items():
            if other_name == name:
                continue
            matches = set()
            for event in other['tests']:
                binary, separator, test = event['name'].partition('$')
                target = binary.rsplit('::', 1)[-1]
                if separator and len(binary_owners.get(target, [])) == 1 and (target, test) in observed:
                    matches.add(event['name'])
            if matches:
                repeats.append({'workspace_log': lane['log'], 'partition_log': other['log'],
                                'matched_binary_and_full_names': sorted(matches)})
    rows.append(case('serial-workspace-repeats-partitions', [check('workspace command', serial),
                                                           check('same binary and full test names repeated', repeats)]))
    missing = [row['case'] for row in rows if row['status'] != 'found']
    return {'schema': 1, 'source_revision': before.get('source_revision'),
            'timing_relationship': before.get('source_timing_relationship'),
            'status': 'pass' if not missing else 'missing-evidence', 'missing': missing, 'cases': rows,
            'limitations': ['Historical source/timing association does not measure causal build time.',
                            'Landing logs contain no CPU/slot or per-builder fingerprint telemetry.',
                            'Unknown reservation expressions cannot establish whole-pool ownership.']}


def markdown(result):
    lines = ['# Known test-cost discovery validation', '', f'Status: **{result["status"]}**.', '',
             'Timings are historical telemetry, never gates. Source and timings may precede different court revisions.', '',
             '| Known case | Discovery | Missing evidence |', '|---|---|---|']
    lines += [f'| {row["case"]} | {row["status"]} | {", ".join(row["missing"]) or "none"} |' for row in result['cases']]
    for row in result['cases']:
        lines += ['', '## ' + row['case'], '']
        for check_row in row['checks']:
            lines += [f'- {check_row["name"]}: **{check_row["status"]}**. {check_row["detail"]}']
            for evidence in check_row['evidence'][:12]:
                lines.append('  - `' + json.dumps(evidence, ensure_ascii=True) + '`')
            if len(check_row['evidence']) > 12:
                lines.append(f'  - {len(check_row["evidence"])} observations retained in JSON.')
    lines += ['', '## Limits', '', *('- ' + text for text in result['limitations'])]
    return '\n'.join(lines) + '\n'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--before', type=Path, required=True)
    parser.add_argument('--source-root', type=Path, required=True)
    parser.add_argument('--landing', type=Path, required=True)
    parser.add_argument('--environments', type=Path)
    parser.add_argument('--config', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    documents = {}
    root = args.source_root.resolve()
    if os.name == 'nt':
        root = Path('\\\\?\\' + str(root))
    for base, dirs, files in os.walk(root):
        dirs[:] = [name for name in dirs if name not in ('target', '.git')]
        for name in files:
            if name.endswith('.rs'):
                path = Path(base) / name
                documents[path.relative_to(root).as_posix()] = path.read_text(encoding='utf-8')
    lanes, raw = {}, {}
    for name in ('scenario', 'ui', 'owner-unit', 'formal', 'workspace-tests', 'phase34'):
        path = args.landing / f'landing-store-{name}.log'
        if path.exists():
            lanes[name] = read_lane(path)
            raw[str(path)] = path.read_text(encoding='utf-8-sig', errors='replace')
    result = validate(json.loads(args.before.read_text()), documents, lanes, raw,
                      json.loads(args.environments.read_text()) if args.environments else None,
                      tomllib.loads(args.config.read_text(encoding='utf-8-sig')))
    result['inputs'] = {key: str(value) for key, value in vars(args).items() if value is not None}
    args.output.mkdir(parents=True, exist_ok=True)
    (args.output / 'known-cases.json').write_text(json.dumps(result, indent=2) + '\n', encoding='utf-8')
    (args.output / 'known-cases.md').write_text(markdown(result), encoding='utf-8')
    print(json.dumps({'status': result['status'], 'missing': result['missing']}))
    return 0 if result['status'] == 'pass' else 1


if __name__ == '__main__':
    raise SystemExit(main())
