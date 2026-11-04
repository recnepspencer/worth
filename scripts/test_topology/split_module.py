"""Move named top-level items of a Rust module file, verbatim, into a child module.

Usage:
    python scripts/test_topology/split_module.py FILE CHILD --doc "Responsibility." ITEM...

ITEM is a top-level item's name, or `NAME#k` for the k-th item of a repeated name (impl blocks).

The child is `<dir of FILE>/CHILD/mod.rs` (FILE must be a `mod.rs`), declared from FILE as
`#[path = "CHILD/mod.rs"] pub mod CHILD;`, and opens with `use super::*;`, which reaches the
parent's items and imports, private ones included. An item keeps its attributes and the comment
lines directly above it. A moved item the parent still names is imported back privately with
`use CHILD::NAME;` (no re-export); a moved private item it names becomes `pub(super)`.
"""
import argparse
from pathlib import Path
import re

from rust_source import tokens
from src_scope import scope_items


def item_spans(text):
    """{name: (start, end, keyword, visibility)} of top-level items, comments above included; a
    repeated name (impl blocks of one trait) is also reachable as `NAME#k`, k from 1."""
    spans, seen = {}, {}
    for _attrs, visibility, keyword, name, start, end in scope_items(text):
        while True:
            line_start = text.rfind('\n', 0, start - 1) + 1 if start else 0
            line = text[line_start:start].strip()
            if start and line.startswith('//') and not line.startswith('//!'):
                start = line_start
            else:
                break
        seen[name] = seen.get(name, 0) + 1
        spans.setdefault(name, (start, end, keyword, visibility))
        spans[f'{name}#{seen[name]}'] = (start, end, keyword, visibility)
    return spans


def split(path, child, doc, names):
    path = Path(path)
    assert path.name == 'mod.rs', 'split a mod.rs module'
    text = path.read_text(encoding='utf-8')
    spans = item_spans(text)
    missing = [n for n in names if n not in spans]
    if missing:
        raise SystemExit(f'no top-level item named {missing} in {path}')
    chosen = sorted((spans[n][0], spans[n][1], n) for n in names)
    moved = []
    rest, cursor = [], 0
    for start, end, name in chosen:
        rest.append(text[cursor:start])
        item = text[start:end]
        keyword, visibility = spans[name][2], spans[name][3]
        moved.append((name, keyword, visibility, item))
        cursor = end
        if text[cursor:cursor + 1] == '\n':
            cursor += 1
    rest.append(text[cursor:])
    parent = re.sub(r'\n{3,}', '\n\n', ''.join(rest))
    remaining = {t.value for t in tokens(parent) if t.kind == 'ident'}
    imports, body = [], []
    for name, keyword, visibility, item in moved:
        name = name.split('#')[0]
        if keyword != 'impl' and name in remaining and keyword in ('fn', 'struct', 'enum', 'type', 'const', 'static', 'trait'):
            if not visibility:
                item = re.sub(rf'(^|\n)((?:\s*#\[[^\n]*\]\n|\s*//[^\n]*\n)*\s*)({keyword}\s)', rf'\1\2pub(super) \3', item, count=1)
            imports.append(f'use {child}::{name};')
        body.append(item)
    child_dir = path.parent / child
    child_dir.mkdir(exist_ok=False)
    (child_dir / 'mod.rs').write_text(f'//! {doc}\nuse super::*;\n\n' + '\n\n'.join(body) + '\n', encoding='utf-8')
    declaration = f'#[path = "{child}/mod.rs"]\npub mod {child};\n' + ''.join(i + '\n' for i in imports)
    path.write_text(parent.rstrip('\n') + '\n' + declaration, encoding='utf-8')
    return len(moved)


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('file')
    parser.add_argument('child')
    parser.add_argument('--doc', required=True)
    parser.add_argument('items', nargs='+')
    args = parser.parse_args(argv)
    print(f'{split(args.file, args.child, args.doc, args.items)} items moved to {args.child}')


if __name__ == '__main__':
    main()
