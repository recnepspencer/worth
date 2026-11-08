"""Read completed evidence only; never execute or alter a test selection."""
from pathlib import Path
import collections
import json
import re

r = Path.cwd()
e = r / 'target/lane-evidence'
prior = json.loads((e / 'pa-merge-final-families.json').read_text())
previous = {row['name']: row for row in prior['outcomes']}
rows = []
failures = []
for ws in ('root', 'query', 'bank', 'fixture', 'store'):
    folder = e / f'pc-merge-{ws}-tier2'
    if not folder.exists():
        continue
    for path in sorted(folder.rglob('*.log')):
        # Build/prerequisite/metadata captures contain no test-body verdicts.
        counts = collections.Counter()
        events = []
        text = path.read_text(encoding='utf-8', errors='replace')
        for line in text.splitlines():
            if not line.startswith('{'):
                continue
            try:
                event = json.loads(line)
            except ValueError:
                continue
            if event.get('type') == 'test' and event.get('event') in ('ok', 'failed', 'ignored'):
                counts[event['event']] += 1
                events.append(event)
        if not counts:
            continue
        exits = re.findall(r'^(?:Exit code: |EXIT: )(\d+)', text, re.M)
        row = {'workspace': ws, 'log': path.relative_to(r).as_posix(),
               'counts': dict(counts), 'exit': int(exits[-1]) if exits else None}
        rows.append(row)
        for event in events:
            if event['event'] != 'failed':
                continue
            old = previous.get(event['name'])
            failures.append({'workspace': ws, 'name': event['name'], 'log': row['log'],
                             'failure': event.get('stdout', '') + event.get('stderr', ''),
                             'prior_class': old.get('final_class', old.get('class')) if old else None,
                             'prior_state': old.get('final_state', old.get('state')) if old else None})
# Legacy fixture fragments use ordinary libtest text rather than JSON events.
fixture = e / 'pc-merge-fixture-tier2/fixture.json'
if fixture.exists():
    suite = json.loads(fixture.read_text())
    for chunk in suite['chunks']:
        path = fixture.parent / chunk['log']
        body = path.read_text(errors='replace')
        counts = collections.Counter(verdict for _, verdict in re.findall(
            r'^test (\S+) \.\.\. (ok|FAILED|ignored)\r?$', body, re.M))
        rows.append({'workspace': 'fixture', 'log': path.relative_to(r).as_posix(),
                     'counts': dict(counts), 'exit': chunk['exit'],
                     'parent_log': 'target/lane-evidence/pc-merge-fixture-tier2-driver.log'})
        for name in chunk['failed']:
            old = previous.get(name)
            marker = re.search(r"(?m)^thread '" + re.escape(name) + r"'[^\n]*panicked", body)
            evidence = body[marker.start():] if marker else body
            failures.append({'workspace': 'fixture', 'name': name,
                             'log': path.relative_to(r).as_posix(), 'failure': evidence,
                             'prior_class': old.get('final_class', old.get('class')) if old else None,
                             'prior_state': old.get('final_state', old.get('state')) if old else None,
                             'assertion_output_deferred_by_interrupted_harness': marker is None})

# These are audited deliberate campaign interruptions, never asserted failures.
for filename, name, classification, rawlog in [
    ('pc-merge-charge-sweep-abort.json',
     'domain_computation::primary_graph::application_attempt::compare_and_commit::commit_outcome::denial::execution::authority_cause::tests::authority_evidence_exposes_request_resources_by_value_through_the_commit_door',
     'CLEANUP-C2-incomplete-charge-prefix-sweep',
     'target/lane-evidence/pc-merge-query-tier2/test-cost/query/run-query-rest.log'),
    ('pc-merge-cancellation-sweep-abort.json',
     'process::host::durable_retry::request_stops::original_directory_receipt_preserves_cancellation_at_every_consult',
     'CLEANUP-C1-incomplete-consult-cancellation-sweep',
     'target/lane-evidence/pc-merge-query-tier2/test-cost/query/run-query-process.log'),
    ('pc-merge-geometry-campaign-abort.json',
     'application_invariant_acceptance::managed_index::maintained::geometry::court::growth::geometry_chain_growth_uses_real_source_and_original_sdk_invoice',
     'CLEANUP-C2-incomplete-growth-campaign',
     'target/lane-evidence/pc-merge-fixture-tier2/fixture-chunk-004.log'),
]:
    audit = e / filename
    if not audit.exists():
        continue
    matches = [row for row in failures if row['name'].split('$')[-1] == name]
    assert len(matches) <= 1
    if matches:
        row = matches[0]
    else:
        row = {'workspace': 'fixture', 'name': name, 'log': rawlog, 'failure': ''}
        failures.append(row)
    row.update(outcome='incomplete', prior_class=classification, prior_state='retained red',
               interruption_audit=audit.relative_to(r).as_posix())

result = {'head': 'fbbdf5edff + exact P-C 80543019fb resolved source',
          'note': 'Partial until the serial Tier-2 driver finishes. Ignored counts are metadata, never executed bodies.',
          'lanes': rows, 'failures': failures}
(e / 'pc-merge-raw-result-census.json').write_text(json.dumps(result, indent=2))
print('Completed body logs:', len(rows), 'failure events:', len(failures))
for row in failures:
    print(row['workspace'], row['prior_class'] or 'NEW/UNCLASSIFIED', row['name'])
