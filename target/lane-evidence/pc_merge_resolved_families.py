"""Append focused repair outcomes; retain the original complete merge census."""
from pathlib import Path
import collections
import json

r = Path.cwd()
e = r / 'target/lane-evidence'
data = json.loads((e / 'pc-merge-families.json').read_text())
for row in data['failures']:
    family = row['family']
    if family == 'HAPPY-content-data-before-codecs':
        row.update(state='green', final_log='target/lane-evidence/pc-merge-repairs/query/run.log')
    elif family == 'HAPPY-observer-prebuild':
        row.update(state='green', final_log='target/lane-evidence/pc-merge-repairs/store/run.log')
    elif family == 'HAPPY-recovery-construction-contract':
        row.update(state='green', final_log='target/lane-evidence/pc-merge-privacy-final/run.log')
    elif family == 'HAPPY-fixture-finite-capture-funding':
        row.update(state='blocked optional funding-provider boundary', owner='Topology / Query funding-provider owner')
    elif family.startswith(('HAPPY-HELD', 'HAPPY_HELD')):
        row['state'] = 'held owner work'
        if 'unaccounted-capture' in family:
            row['owner'] = 'World / Query opening funding owners (previously held)'
    else:
        row['state'] = 'recorded cleanup red/incomplete; not broadly rerun after the repair batch'
data['repair_commits'] = json.loads((e / 'pc-merge-repair-commits.json').read_text())
data['final_states'] = collections.Counter(row['state'] for row in data['failures'])
data['lint_resolution'] = {'expression_equivalent_sites': 34, 'files': 8,
                           'held_native_dead_code': 32, 'scoped_lint_validation': 'follows stack verification'}
(e / 'pc-merge-resolved-families.json').write_text(json.dumps(data, indent=2))
lines = ['# P-C merge qualification and repair families', '',
    'Census source: e787b51b3e. Repair source:592aaaae2f. This records a partial Tier2 selection, not milestone certification.', '',
    '| Family | Cases | Resolution |', '|---|---:|---|']
for family, count in sorted(data['totals'].items()):
    states = sorted({row['state'] for row in data['failures'] if row['family'] == family})
    lines.append(f'| {family} | {count} | ' + '; '.join(states) + ' |')
lines += ['', '34 expression-equivalent lint sites in8files are committed.32 Native dead-code diagnostics stay P-A held.',
    '', 'The privacy inventory initially compared only headers and missed Reopened constructor help. A bulk diagnostic pass through the actual normalizer identified exactly that remaining mismatch. The final court retains58 probes:54 snapshots and4 semantic privacy contracts, with unchanged complete dependency audiences.',
    '', 'Focused results: Native6pass/0fail/1ignored-metadata; Query1/0/0; Store12/1/0 initially, then its sole failed privacy court1/0/0. Five observer journeys, six transition-index journeys and the epoch-refusal court pass.',
    '', 'Raw commands, HEAD, dirty status, times and exits are in each cited log. Counts include no ignored body execution. The original100 non-successes are preserved in pc-merge-families.json and full raw logs.',
    '', 'Optional Topology funding-provider and prior principal-resolution exhaustion are not repaired. The current Topology setup refuses before that prior principal probe can be reached.', '']
(e / 'pc-merge-resolution-families.md').write_text('\n'.join(lines))
cleanup = ['# P-C merge cleanup reds', '',
    'Observed at e787b51b3e; retained under Delivery tracks C1/C2/C4. These85 cases include3 audited incomplete campaigns. Do not infer final-source green or rerun coverage.', '']
for row in data['failures']:
    if not row['family'].startswith('CLEANUP'):
        continue
    text = row['failure']
    if len(text) > 14000:
        text = text[:3000] + '\n[Projection abbreviated here; full original output remains in the cited log.]\n' + text[-9000:]
    cleanup += ['## `' + row['name'] + '`', '', row['family'] + '; owner: ' + row['owner'] + '.', '',
                row['reason'], '', 'Log: `' + row['log'] + '`.', '', '```text',
                text or 'Audited incomplete campaign; see interruption_audit in the JSON record.', '```', '']
(e / 'pc-merge-cleanup-reds.md').write_text('\n'.join(cleanup))
print(json.dumps(data['final_states'], indent=2))
