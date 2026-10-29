"""Plan only module mounts and literal path edits; retain all other source bytes."""
import json
import os
from pathlib import Path
import re

from module_tree import literal_references
from rust_source import replace_spans


def file_alias(path, owner):
    relative = path.relative_to(owner['root'] / 'tests').with_suffix('')
    return '_'.join(relative.parts).replace('-', '_')


def relpath(path, parent):
    return Path(os.path.relpath(path, parent)).as_posix()


def rewrite_node(node, destination, owner, assignments, aliases, current_kit=None):
    changes, records = [], []

    def edit(start, end, replacement, kind):
        if node.text[start:end] != replacement:
            changes.append((start, end, replacement))
            records.append({'source': str(node.source), 'offset': node.offset + start,
                            'before': node.text[start:end], 'after': replacement, 'kind': kind})

    for child in node.children:
        item = child.declaration
        if item.body:
            continue
        assignment = assignments.get(child.source)
        target = assignment['destination'] if assignment else child.source
        foreign_kit = assignment and '-testkit' in assignment['owner'] and child.source in aliases
        canonical_child = (current_kit and foreign_kit and assignment['owner'] == current_kit
                           and aliases.get(child.source) == aliases.get(node.source, '') + '::' + item.name)
        if foreign_kit and not canonical_child:
            prefix = 'crate' if assignment['owner'] == current_kit else assignment['owner'].replace('-', '_')
            alias = aliases[child.source]
            original = node.text[item.start:item.end]
            visibility = re.search(r'\b(pub(?:\s*\([^)]*\))?\s+)?mod\s+', original)
            visibility = visibility[1] or ''
            attributes = re.sub(r'#\s*\[\s*path\s*=\s*(?:r#*".*?"#*|"(?:\\.|[^"\\])*")\s*\]\s*', '', item.attributes, flags=re.S)
            replacement = attributes + visibility + 'use ' + prefix + '::' + alias + ' as ' + item.name + ';'
            edit(item.start, item.end, replacement, 'module-testkit-import')
        else:
            value = json.dumps(relpath(target, destination.parent))
            if item.path_span:
                edit(*item.path_span, value, 'module-path')
            else:
                edit(item.start, item.start, '#[path = ' + value + ']\n', 'module-path')
    module_spans = [(child.declaration.start, child.declaration.end) for child in node.children]
    for reference in literal_references(node, owner['root']):
        start, end = (v - node.offset for v in reference['span'])
        if any(a <= start < b for a, b in module_spans):
            continue
        original = reference['path']
        assignment = assignments.get(original)
        if assignment:
            target = assignment['destination']
        elif '*' in str(original):
            prefix = str(original).split('*', 1)[0]
            matched = [(p, value) for p, value in assignments.items() if str(p).startswith(prefix)]
            if not matched:
                continue
            p, value = matched[0]
            suffix = str(p)[len(prefix):]
            target = Path(str(value['destination'])[:-len(suffix)] + str(original)[len(prefix):]) if suffix else value['destination']
        elif original.exists():
            target = original
        else:
            continue
        parent = destination.parent
        if reference['kind'] in ('manifest-path', 'test-path'):
            parent = next(p for p in destination.parents if p.parent.name == 'crates')
        value = relpath(target, parent)
        if reference.get('leading_slash'):
            value = '/' + value
        edit(start, end, json.dumps(value), reference['kind'])
    return replace_spans(node.text, changes), records


def rewritten_file(node, destination, owner, assignments, aliases, current_kit=None):
    """Apply recursive inline-scope edits to a single physical source file."""
    text, records = rewrite_node(node, destination, owner, assignments, aliases, current_kit)
    # Rewriting inline bodies uses the original parent's offsets, so collect their
    # absolute edits and apply once to the original file rather than shifted text.
    if any(child.declaration.body for child in node.children):
        changes = []
        for record in records:
            start = record['offset'] - node.offset
            changes.append((start, start + len(record['before']), record['after']))
        def descend(parent, inline_names):
            for child in parent.children:
                if not child.declaration.body:
                    continue
                child_destination = destination.parent.joinpath(*inline_names, child.names[-1], destination.name)
                _text, child_records = rewrite_node(child, child_destination, owner, assignments, aliases, current_kit)
                for record in child_records:
                    start = record['offset'] - node.offset
                    changes.append((start, start + len(record['before']), record['after']))
                records.extend(child_records)
                descend(child, inline_names + (child.names[-1],))
        descend(node, ())
        text = replace_spans(node.text, changes)
    return text, records
