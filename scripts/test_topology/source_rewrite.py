"""Rebase literal includes and compile-time filesystem operands in one scope."""
import json
import os
from pathlib import Path

from manifest_paths import Destinations, rewrite as manifest_rewrite
from module_tree import Node, literal_references
from rust_source import replace_spans


def relpath(path, parent):
    return Path(os.path.relpath(path, parent)).as_posix()


def rewritten_file(node, destination, owner, assignments):
    if node.children:
        raise ValueError('source rebasing requires an extracted lexical module: ' + str(node.source))
    if '_destinations' not in owner:
        owner['_destinations'] = Destinations(owner, assignments)
    text, records = manifest_rewrite(node.text, node.source, destination, owner, owner['_destinations'])
    prepared = Node(node.source, node.names, text)
    changes = []
    for reference in literal_references(prepared, owner['root']):
        if reference['kind'] not in ('include', 'include_str', 'include_bytes'):
            continue
        original = reference['path']
        target = assignments.get(original, {}).get('destination', original)
        start, end = reference['span']
        value = json.dumps(relpath(target, destination.parent))
        if text[start:end] != value:
            changes.append((start, end, value))
            records.append({'source': str(node.source), 'offset': start, 'kind': reference['kind'],
                            'before': text[start:end], 'after': value})
    return replace_spans(text, changes), records
