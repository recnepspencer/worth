"""Rebase relative paths inside nested Cargo fixture projects after relocation."""
import json
from pathlib import Path
import re

from source_rewrite import relpath


def rewrite(source, destination, assignments):
    text = source.read_text(encoding='utf-8')
    changes, records = [], []
    for match in re.finditer(r'\bpath\s*=\s*"([^"\n]+)"', text):
        original = (source.parent / match[1]).resolve()
        if (original / 'Cargo.toml') in assignments:
            target = assignments[original / 'Cargo.toml']['destination'].parent
        elif original in assignments:
            target = assignments[original]['destination']
        else:
            target = original
        value = relpath(target, destination.parent)
        if value != match[1]:
            start, end = match.span(1)
            changes.append((start, end, value))
            records.append({'source': str(source), 'offset': start, 'before': match[1],
                            'after': value, 'kind': 'fixture-manifest-path'})
    from rust_source import replace_spans
    return replace_spans(text, changes), records
