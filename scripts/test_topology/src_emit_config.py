"""Boundary configuration that follows the moved tests.

- `src-court-baseline.json` loses every entry whose source module moved (it only shrinks);
- `src-court-private-logic-audit.json` loses admissions for tests that left `src`;
- `control-callers.txt` follows moved callers to their new files, and registers new court
  and testkit files that name a library-owned `*_for_test`/`*ForTest` door;
- `court-frontier-splits.toml` registers every court that shares a frontier with a sibling.
"""
import json
import re

TEST_DOOR = re.compile(r'\b\w+(?:_for_test|ForTest)\b')
LIBRARY_OWNER = 'workspaces/worth-query/crates/worth-query-execution/'


def shrink_baseline(text, moved_sources, moved_inline):
    rows = json.loads(text)
    kept = [r for r in rows if r['source'] not in moved_sources
            and (r['source'], r['module'].split('::')[-1]) not in moved_inline]
    lines = ',\n'.join(json.dumps(r, separators=(',', ':')) for r in kept)
    return '[\n' + lines + '\n]\n', len(rows) - len(kept)


def shrink_admissions(text, moved_sources):
    data = json.loads(text)
    before = len(data['audits'])
    data['audits'] = [a for a in data['audits'] if a['source'] not in moved_sources]
    return json.dumps(data, indent=2) + '\n', before - len(data['audits'])


def control_callers(text, relocated, new_files, library_doors, door_owners=None):
    """relocated: old caller path -> new file paths; new_files: {path: content}."""
    out, seen = [], set()
    for line in text.splitlines():
        parts = line.split(' ')
        if len(parts) == 2 and parts[1] in relocated:
            for target in relocated[parts[1]]:
                if TEST_DOOR.search(new_files.get(target, '')):
                    entry = f'{parts[0]} {target}'
                    if entry not in seen:
                        out.append(entry)
                        seen.add(entry)
            continue
        out.append(line)
        seen.add(line)
    additions = []
    for path, content in sorted(new_files.items()):
        if set(TEST_DOOR.findall(content)) & library_doors:
            entry = f'{LIBRARY_OWNER} {path}'
            if entry not in seen:
                additions.append(entry)
                seen.add(entry)
    for trait, owner in sorted((door_owners or {}).items()):
        for path, content in sorted(new_files.items()):
            if not path.startswith(owner) and re.search(r'\b' + trait + r'\b', content):
                entry = f'{owner} {path}'
                if entry not in seen:
                    additions.append(entry)
                    seen.add(entry)
    if additions:
        out.append('# Execution courts and testkits moved out of src name the original library test doors.')
        out.extend(additions)
    return '\n'.join(out) + '\n'


def library_test_doors(model):
    names = set()
    for scope_names, scope in model.production.scopes.items():
        names |= {n for n in scope.declared if TEST_DOOR.fullmatch(n)}
        names |= {n for n in scope.members | scope.gated_members if TEST_DOOR.fullmatch(n)}
    return names


def under(child, owner):
    return child == owner or child.startswith(owner + '/')


def splits_toml(text, courts, owners_of):
    """Append one [[splits]] per court whose frontier another court shares."""
    groups = {}
    for name, row in courts.items():
        groups.setdefault(tuple(row['frontier']), []).append(name)
    blocks = []
    for frontier, names in sorted(groups.items()):
        if len(names) < 2:
            continue
        base = min(names, key=len)
        for name in sorted(names):
            owners = owners_of[name]
            delegates = sorted({c[len(o) + 1:] for o in owners for peer in names if peer != name
                                for c in owners_of[peer] if c != o and under(c, o)})
            area = name[len(base) + 1:] if name != base else area_of_base(owners)
            blocks.append('\n'.join([
                '[[splits]]', f'court = "{name}"',
                'frontier = [' + ', '.join(f'"{f}"' for f in frontier) + ']',
                f'area = "{area}"',
                'owners = [' + ', '.join(f'"{o}"' for o in owners) + ']',
                'delegates = [' + ', '.join(f'"{d}"' for d in delegates) + ']']))
    return text.rstrip('\n') + '\n\n' + '\n\n'.join(blocks) + '\n' if blocks else text


def area_of_base(owners):
    lead = min(owners).split('/')
    words = [w for w in lead if w not in ('domain_computation', 'tests', 'process', 'cost', 'serial')]
    return (words[0] if words else 'entry').replace('_', '-')
