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
    # +4,000 for Store M4.3e paged CHK24 CheckpointCommitIndex and lazy cold catalog (G7);
    # +5,000 for Store M4.3f (P0-A) paged CHK24 naming/group state, chunked continuation
    # rows, cold retained-fork slicing, commit preflight, and retention compaction;
    # +1,000 for Store M4.3f Round 2 hardening (CowMap AVL tree & per-branch CHK24 validation);
    # +500 for Store M4.3f Round 4 hardening (HolderEpoch retirement tracking, single-domain
    # CHK24 pages, immutable checkpoint naming custody, and Round 4 unit tests);
    # +300 for Store M4.3f Round 6-7 hardening (deadlock-free ACK guard, bounded GROUP_LOG
    # branch digest keys, and read-free VALUE_CHUNK continuation validation);
    # +200 for Store M4.3f Round 10 post-rebase cold merge guards and O(batch) index journal;
    # +1,000 for Store M4.3f Round 11-13 per-branch event/GROUP_LOG window indexes and OrdMap histories;
    # +2,600 for public #455 incremental group authority with its differential oracle court,
    # delta group state rows and WPGL v2 kept-group member deltas; +300 for its
    # contiguous group-member page domain and Rust -> Lean conformance trace test;
    # +400 for the persistent-naming Rust -> Lean conformance trace tests;
    # +200 for the fork_children and CowMap fork Rust -> Lean conformance traces;
    # +1,000 for the Store M4 ChunkedCoverage Rust -> Lean conformance trace tests.
    if all_total > 536_500:
        failures.append(f'combined Total LOC exceeds 536,500: {all_total:,}')
    return failures
