"""Inventory of worth-query-execution's sealed test doors, read from the door files themselves.

Item doors are the `pub use` lines under `src/court_doors_for_test/`. Member doors are the
`pub fn *_for_test` doors in every `court_doors_for_test.rs` / `<stem>_court_doors_for_test.rs`
child, classified by body: plain delegation (method), `&self.f` (field), `&mut self.f` (field,
mutable), `self.f` (field, by value), `new_for_test` (constructor); anything else is a hand
door. A door is listed for authority review when its owning type, name or signature names an
authority, a proof, a permit or a mint, or when it re-exports such an item.
"""
from pathlib import Path
import re

LIBRARY = 'workspaces/worth-query/crates/worth-query-execution/src'
AUTHORITY = re.compile(r'Authority|authority|Proof|proof|Permit|permit|\bmint|Mint')
IMPL_BLOCK = re.compile(r'(?ms)^impl\b(.*?)\{\n(.*?)^\}\n?')
DOOR = re.compile(r'pub fn (\w+_for_test)\b([^{]*)\{\s*(.*?)\s*\}\n', re.S)


def member_files(root):
    library = root / LIBRARY
    return sorted(set(library.rglob('court_doors_for_test.rs')) | set(library.rglob('*_court_doors_for_test.rs')))


def kind(name, body):
    base = name[:-len('_for_test')]
    if name == 'new_for_test':
        return 'constructor'
    if re.match(rf'Self::{re.escape(base)}(::<.*?>)?\(', body, re.S):
        return 'method'
    if body == f'&self.{base}':
        return 'field'
    if base.endswith('_mut') and body == f'&mut self.{base[:-4]}':
        return 'field_mut'
    if base.endswith('_into') and body == f'self.{base[:-5]}':
        return 'field_into'
    return 'hand'


def inventory(root):
    counts, authority, members = {}, [], 0
    for path in member_files(root):
        rel = str(path.relative_to(root / LIBRARY))
        for block in IMPL_BLOCK.finditer(path.read_text(encoding='utf-8')):
            header = re.sub(r'\s+', ' ', block.group(1)).strip()
            for name, signature, body in DOOR.findall(block.group(2)):
                found = kind(name, body)
                counts[found] = counts.get(found, 0) + 1
                members += 1
                if AUTHORITY.search(header + ' ' + name + ' ' + signature):
                    authority.append(f'{rel}: impl {header} :: {name} ({found})')
    items = []
    for path in sorted((root / LIBRARY / 'court_doors_for_test').rglob('mod.rs')):
        for line in path.read_text(encoding='utf-8').splitlines():
            match = re.match(r'\s*pub use ([\w:]+)(?: as \w+)?;', line)
            if match:
                items.append(match.group(1))
    item_authority = [i for i in items if AUTHORITY.search(i.rsplit('::', 1)[-1])]
    return {'item_doors': len(items), 'member_doors': members, 'member_kinds': dict(sorted(counts.items())),
            'door_files': len(member_files(root)), 'authority_member_doors': sorted(authority),
            'authority_item_doors': sorted(item_authority)}


def markdown(data):
    lines = ['# worth-query-execution sealed test doors', '',
             f"- item doors: {data['item_doors']}", f"- member doors: {data['member_doors']} in {data['door_files']} files"]
    lines += [f'  - {k}: {v}' for k, v in data['member_kinds'].items()]
    lines += ['', '## Authority doors for owner review', '',
              f"### Member doors ({len(data['authority_member_doors'])})", '']
    lines += [f'- `{d}`' for d in data['authority_member_doors']]
    lines += ['', f"### Item doors ({len(data['authority_item_doors'])})", '']
    lines += [f'- `{d}`' for d in data['authority_item_doors']]
    return '\n'.join(lines) + '\n'
