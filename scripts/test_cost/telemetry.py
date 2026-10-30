"""Non-gating CPU, setup frequency and lane timing observations."""
from collections import Counter, defaultdict
import math


def setup_counts(events):
    per_binary, per_lane = Counter(), Counter()
    for event in events:
        key = (event['builder'], event['fingerprint'])
        per_lane[key] += 1
        per_binary[(event['binary'], *key)] += 1
    return {'status': 'measured' if events else 'unmeasured',
            'per_binary': [dict(binary=binary, builder=builder, fingerprint=fingerprint, count=count)
                           for (binary, builder, fingerprint), count in sorted(per_binary.items())],
            'per_lane': [dict(builder=builder, fingerprint=fingerprint, count=count)
                         for (builder, fingerprint), count in sorted(per_lane.items())]}


def slot_use(events):
    rows, unknown = [], []
    for event in events:
        wall, cpu, reserved = event['wall_seconds'], event['cpu_seconds'], event['reserved_slots']
        if cpu is None or reserved is None or event.get('accounting', 'complete') != 'complete':
            unknown.append(event)
            continue
        if not all(math.isfinite(n) and n >= 0 for n in (wall, cpu)) or reserved < 1 or wall == 0:
            unknown.append(dict(event, reason='invalid CPU accounting observation'))
            continue
        parallelism = cpu / wall
        rows.append(dict(event, measured_parallelism=parallelism,
                         recommended_slots=max(1, math.ceil(parallelism)),
                         over_reserved=reserved > max(1, parallelism) * 2))
    return {'status': 'telemetry' if rows else 'unmeasured', 'tests': rows, 'unmeasured': unknown}


def timing_summary(lane):
    tests = lane['tests']
    summary = {'pass': sum(t['event'] == 'ok' for t in tests),
               'fail': sum(t['event'] == 'failed' for t in tests),
               'ignored': (sum(t['event'] == 'ignored' for t in tests) or
                           sum(suite.get('ignored', 0) for suite in lane.get('suites', []))),
               **lane['telemetry']}
    summary['slowest'] = sorted([{'name': t['name'], 'seconds': t['exec_time'],
                                 'event': t['event']} for t in tests if 'exec_time' in t],
                                key=lambda t: (-t['seconds'], t['name']))[:40]
    # Sum(exec_time) is work, never lane wall time or a build-time subtraction.
    summary.setdefault('build_seconds', None)
    summary.setdefault('run_seconds', None)
    return summary
