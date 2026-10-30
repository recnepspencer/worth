"""Merge retained CPU telemetry into bounded scheduling policy with sample provenance."""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path

from slot_history import read_history
from policy_catalog import files as catalog_files, index as read_index, read as read_catalog, write as write_catalog
from telemetry import slot_use


def reservation_values(rows):
    result = {}
    for row in rows:
        identity = row['binary'] + '\n' + row['test']
        if identity in result:
            raise ValueError('reservation identity repeats in its catalog')
        result[identity] = row['slots']
    return result


def reservation_rows(values):
    return [dict(binary=key.split('\n', 1)[0], test=key.split('\n', 1)[1], slots=slots)
            for key, slots in sorted(values.items())]


def entries(directory, manifest, collection):
    return read_catalog(directory, manifest['catalogs'][collection]) if manifest else []


def pending_claims(directory):
    return [row['claim'] for row in entries(directory, read_index(directory), 'pending')]


def generate(directory, logs, baseline=None, pending=(), known=None, execution_heads=None, resolved=()):
    directory = Path(directory).resolve()
    explicit = [Path(path).resolve() for path in logs]
    if len(set(explicit)) != len(explicit):
        raise ValueError('explicit telemetry log repeats')
    previous = directory / 'index.json'
    old = read_index(directory)
    old_files = set().union(*(catalog_files(directory, descriptor) for descriptor in old['catalogs'].values())) if old else set()
    # Committed, hashed sample archives carry history into fresh checkouts.
    # Historical target logs are optional; explicitly supplied new logs are real.
    history = read_history(directory, old, explicit, execution_heads)
    peaks, reservations, provenance, samples = {}, {}, [], []
    totals = Counter()
    for source, events in history:
        digest = source['sha256']
        provenance.append(source)
        for index, event in enumerate(events):
            outcome, recommendation, reason = 'unmeasured', None, None
            if type(event.get('exit')) is int and event['exit'] != 0:
                outcome = 'failed-child'
            elif type(event.get('exit')) is not int:
                reason = 'child exit observation missing'
            else:
                try:
                    measured = slot_use([event])
                    if measured['standalone']:
                        outcome = 'standalone-measured'
                    elif measured['tests']:
                        row = measured['tests'][0]
                        recommendation, outcome = row['recommended_slots'], 'measured-success'
                        key = row['binary'] + '\n' + row['test']
                        peaks[key] = max(peaks.get(key, 1), recommendation)
                        reservations[key] = max(reservations.get(key, 1), row['reserved_slots'])
                    else:
                        reason = 'CPU, wall, reservation or accounting posture unavailable'
                except (KeyError, TypeError, ValueError) as error:
                    reason = str(error)
            totals[outcome] += 1
            samples.append({'telemetry_log': source['telemetry_log'], 'capture_sha256': digest, 'sample_index': index,
                            'disposition': outcome, 'recommended_slots': recommendation,
                            'reason': reason, 'observation': event})
    declarations = reservation_values(entries(directory, old, 'baselines'))
    declarations.update(baseline or {})
    pending = sorted(set(pending) | ({row['claim'] for row in entries(directory, old, 'pending')} - set(resolved)))
    pending_keys = {name.replace('$', '\n', 1) for name in pending}
    prior_retired = set(reservation_values(entries(directory, old, 'retired')))
    retired = ((set(peaks) - set(known)) if known is not None else prior_retired) - pending_keys
    active = {key: slots for key, slots in peaks.items() if key not in retired}
    policy = {key: slots for key, slots in active.items()
              if slots > 1 or slots != declarations.get(key, reservations[key])}
    directory.mkdir(parents=True, exist_ok=True)
    catalogs = {
        'sources': write_catalog(directory, 'sources', provenance),
        'samples': write_catalog(directory, 'samples', samples),
        'rules': write_catalog(directory, 'rules', reservation_rows(policy)),
        'baselines': write_catalog(directory, 'baselines', reservation_rows(
            {key: declarations.get(key, reservations[key]) for key in peaks})),
        'retired': write_catalog(directory, 'retired', reservation_rows({key: peaks[key] for key in retired})),
        'pending': write_catalog(directory, 'pending', [dict(claim=claim) for claim in pending]),
    }
    result = {'schema': 2, 'purpose': 'telemetry-only scheduling; never a correctness or timing gate',
              'aggregation': 'maximum recommendation from every successful complete sample per exact identity',
              'status': ('partial' if totals['unmeasured'] or pending else 'measured') if totals['measured-success'] else
                        'standalone-only' if totals['standalone-measured'] else 'unmeasured',
              'complete_samples': totals['measured-success'] + totals['standalone-measured'],
              'scheduler_samples': totals['measured-success'], 'standalone_complete_samples': totals['standalone-measured'],
              'unmeasured_samples': totals['unmeasured'],
              'failed_child_samples': totals['failed-child'], 'sample_count': len(samples),
              'measured_identities': len(peaks), 'policy_identities': len(policy),
              'retired_identities': len(retired),
              'inventory_basis': {'scope': 'explicit complete current inventory', 'identities': len(known),
                  'sha256': hashlib.sha256(json.dumps(sorted(known)).encode()).hexdigest()} if known is not None else old.get('inventory_basis'),
              'source_count': len(provenance), 'pending_claim_count': len(pending), 'catalogs': catalogs}
    temporary = previous.with_suffix('.tmp')
    temporary.write_text(json.dumps(result, indent=2) + '\n', encoding='utf-8')
    temporary.replace(previous)
    current_files = set().union(*(catalog_files(directory, descriptor) for descriptor in catalogs.values()))
    for name in sorted(old_files - current_files):
        path = (directory / name).resolve()
        if path.parent != directory:
            raise ValueError('retired catalog path escapes its owner')
        path.unlink(missing_ok=True)
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--telemetry', action='append', type=Path, required=True)
    parser.add_argument('--baseline', type=Path)
    parser.add_argument('--pending', action='append', default=[])
    parser.add_argument('--resolved', action='append', default=[], help='claims resolved by independently verified execution lineage')
    args = parser.parse_args()
    baseline = json.loads(args.baseline.read_text()) if args.baseline else None
    result = generate(args.output, args.telemetry, baseline, args.pending, resolved=args.resolved)
    print(json.dumps({key: result[key] for key in ('status', 'sample_count', 'complete_samples',
        'unmeasured_samples', 'failed_child_samples', 'measured_identities', 'policy_identities', 'pending_claim_count')}))
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
