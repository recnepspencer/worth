"""Exact lane multiplicity from nextest's discovered and selected inventories."""
from collections import Counter


def inventory(value):
    if isinstance(value, list):
        rows = value
    else:
        suites = value.get('rust-suites', {})
        rows = [dict(binary=binary, test=test, ignored=case['ignored'],
                     selected=case.get('filter-match', {'status': 'matches'})['status'] == 'matches')
                for binary, suite in suites.items() for test, case in suite['testcases'].items()]
    identities = [(row['binary'], row['test']) for row in rows]
    if len(set(identities)) != len(identities):
        raise ValueError('inventory repeats a test identity')
    return rows


def prove(discovered, selections):
    discovered = inventory(discovered)
    known = {(r['binary'], r['test']): r for r in discovered}
    counts = Counter()
    lanes = {}
    for lane, value in sorted(selections.items()):
        selected = [r for r in inventory(value) if r.get('selected', True) and not r['ignored']]
        identities = [(r['binary'], r['test']) for r in selected]
        unknown = set(identities) - set(known)
        if unknown:
            raise ValueError(f'{lane} selects tests missing from inventory: {sorted(unknown)}')
        lanes[lane] = identities
        counts.update(identities)
    rows = [dict(row, selections=counts[key], lanes=[lane for lane, tests in lanes.items() if key in tests],
                 disposition='scheduled-only' if row['ignored'] else 'landing')
            for key, row in sorted(known.items())]
    failures = [r for r in rows if not r['ignored'] and r['selections'] != 1]
    return {'status': 'pass' if not failures else 'fail', 'tests': rows,
            'zero': [r for r in failures if r['selections'] == 0],
            'multiple': [r for r in failures if r['selections'] > 1],
            'scheduled_only': sum(r['ignored'] for r in rows)}
