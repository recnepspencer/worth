"""Rank judgment residue using measured test work; unknown costs stay unranked."""
import argparse
from collections import Counter
import json
import math
from pathlib import Path

from logs import llvm_lines, read_lane

# These classify judgment responsibilities, not costs or execution inventories.
# All numeric ranking inputs must come from actual retained test events.
ITEMS = {
    'store': [
        ('home-media', 'Relational home media sweeps', ('media_sweep::',), 'remaining',
         'Profile per-worker prefix restoration and filesystem I/O; retain every seed, cut and continuation.'),
        ('home-process', 'Relational home process kills', ('relational_home$process_kill::', 'relational_home::process_kill::'), 'pool implemented; runtime pending',
         'Independent stops may run concurrently; preserve actual process kills, both checkpoint cases and the late-arrival race.'),
        ('qualification', 'Backend qualification setup', ('backend_qualification',), 'remaining',
         'Share only evidence with equal backend, scenario, simulation and executed-evidence identity; preserve negative twins.'),
        ('checkpoint', 'Physical checkpoint oracle', ('physical_checkpoint_oracle::',), 'pool implemented; runtime pending',
         'Preserve every role/ordinal/directive, killed-recovery witness, suffix and second recovery; review prefix setup next.'),
        ('anchored', 'Member-carrying anchored replacement', ('anchored_record_members::oracle::',), 'pool implemented; runtime pending',
         'Keep per-case answers and both final witnesses; preserve whole old/new values and all fault tuples.'),
        ('limits', 'Published-object adaptive limit sweeps', ('limit_sweep::',), 'remaining',
         'Each adaptive search depends on earlier denials; preserve exact visited evidence and ordered recovery. Review dimensions separately.'),
        ('branch', 'Branch-transition oracle', ('branch_transitions_oracle::',), 'pool implemented; runtime pending',
         'Retain unresolved-refusal and retry witnesses, every admitted schedule and complete suffix continuation.'),
        ('journal', 'Canonical-journal oracle', ('canonical_journal_oracle::',), 'pool implemented; runtime pending',
         'Keep reference runtime ownership and worker-local admission; prefix reuse must preserve filesystem root binding.'),
        ('recovery', 'Recovery interruption families', ('recovery_interruption::',), 'pool implemented; runtime pending',
         'Pool independent stages only; each stage retains actual killed recovery followed by its ordered second/third recoveries.'),
        ('pressure', 'Pressure harness setup', ('pressure_harness',), 'remaining',
         'Retain execution provenance and mismatch/copy negative cases before sharing replay construction.'),
    ],
    'root': [
        ('proof', 'Proof prerequisite graphs', ('band_guard_cargo_contract::',), 'prebuild implemented; runtime pending',
         'Real package checks, dependency tree and independently compiled guarded/unguarded release IR remain required.'),
        ('setup', 'Repeated world construction and polling', (), 'remaining',
         'Measure fingerprints first; share immutable construction only, preserving each mutable fate and authority owner.'),
    ],
    'query': [
        ('expiry', 'Signed-cutoff and authorization expiry waits', ('inbound_admission::', 'authorization::'), 'remaining',
         'Cross the actual owning clock boundary; signed correlation and accepted-slot reclamation cannot be replaced by a positive-event wait.'),
        ('application', 'Installed-operation/application graph construction', ('application_graph::', 'installed_operation::'), 'remaining',
         'Keep independent issuers, branch state, custody, mutation fate and authentic authority; measure equal inputs before sharing.'),
        ('process', 'Host process deadlines', ('process_deadline::',), 'remaining',
         'Child termination and output bounds remain real and diagnosable; similar representation does not prove shared lifecycle.'),
    ],
    'bank': [
        ('metadata', 'Estate production-dependency metadata', ('production_bank_packages_use_only_query_audience_crates',), 'prebuild implemented; runtime pending',
         'Read raw current-workspace Cargo metadata and retain every audience predicate; reject missing, invalid or foreign observations.'),
        ('rail', 'Fresh-process/external-rail startup', ('external_effect_dispatch::', 'inbound_completion::'), 'remaining',
         'Keep independently built executables, real completion, no redispatch and correct indeterminate outcomes.'),
        ('maintenance', 'Async maintenance waits', ('maintenance::',), 'remaining',
         'Keep absence of extra calls after settlement/stop, bounded automatic sweeps and overlap sensitivity.'),
    ],
}


def measured(events):
    selected = [event for event in events if event.get('event') in ('ok', 'failed')
                and isinstance(event.get('exec_time'), (int, float))
                and math.isfinite(event['exec_time']) and event['exec_time'] >= 0]
    if not selected:
        return {'frequency': None, 'work_seconds': None, 'failed': None, 'tests': []}
    return {'frequency': len(selected), 'work_seconds': sum(event['exec_time'] for event in selected),
            'failed': sum(event['event'] == 'failed' for event in selected),
            'tests': [{'name': event['name'], 'event': event['event'], 'seconds': event['exec_time']}
                      for event in selected]}


def observations(items, lanes):
    rows = []
    seen = set()
    for lane in lanes:
        path = str(Path(lane['log']).resolve())
        if path in seen:
            raise ValueError('same evidence log supplied twice: ' + path)
        seen.add(path)
        for identity, title, patterns, status, claim in items:
            cost = measured([event for event in lane['tests'] if any(
                pattern in (event['name'] if '$' in pattern else event['name'].partition('$')[2])
                for pattern in patterns)])
            if cost['frequency'] is not None:
                rows.append(dict(item=identity, title=title, status=status, claim=claim,
                                 log=lane['log'], **cost))
    return sorted(rows, key=lambda row: (-row['work_seconds'], row['item'], row['log']))


def coverage_summary(proof):
    tests = proof.get('tests', [])
    if not tests:
        return {'status': proof.get('status', 'unmeasured'), 'inventory': None,
                'landing': None, 'scheduled_only': None, 'runtime_executed': 'unmeasured'}
    scheduled = sum(bool(row.get('ignored', False)) for row in tests)
    return {'status': proof.get('status', 'unmeasured'), 'inventory': len(tests),
            'landing': len(tests) - scheduled, 'scheduled_only': scheduled,
            'zero': len(proof.get('zero', [])), 'multiple': len(proof.get('multiple', [])),
            'runtime_executed': 'unmeasured; selection proof is not execution evidence'}


def telemetry(current, extra):
    setup, slots, llvm = [], [], []
    raw_paths = [str(Path(lane['log']).resolve()) for lane in extra]
    if len(raw_paths) != len(set(raw_paths)):
        raise ValueError('same runtime/telemetry evidence log supplied twice')
    for lane in current.get('lanes', {}).values():
        if lane.get('log') and str(Path(lane['log']).resolve()) in raw_paths:
            continue
        setup.extend(lane.get('repeated_setup', {}).get('per_lane', []))
        slots.extend(lane.get('slot_use_telemetry', {}).get('tests', []))
    for lane in extra:
        # These events prove construction frequency, not its exclusive duration.
        counts = Counter((row['builder'], row['fingerprint']) for row in lane.get('setup', []))
        setup.extend({'builder': builder, 'fingerprint': fingerprint, 'count': count, 'log': lane['log']}
                     for (builder, fingerprint), count in sorted(counts.items()))
        slots.extend(lane.get('cpu', []))
    for owner, rows in current.get('llvm_lines', {}).items():
        llvm.extend(dict(owner=owner, **row) for row in rows if row.get('name') != 'TOTAL')
    return {'setup': setup, 'slots': slots, 'llvm': llvm}


def retained_notes(workspace, notes):
    """Retain substantive baseline claims while removing superseded census/status prose."""
    if workspace == 'store':
        start = notes.find('## Safe-loop inventory')
        end = notes.find('## Mechanical queue', start)
        return notes[start:end].strip() if start >= 0 and end > start else notes.strip()
    if workspace == 'root':
        start = notes.find('## Cargo contracts requiring prebuild')
        end = notes.find('## Selection hazards', start)
        return notes[start:end].strip() if start >= 0 else notes.strip()
    if workspace == 'query':
        start = notes.find('Concrete sleep examples:')
        end = notes.find('Nearest working prebuild analogue:', start)
        return notes[start:end].strip() if start >= 0 else notes.strip()
    if workspace == 'bank':
        start = notes.find('`workspaces/worth-query-bank-world/crates/worth-query-bank-testkit-estate-certification')
        middle = notes.find('## Concrete wait residue')
        contract = notes[start:notes.find('Nearest working analogue:', start)].strip() if start >= 0 else ''
        waits = notes[middle:notes.find('The initial static scanner reports', middle)].strip() if middle >= 0 else ''
        return contract + '\n\n' + waits
    raise ValueError('unknown governed workspace: ' + workspace)


def generate(workspace, before, current, baseline_lanes=(), runtime_lanes=(), proof=None, notes='',
             telemetry_lanes=(), llvm=()):
    items = ITEMS[workspace]
    baseline = observations(items, baseline_lanes)
    after = observations(items, runtime_lanes)
    source = 'current observed executions' if runtime_lanes else 'baseline only; current runtime unmeasured'
    ranked = after if runtime_lanes else baseline
    for index, row in enumerate(ranked, 1):
        row['rank'] = index
        row['ranking_basis'] = source
    known = {row['item'] for row in ranked}
    unranked = [dict(item=identity, title=title, status=status, claim=claim,
                     reason='No measured per-test work/frequency supplied for this item.')
                for identity, title, _, status, claim in items if identity not in known]
    unranked.append({'item': 'generic-ir', 'title': 'Compile-heavy court/testkit generic helpers',
                     'status': 'remaining', 'claim': 'Preserve typed authorities and authentic fixture construction; erase only work independent of type parameters.',
                     'reason': 'LLVM lines are code-size attribution, not measured compile seconds or runtime work.'})
    result = {'schema': 1, 'workspace': workspace, 'ranking_basis': source,
              'before_source_counts': before.get('source_counts', {}),
              'current_source_counts': current.get('source_counts', {}),
              'coverage': coverage_summary(proof if proof is not None else current.get('coverage', {})),
              'ranked': ranked, 'unranked': unranked, 'baseline': baseline, 'after': after,
              'telemetry': telemetry(current, [*runtime_lanes, *telemetry_lanes]),
              'notes': retained_notes(workspace, notes)}
    for owner, rows in llvm:
        result['telemetry']['llvm'].extend(dict(owner=owner, **row) for row in rows if row.get('name') != 'TOTAL')
    return result


def markdown(result):
    name = result['workspace']
    coverage = result['coverage']
    lines = [f'# {name.title()} test-cost judgment residue', '',
             f'Ranking basis: **{result["ranking_basis"]}**. Rank is sum of observed test `exec_time`: '
             'mean observed cost times execution frequency. It is test work, not lane wall time, CPU time, predicted savings or a budget. '
             'Failed durations remain labeled; ignored tests are excluded.', '',
             '## Current source and selection evidence', '',
             f'Before lexical source counts: `{json.dumps(result["before_source_counts"], sort_keys=True)}`.', '',
             f'Current lexical source counts: `{json.dumps(result["current_source_counts"], sort_keys=True)}`. '
             'These are sites, not tests or compiler sessions.', '',
             f'Native selection proof: **{coverage["status"]}**; inventory {coverage["inventory"] if coverage["inventory"] is not None else "unknown"}, '
             f'landing {coverage["landing"] if coverage["landing"] is not None else "unknown"}, '
             f'scheduled-only {coverage["scheduled_only"] if coverage["scheduled_only"] is not None else "unknown"}. '
             f'Runtime: {coverage["runtime_executed"]}. Ignored tests remain scheduled-only.', '',
             '## Implemented mechanics; runtime certification pending', '',
             '- Compiler/trybuild calls moved to outer prebuilds; tests consume typed prebuilt observations. '
             'The boundary rule and build-census ratchet enforce the captured process boundary.',
             '- Root/Query/Bank gates use the shared prebuilt runner; ordinary root-rest coverage and harness-free main handoff are implemented.',
             '- Positive root/Store fixture programs moved to ordinary executable courts. Store compile-fail batching, '
             'production-authority proof, formal fold and independent sweep pools are implemented. Their actual runtime results remain required.',
             '- Slot recommendations require actual complete CPU accounting. A configured reservation, lexical loop or long duration is not a measurement.', '',
             '## Ranked observed judgment work', '',
             '| Rank | Family | Executions | Work seconds | Failed | Implementation / remaining judgment | Evidence |',
             '|---:|---|---:|---:|---:|---|---|']
    for row in result['ranked']:
        lines.append(f'| {row["rank"]} | {row["title"]} | {row["frequency"]} | {row["work_seconds"]:.3f} | '
                     f'{row["failed"]} | {row["status"]}; {row["claim"]} | `{row["log"]}` |')
    if not result['ranked']:
        lines.append('| unranked | No measured family work supplied | unknown | unknown | unknown | Unranked; obtain real logs | none |')
    lines += ['', '## Unranked judgment', '']
    lines += [f'- **{row["title"]}** ({row["status"]}): {row["reason"]} Claim: {row["claim"]}' for row in result['unranked']]
    telemetry = result['telemetry']
    lines += ['', '## Setup, slots and generic instantiations', '',
              f'Setup fingerprint records: {len(telemetry["setup"])}; CPU/slot records: {len(telemetry["slots"])}; '
              f'LLVM instantiation rows: {len(telemetry["llvm"])}. An empty input is unmeasured, never zero cost.', '']
    for row in sorted(telemetry['setup'], key=lambda row: (-row['count'], row['builder'], row['fingerprint']))[:25]:
        lines.append(f'- `{row["builder"]}` / `{row["fingerprint"]}`: {row["count"]} constructions. '
                     'Exclusive builder time unmeasured; not assigned a seconds rank.')
    for row in sorted(telemetry['llvm'], key=lambda row: (-row['lines'], row['name']))[:20]:
        lines.append(f'- `{row["owner"]}` / `{row["name"]}`: {row["lines"]} IR lines, '
                     f'{row.get("copies", "unknown")} copies. Compile-time savings unmeasured.')
    lines += ['', '## Retained baseline family judgments and handoff', '',
              'The following source review is retained from the initial census. Its source paths/line numbers and '
              'descriptions of then-current signatures are historical; the implementation statuses above supersede its mechanical queue. '
              'It defines the claims and unsafe shortcuts the next fixer must preserve.', '', result['notes'], '',
              '## Stopping point', '',
              'No after-fix speedup is inferred from baseline work. Runtime certification is pending unless actual execution evidence '
              'is supplied above. Next work is the highest measured remaining family, then fingerprint/LLVM attribution for unknown costs. '
              'No sweep sampling, continuation deduplication or ignored execution is authorized.']
    return '\n'.join(lines) + '\n'


def read_inputs(census, explicit):
    paths = [Path(value) for value in explicit]
    if not paths:
        paths = [Path(lane['log']) for lane in census.get('lanes', {}).values()]
    return [read_lane(path) for path in paths]


def read_llvm(value):
    owner, separator, path = value.partition('=')
    if not separator or not owner or not path:
        raise ValueError('LLVM input must be OWNER=PATH')
    return owner, llvm_lines(Path(path).read_text(encoding='utf-8'))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--workspace', choices=ITEMS, required=True)
    parser.add_argument('--before', type=Path, required=True)
    parser.add_argument('--current', type=Path, required=True)
    parser.add_argument('--baseline-log', action='append', default=[])
    parser.add_argument('--runtime-log', action='append', default=[])
    parser.add_argument('--coverage', type=Path)
    parser.add_argument('--telemetry-log', action='append', default=[], help='CPU/setup JSONL; does not imply a test run')
    parser.add_argument('--llvm', action='append', default=[], metavar='OWNER=PATH')
    parser.add_argument('--notes', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    before, current = (json.loads(path.read_text(encoding='utf-8')) for path in (args.before, args.current))
    result = generate(args.workspace, before, current, read_inputs(before, args.baseline_log),
                      read_inputs(current, args.runtime_log),
                      json.loads(args.coverage.read_text()) if args.coverage else None,
                      args.notes.read_text(encoding='utf-8'),
                      telemetry_lanes=read_inputs({}, args.telemetry_log),
                      llvm=[read_llvm(value) for value in args.llvm])
    result['inputs'] = {key: str(value) if isinstance(value, Path) else value for key, value in vars(args).items()}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(markdown(result), encoding='utf-8')
    args.output.with_suffix('.json').write_text(json.dumps(result, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({'workspace': args.workspace, 'ranking_basis': result['ranking_basis'],
                      'ranked': len(result['ranked']), 'unranked': len(result['unranked']), 'coverage': result['coverage']}))
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
