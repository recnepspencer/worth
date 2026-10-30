"""Non-gating CPU, setup frequency and lane timing observations."""
from collections import Counter, defaultdict
from enum import Enum
import math


class SchedulerKind(Enum):
    NEXTEST = 'nextest'
    STANDALONE = 'standalone-process'


def scheduler(event):
    # This sentinel is assigned only to harness-free processes by standalone.py.
    if event.get('test') == '<main>':
        return SchedulerKind.STANDALONE
    return SchedulerKind(event.get('scheduler', SchedulerKind.NEXTEST.value))


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
    rows, unknown, standalone = [], [], []
    for event in events:
        wall, cpu = event.get('wall_seconds'), event.get('cpu_seconds')
        try:
            kind = scheduler(event)
        except ValueError:
            unknown.append(dict(event, reason='unrecognized process scheduler'))
            continue
        if cpu is None or event.get('accounting', 'complete') != 'complete':
            unknown.append(event)
            continue
        if not all(type(n) in (int, float) and math.isfinite(n) and n >= 0 for n in (wall, cpu)) or wall == 0:
            unknown.append(dict(event, reason='invalid CPU accounting observation'))
            continue
        parallelism = cpu / wall
        if kind is SchedulerKind.STANDALONE:
            standalone.append(dict(event, scheduler=kind.value, disposition='standalone-measured',
                                   reservation_scope='not-nextest-managed', measured_parallelism=parallelism))
            continue
        reserved = event.get('reserved_slots')
        if type(reserved) is not int or reserved < 1:
            unknown.append(dict(event, reason='actual scheduler reservation unavailable'))
            continue
        rows.append(dict(event, measured_parallelism=parallelism,
                         recommended_slots=max(1, math.ceil(parallelism)),
                         over_reserved=reserved > max(1, parallelism) * 2))
    return {'status': 'telemetry' if rows else 'standalone-only' if standalone else 'unmeasured',
            'tests': rows, 'standalone': standalone, 'unmeasured': unknown}


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
    if not tests and not lane.get('suites') and lane.get('plain_test_counts') is not None:
        summary.update(lane['plain_test_counts'])
    # Sum(exec_time) is work, never lane wall time or a build-time subtraction.
    summary.setdefault('build_seconds', None)
    summary.setdefault('run_seconds', None)
    return summary
