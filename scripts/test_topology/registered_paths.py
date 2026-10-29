"""Relocate closed source registrations to every actual compiled source slice."""
import re

from plan_effects import relative
from rust_source import replace_spans


def relocate(effects, repository, assignments, compiled_sources=()):
    mapping = {relative(source, repository): {relative(row['destination'], repository)}
               for source, row in assignments.items()}
    compiled = {}
    for row in compiled_sources:
        compiled.setdefault(row['source'], set()).add(row['destination'])
    mapping.update(compiled)
    rewrites = []
    config = repository / 'tools/boundary-check/config'
    if not config.exists():
        return rewrites, []
    pattern = re.compile(r'(?:crates|workspaces)/[A-Za-z0-9_./-]+')
    for path in sorted(config.iterdir()):
        if path.suffix not in ('.txt', '.toml', '.json'):
            continue
        text = path.read_text(encoding='utf-8')
        changes, offset = [], 0
        for line in text.splitlines(keepends=True):
            variants = [line]
            for match in pattern.finditer(line):
                old = match[0]
                destinations = mapping.get(old)
                if destinations is None and '/tests/' in old:
                    destinations = set().union(*(values for source, values in mapping.items()
                        if source.startswith(old.rstrip('/') + '/')))
                    destinations = {value for value in destinations if value.endswith('.rs')}
                if not destinations:
                    continue
                if len(destinations) > 1 and path.suffix != '.txt':
                    raise ValueError('one-to-many registered source needs an explicit format ruling: ' + str(path) + ': ' + old)
                variants = [variant.replace(old, destination) for variant in variants for destination in sorted(destinations)]
            updated = ''.join(dict.fromkeys(variants))
            if updated != line:
                changes.append((offset, offset + len(line), updated))
                rewrites.append({'kind': 'registry-path', 'source': str(path), 'offset': offset,
                                 'before': line, 'after': updated})
            offset += len(line)
        if changes:
            effects.write(path, replace_spans(text, changes))
    return rewrites, []
