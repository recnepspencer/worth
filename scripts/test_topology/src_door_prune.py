"""Remove sealed doors the library no longer compiles: stale after the library changed.

Usage:
    python scripts/test_topology/src_door_prune.py CHECK.json

CHECK.json is `cargo check --message-format=json` output for worth-query-execution. A member
door (an `impl` block in a `*court_doors_for_test.rs` file) holding an error is removed whole;
an item door (`pub use` in `src/court_doors_for_test/**/mod.rs`) naming an item that no longer
resolves is removed, and a door module left without doors goes with its declaration. The door
phase regenerates whatever the courts still need against the library's current shapes.
"""
import json
from pathlib import Path
import re
import sys

LIBRARY = 'workspaces/worth-query/crates/worth-query-execution/src/'
WORKSPACE = 'workspaces/worth-query/'
IMPL_BLOCK = re.compile(r'(?ms)^impl\b.*?^\}\n?')


def error_spans(check):
    for line in Path(check).read_text(encoding='utf-8').splitlines():
        try:
            message = json.loads(line)
        except ValueError:
            continue
        if message.get('reason') != 'compiler-message' or message['message']['level'] != 'error':
            continue
        for span in message['message']['spans']:
            if span['is_primary']:
                yield (Path(WORKSPACE) / span['file_name']).resolve(), span['line_start']


def prune(root, check):
    member, item = {}, {}
    for path, line in error_spans(check):
        rel = str(path.relative_to(root))
        if rel.endswith('court_doors_for_test.rs'):
            member.setdefault(path, set()).add(line)
        elif '/src/court_doors_for_test/' in rel:
            item.setdefault(path, set()).add(line)
    removed = 0
    for path, lines in member.items():
        text = path.read_text(encoding='utf-8')
        keep, cursor = [], 0
        for block in IMPL_BLOCK.finditer(text):
            first = text.count('\n', 0, block.start()) + 1
            last = text.count('\n', 0, block.end())
            keep.append(text[cursor:block.start()])
            if any(first <= l <= last for l in lines):
                removed += 1
            else:
                keep.append(block.group(0))
            cursor = block.end()
        keep.append(text[cursor:])
        path.write_text(re.sub(r'\n{3,}', '\n\n', ''.join(keep)), encoding='utf-8')
    for path, lines in item.items():
        text = path.read_text(encoding='utf-8').splitlines(keepends=True)
        kept = [l for i, l in enumerate(text, 1) if i not in lines or not l.lstrip().startswith('pub use')]
        removed += len(text) - len(kept)
        path.write_text(''.join(kept), encoding='utf-8')
    # Door modules left without doors go, with the `pub mod` that declared them.
    changed = True
    while changed:
        changed = False
        for module in sorted((root / LIBRARY / 'court_doors_for_test').rglob('mod.rs'), reverse=True):
            body = [l for l in module.read_text(encoding='utf-8').splitlines() if l.strip() and not l.startswith('//')]
            if body or module.parent.name == 'court_doors_for_test':
                continue
            parent = module.parent.parent / 'mod.rs'
            name = module.parent.name
            module.unlink()
            module.parent.rmdir() if not any(module.parent.iterdir()) else None
            if parent.exists():
                lines = parent.read_text(encoding='utf-8').splitlines(keepends=True)
                out, skip = [], False
                for i, l in enumerate(lines):
                    if l.strip() == f'#[path = "{name}/mod.rs"]' and i + 1 < len(lines) and lines[i + 1].strip() == f'pub mod {name};':
                        skip = True
                        continue
                    if skip and l.strip() == f'pub mod {name};':
                        skip = False
                        continue
                    out.append(l)
                parent.write_text(''.join(out), encoding='utf-8')
            changed = True
    return removed


def main(argv=None):
    args = argv or sys.argv[1:]
    print(f'{prune(Path(".").resolve(), args[0])} stale doors removed')


if __name__ == '__main__':
    main()
