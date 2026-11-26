"""Measure Relational owners against their production floors and source caps."""
from collections import Counter

from relational_dag_rust import ProductionView


def check_owner_sizes(audit, examples):
    """Count each Relational owner using the split's source and production measures."""
    total, production = Counter(), Counter()
    for source, owner in audit.file_owners.items():
        if source.suffix == '.rs' and owner in audit.map.owners:
            total[owner] += len(source.read_bytes().splitlines())
    example_lines = sum(len(path.read_bytes().splitlines()) for path in examples.rglob('*.rs'))
    total['runtime'] += example_lines
    for module in audit.model.views:
        owner, _ = audit.locations[module]
        if owner in audit.map.owners:
            production[owner] += ProductionView(audit.model.index[module]).own_text().count('\n')
    all_total, all_production = total.total(), production.total()
    print('Owner         Placement    Total LOC    Prod LOC  Prod floor   Total cap')
    failures = []
    for owner, budget in audit.map.owners.items():
        actual, prod = total[owner], production[owner]
        floor, cap = budget['floor'], budget['cap']
        placement = 'physical' if budget.get('physical') or owner == 'runtime' else 'virtual'
        print(f'{owner:<12} {placement:>10} {actual:>11,} {prod:>11,} {floor:>11,} {cap:>11,}')
        if prod < floor or actual > cap:
            failures.append(f'{owner} must meet its production floor and total cap')
        if 100 * actual > 23 * all_total or 100 * prod > 23 * all_production:
            failures.append(f'{owner} exceeds the 23.0% Total or Prod share')
    print(f'{"combined":<12} {"":>10} {all_total:>11,} {all_production:>11,}')
    print(f'Runtime Total includes {example_lines:,} example lines')
    # +5,000 for Persistent Naming (#421) and Store M4.3b CHK24/REG8 paged record identity;
    # +4,000 for Store M4.3e paged CHK24 CheckpointCommitIndex and lazy cold catalog (G7).
    if all_total > 524_000:
        failures.append(f'combined Total LOC exceeds 524,000: {all_total:,}')
    return failures
