"""Relocate literal source-file references held by product readiness catalogues."""
from manifest_paths import Destinations, rewrite


def relocate(owners, assignments, effects):
    receipts = []
    for owner in owners:
        destinations = owner.setdefault('_destinations', Destinations(owner, assignments))
        for source in sorted((owner['root'] / 'src').rglob('*.rs')):
            text = source.read_text(encoding='utf-8')
            if '.rs"' not in text:
                continue
            updated, records = rewrite(text, source, source, owner, destinations)
            if records:
                effects.write(source, updated)
                receipts.extend(records)
    return receipts
