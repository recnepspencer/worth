"""Preserve full attempts and link repair evidence to exact binary/test claims."""
import argparse
from collections import Counter, defaultdict
import hashlib
import json
from pathlib import Path

from coverage import inventory
from executed_coverage import observations, prove
from logs import read_lane

HEADERS = ('Command', 'HEAD', 'Git status --short empty', 'Start', 'End', 'Exit code')


def key(row):
    return row['binary'], row['test']


def provenance(path, command=False):
    path = Path(path).resolve()
    raw = path.read_bytes()
    result = {'path': str(path), 'sha256': hashlib.sha256(raw).hexdigest(), 'bytes': len(raw)}
    if command:
        headers = read_lane(path)['headers']
        if any(headers.get(field) is None for field in HEADERS):
            raise ValueError('incomplete execution provenance: ' + str(path))
        result['headers'] = headers
    return result


def load_attempt(specification):
    report_path = Path(specification['report'])
    report = json.loads(report_path.read_text(encoding='utf-8'))
    recorded = report['executed_coverage']
    native_path = Path(specification['native_inventory'])
    native = json.loads(native_path.read_text(encoding='utf-8'))
    logs = list(recorded['logs'])
    events, walls = observations(logs, native)
    rows = recorded['tests']
    if any(row['selections'] not in (0, 1) for row in rows):
        raise ValueError('attempt selects a claim more than once: ' + specification['name'])
    selected = [row for row in rows if row['selections'] == 1]
    checked = prove(rows, {'selection': selected}, events,
                    require_full_workspace=recorded['require_full_workspace'])
    for field in ('selected', 'executed', 'passed', 'failed', 'missing', 'unexpected', 'multiple',
                  'unknown', 'ignored_executed', 'ignored_selected', 'selection_multiple', 'scope'):
        if json.dumps(checked[field], sort_keys=True) != json.dumps(recorded[field], sort_keys=True):
            raise ValueError(f'attempt summary contradicts raw observations: {specification["name"]}: {field}')
    captures = [provenance(path, command=True) for path in logs]
    # Initial capture recovery already binds every raw capture by content hash.
    originals = {str(Path(row['retained_log']).resolve()): row for row in
                 report.get('capture_recovery', {}).get('captures', [])}
    for capture in captures:
        original = originals.get(capture['path'])
        if original and original['sha256'] != capture['sha256']:
            raise ValueError('initial capture no longer matches its recorded hash: ' + capture['path'])
    for index, event in enumerate(events):
        event['observation'] = index
    budget_path = report_path.with_name('build-ratchet.json')
    budget = json.loads(budget_path.read_text()) if budget_path.exists() else None
    return {'reported_status': report.get('status'), 'build_ratchet': budget,
            'name': specification['name'], 'known': inventory(rows), 'selected': selected,
            'events': events, 'check': checked, 'captures': captures, 'walls': walls,
            'inputs': [provenance(report_path), provenance(native_path)] +
                      ([provenance(budget_path)] if budget_path.exists() else []),
            'rationale': specification.get('rationale', 'Scope is the recorded selected subset; no source-wide inference.')}


def compose(final_rows, attempts, affected=()):
    """Compose claim evidence without converting multiple attempts into one run.

    Each selected repair scope is affected. Explicit affected claims can require
    a rerun even when their original execution passed. Caller-provided affected
    scope is recorded, not asserted to be a complete audit of every source edit.
    """
    final = {key(row): row for row in inventory(final_rows)}
    if not final:
        raise ValueError('final inventory must contain observed workspace claims')
    if not attempts or len({attempt['name'] for attempt in attempts}) != len(attempts):
        raise ValueError('lineage requires distinct, ordered attempt names')
    initial = {key(row): row for row in inventory(attempts[0]['known'])}
    required = {identity: 0 for identity, row in final.items() if not row['ignored']}
    histories = defaultdict(list)
    summaries, anomalies = [], []
    seen_logs = set()
    positions = {attempt['name']: index for index, attempt in enumerate(attempts)}
    for obligation in affected:
        identity = key(obligation)
        if identity not in final or obligation['since'] not in positions:
            raise ValueError('affected obligation has no final claim or attempt: ' + repr(obligation))
        required[identity] = max(required.get(identity, 0), positions[obligation['since']])
    for index, attempt in enumerate(attempts):
        selected = {key(row) for row in attempt['selected'] if not row['ignored']}
        counts = Counter(key(event) for event in attempt['events'])
        prior_failed = {identity for identity, events in histories.items() if events[-1]['event'] == 'failed'}
        prior_passed = {identity for identity, events in histories.items() if events[-1]['event'] == 'ok'}
        for identity in selected:
            if identity in required:
                required[identity] = max(required[identity], index)
        for capture in attempt.get('captures', []):
            if capture['path'] in seen_logs:
                raise ValueError('same execution log supplied in different attempts: ' + capture['path'])
            seen_logs.add(capture['path'])
        for ordinal, event in enumerate(attempt['events']):
            identity = key(event)
            histories[identity].append(dict(event, attempt=attempt['name'], attempt_index=index,
                                            observation=ordinal))
            if identity not in selected or identity not in final or final[identity]['ignored']:
                anomalies.append({'attempt': attempt['name'], 'claim': identity, 'kind': 'unexpected-or-ignored-execution'})
        repeated = sorted(identity for identity, count in counts.items() if count > 1)
        if repeated:
            anomalies.append({'attempt': attempt['name'], 'kind': 'repeated-within-attempt', 'claims': repeated})
        command_failures = [capture for capture in attempt.get('captures', [])
                            if int(capture['headers']['Exit code']) != 0]
        for capture in command_failures:
            if not any(event['event'] == 'failed' and str(Path(event['log']).resolve()) == capture['path']
                       for event in attempt['events']):
                anomalies.append({'attempt': attempt['name'], 'kind': 'unexplained-nonzero-command',
                                  'capture': capture})
        summary = {'name': attempt['name'], 'selected': len(selected), 'executed': sum(counts.values()),
                   'passed': sum(event['event'] == 'ok' for event in attempt['events']),
                   'failed': [key(event) for event in attempt['events'] if event['event'] == 'failed'],
                   'prior_failed_claims_rerun': sorted(prior_failed & counts.keys()),
                   'prior_failed_claims_not_rerun': sorted(prior_failed - counts.keys()),
                   'previously_passing_affected_claims_rerun': sorted(prior_passed & counts.keys()),
                   'selected_but_missing': sorted(selected - counts.keys()),
                   'command_failures': command_failures, 'rationale': attempt.get('rationale'),
                   'captures': attempt.get('captures', []), 'inputs': attempt.get('inputs', []),
                   'raw_events': attempt['events'], 'reported_status': attempt.get('reported_status'),
                   'build_ratchet': attempt.get('build_ratchet')}
        summaries.append(summary)
    claims, unresolved = [], []
    for identity, row in sorted(final.items()):
        history = histories[identity]
        reason = None
        if row['ignored']:
            disposition = 'scheduled-only'
        elif not history:
            disposition, reason = 'unresolved', 'no recorded execution'
        elif history[-1]['attempt_index'] < required[identity]:
            disposition, reason = 'unresolved', 'affected claim not rerun after its required attempt'
        elif history[-1]['event'] != 'ok':
            disposition, reason = 'unresolved', 'latest recorded execution failed'
        else:
            disposition = 'composed-passing-evidence'
        value = dict(binary=identity[0], test=identity[1], disposition=disposition,
                     required_since=attempts[required.get(identity, 0)]['name'], reason=reason,
                     added_after_initial=identity not in initial, observations=history)
        claims.append(value)
        if disposition == 'unresolved':
            unresolved.append(value)
    removed = sorted(set(initial) - set(final))
    changed = sorted(identity for identity in set(initial) & set(final)
                     if initial[identity]['ignored'] != final[identity]['ignored'])
    if removed or changed:
        anomalies.append({'kind': 'unexplained-inventory-change', 'removed': removed, 'changed_disposition': changed})
    full = {identity for identity, row in final.items() if not row['ignored']}
    green_attempts = [attempt['name'] for attempt, summary in zip(attempts, summaries)
                      if {key(row) for row in attempt['selected'] if not row['ignored']} == full
                      and summary['executed'] == len(full) and not summary['failed']
                      and not summary['selected_but_missing'] and not summary['command_failures']]
    return {'schema': 1, 'status': 'composed-evidence-incomplete' if unresolved or anomalies else 'composed-evidence-complete',
            'certified': False, 'full_workspace_green_attempts': green_attempts,
            'inventory': len(final), 'landing': len(full), 'scheduled_only': len(final) - len(full),
            'total_recorded_executions': sum(row['executed'] for row in summaries),
            'added_claims': sorted(set(final) - set(initial)), 'removed_claims': removed,
            'unresolved': unresolved, 'anomalies': anomalies, 'attempts': summaries, 'claims': claims,
            'explicit_affected_obligations': list(affected),
            'limits': ['Attempts retain their own revisions, failures and every execution occurrence.',
                       'Composed passing claims are not one full green run or final certification.',
                       'Completeness of the causal affected-source scope still requires independent review.',
                       'Ignored inventory entries are scheduled-only and may not execute.',
                       'Claim lineage does not waive build-ratchet failures or certify source ancestry.']}


def markdown(result):
    lines = ['# Store verification lineage', '', f'Status: **{result["status"]}**; certified: **false**.', '',
             f'Final inventory {result["inventory"]}; landing {result["landing"]}; scheduled-only {result["scheduled_only"]}. '
             f'Total actual executions across attempts: {result["total_recorded_executions"]}. '
             f'Full-workspace green attempts: {result["full_workspace_green_attempts"]}.', '',
             '| Attempt | Selected | Executed | Passed | Failed | Earlier failed claims rerun | Earlier passing claims rerun |',
             '|---|---:|---:|---:|---:|---:|---:|']
    for row in result['attempts']:
        lines.append(f'| {row["name"]} | {row["selected"]} | {row["executed"]} | {row["passed"]} | '
                     f'{len(row["failed"])} | {len(row["prior_failed_claims_rerun"])} | '
                     f'{len(row["previously_passing_affected_claims_rerun"])} |')
    lines += ['', '## Added claims', '']
    lines += ['- `' + binary + '$' + test + '`' for binary, test in result['added_claims']]
    lines += ['', '## Unresolved claims', '']
    lines += [f'- `{row["binary"]}${row["test"]}`: {row["reason"]}.' for row in result['unresolved']]
    if not result['unresolved']:
        lines.append('None in the supplied final inventory and affected scopes; final certification is still separate.')
    lines += ['', '## Attempt provenance and repairs', '']
    for attempt in result['attempts']:
        lines += ['', '### ' + attempt['name'], '', str(attempt['rationale']), '']
        for capture in attempt['captures']:
            header = capture['headers']
            lines.append(f'- `{capture["path"]}`: HEAD `{header["HEAD"]}`, status-clean `{header["Git status --short empty"]}`, '
                         f'exit {header["Exit code"]}; SHA-256 `{capture["sha256"]}`.')
        lines += ['- Failed in this attempt: `' + json.dumps(attempt['failed']) + '`.',
                  '- Earlier failed claims rerun: `' + json.dumps(attempt['prior_failed_claims_rerun']) + '`.']
    lines += ['', '## Limits', '', *('- ' + value for value in result['limits']), '',
              'All occurrence-level histories, captured commands/times, content hashes, input provenance and anomalies are retained in JSON.']
    return '\n'.join(lines) + '\n'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--input', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    specification = json.loads(args.input.read_text(encoding='utf-8'))
    if specification.get('schema') != 1:
        raise ValueError('unsupported lineage input schema')
    native_path = Path(specification['final_native_inventory'])
    final_rows = inventory(json.loads(native_path.read_text(encoding='utf-8')))
    final_inputs = [provenance(native_path)]
    if specification.get('final_documentation'):
        path = Path(specification['final_documentation'])
        final_rows += inventory(json.loads(path.read_text(encoding='utf-8'))['inventory'])
        final_inputs.append(provenance(path))
    result = compose(final_rows, [load_attempt(row) for row in specification['attempts']], specification.get('affected', []))
    result['inputs'] = [provenance(args.input), *final_inputs]
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.with_suffix('.json').write_text(json.dumps(result, indent=2) + '\n', encoding='utf-8')
    args.output.with_suffix('.md').write_text(markdown(result), encoding='utf-8')
    print(json.dumps({field: result[field] for field in ('status', 'certified', 'inventory', 'landing',
                     'total_recorded_executions', 'full_workspace_green_attempts')}))
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
