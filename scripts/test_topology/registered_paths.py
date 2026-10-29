"""Relocate exact registered source paths without broadening caller prefixes."""
import re

from plan_effects import relative
from rust_source import replace_spans


def relocate(effects, repository, assignments):
    mapping = {relative(source, repository): relative(row['destination'], repository)
               for source, row in assignments.items()}
    rewrites, unresolved = [], []
    config = repository / 'tools/boundary-check/config'
    if not config.exists():
        return rewrites, unresolved
    pattern = re.compile(r'(?:crates|workspaces)/[A-Za-z0-9_./-]+')
    for path in sorted(config.iterdir()):
        if path.suffix not in ('.txt', '.toml', '.json'):
            continue
        text = path.read_text(encoding='utf-8')
        changes = []
        for match in pattern.finditer(text):
            old = match[0]
            if old in mapping:
                changes.append((match.start(), match.end(), mapping[old]))
                rewrites.append({'kind': 'registry-path', 'source': str(path), 'offset': match.start(),
                                 'before': old, 'after': mapping[old]})
            elif '/tests/' in old:
                covered = [source for source in mapping if source.startswith(old.rstrip('/') + '/')]
                if covered:
                    unresolved.append({'registry': relative(path, repository), 'prefix': old,
                                       'sources': sorted(covered)})
        if changes:
            effects.write(path, replace_spans(text, changes))
    return rewrites, unresolved
