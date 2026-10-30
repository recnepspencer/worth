"""Combine native and rustdoc inventories without conflating their test owners."""
from coverage import inventory, prove


def merge(discovered, selections, documentation):
    """Return complete inventory, lane selections, and the structural proof.

    Rustdoc execution supplies exact names that nextest cannot discover. Its
    ignored snippets remain inventoried but are never landing selections.
    Execution outcomes are verified separately from this selection proof.
    """
    known = inventory(discovered)
    chosen = {lane: inventory(value) for lane, value in selections.items()}
    if documentation is not None:
        if 'documentation' in chosen:
            raise ValueError('documentation lane already exists')
        docs = inventory(documentation['inventory'])
        selected = inventory(documentation['selections'])
        for row in docs:
            if not row['binary'].endswith('::doc'):
                raise ValueError('documentation inventory has a non-documentation owner')
        expected = {(row['binary'], row['test']) for row in docs if not row['ignored']}
        observed = {(row['binary'], row['test']) for row in selected}
        if expected != observed or any(row['ignored'] or not row.get('selected', True) for row in selected):
            raise ValueError('documentation selections must contain every nonignored snippet exactly once')
        known = inventory([*known, *docs])
        chosen['documentation'] = selected
    return known, chosen, prove(known, chosen)
