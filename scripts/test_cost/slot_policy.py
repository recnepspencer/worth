"""Merge retained CPU telemetry into bounded scheduling policy with sample provenance."""
import argparse
from collections import Counter, defaultdict
import hashlib
import json
import os
from pathlib import Path

from slot_history import archive_digest, read_history
from telemetry import slot_use


def owned(directory, name):
    path = (directory / name).resolve()
    if path.parent != directory:
        raise ValueError('generated policy file escapes its owner: ' + name)
    return path


def write(path, text):
    temporary = path.with_suffix(path.suffix + '.tmp')
    temporary.write_text(text, encoding='utf-8')
    os.replace(temporary, path)


def write_shards(directory, values, prefix=''):
    by_binary = defaultdict(dict)
    for key, slots in sorted(values.items()):
        binary, test = key.split('\n', 1)
        by_binary[binary][test] = slots
    files = []
    for binary, tests in sorted(by_binary.items()):
        digest, rows = hashlib.sha256(binary.encode()).hexdigest()[:20], sorted(tests.items())
        for offset in range(0, len(rows), 100):
            name = f'{prefix}{digest}-{offset // 100:03}.json'
            files.append(name)
            write(owned(directory, name), json.dumps({'binary': binary, 'tests': dict(rows[offset:offset + 100])}, indent=2) + '\n')
    return files


def generate(directory, logs, baseline=None, pending=(), known=None, execution_heads=None, resolved=()):
    directory = Path(directory).resolve()
    explicit = [Path(path).resolve() for path in logs]
    if len(set(explicit)) != len(explicit):
        raise ValueError('explicit telemetry log repeats')
    previous = directory / 'index.json'
    old = json.loads(previous.read_text()) if previous.exists() else {}
    if old and old.get('schema') != 1:
        raise ValueError('unsupported measured-reservation schema')
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
                    if measured['tests']:
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
    declarations = {}
    for name in old.get('baseline_files', []):
        shard = json.loads(owned(directory, name).read_text())
        declarations.update({shard['binary'] + '\n' + test: slots for test, slots in shard['tests'].items()})
    declarations.update(baseline or {})
    # Resolution is explicit verified-lineage input, independent of CPU samples.
    # A claim explicitly marked pending in this call remains pending.
    pending = sorted(set(pending) | (set(old.get('pending_claims', [])) - set(resolved)))
    pending_keys = {name.replace('$', '\n', 1) for name in pending}
    prior_retired = set()
    for name in old.get('retired_files', []):
        shard = json.loads(owned(directory, name).read_text())
        prior_retired.update(shard['binary'] + '\n' + test for test in shard['tests'])
    retired = ((set(peaks) - set(known)) if known is not None else prior_retired) - pending_keys
    active = {key: slots for key, slots in peaks.items() if key not in retired}
    policy = {key: slots for key, slots in active.items()
              if slots > 1 or slots != declarations.get(key, reservations[key])}
    directory.mkdir(parents=True, exist_ok=True)
    files = write_shards(directory, policy)
    baseline_files = write_shards(directory, {key: declarations.get(key, reservations[key]) for key in peaks}, 'baseline-')
    retired_files = write_shards(directory, {key: peaks[key] for key in retired}, 'retired-')
    archives, archive_hashes = [], {}
    for offset in range(0, len(samples), 100):
        name = f'observations-{offset // 100:03}.jsonl'
        archives.append(name)
        write(owned(directory, name), ''.join(json.dumps(sample, sort_keys=True) + '\n' for sample in samples[offset:offset + 100]))
        archive_hashes[name] = archive_digest(owned(directory, name).read_bytes())
    result = {'schema': 1, 'purpose': 'telemetry-only scheduling; never a correctness or timing gate',
              'aggregation': 'maximum recommendation from every successful complete sample per exact identity',
              'status': 'unmeasured' if not totals['measured-success'] else 'partial' if totals['unmeasured'] or pending else 'measured',
              'complete_samples': totals['measured-success'], 'unmeasured_samples': totals['unmeasured'],
              'failed_child_samples': totals['failed-child'], 'sample_count': len(samples),
              'measured_identities': len(peaks), 'policy_identities': len(policy),
              'retired_identities': len(retired),
              'inventory_basis': {'scope': 'explicit complete current inventory', 'identities': len(known),
                  'sha256': hashlib.sha256(json.dumps(sorted(known)).encode()).hexdigest()} if known is not None else old.get('inventory_basis'),
              'observations': provenance, 'pending_claims': pending,
              'files': files, 'baseline_files': baseline_files, 'retired_files': retired_files,
              'observation_files': archives, 'observation_sha256': archive_hashes}
    write(previous, json.dumps(result, indent=2) + '\n')
    old_files = old.get('files', []) + old.get('observation_files', []) + old.get('baseline_files', []) + old.get('retired_files', [])
    for name in sorted(set(old_files) - set(files + archives + baseline_files + retired_files)):
        owned(directory, name).unlink(missing_ok=True)
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
        'unmeasured_samples', 'failed_child_samples', 'measured_identities', 'policy_identities', 'pending_claims')}))
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
