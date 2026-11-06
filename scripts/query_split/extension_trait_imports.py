#!/usr/bin/env python3
"""Bring extension traits into scope at their call sites, and reshape door calls a move retargeted.

Usage:
    python scripts/query_split/extension_trait_imports.py <spec.json> [--root .] [--dry-run]

`spec.json`:
    {"imports": [{"method": "field_effect", "trait": "crate::...::CommittedFieldEffect",
                  "door": false}],
     "reshapes": [{"under": "<directory>", "pattern": "<regex>", "replacement": "<text>"}]}

An import names a trait by its library path. Every Rust file that calls `.method(` or
`::method(` gains `use <path>;`:
- for a production trait (`door` false), in the execution library, with the path as given;
- for a door trait (`door` true), in the execution courts and testkits, with the path through
  `worth_query_execution::court_doors_for_test::` (src_court_doors.py items then routes it).

Files inside the trait's own module are skipped (it is in scope there), as are files that already
name the trait. The import goes after the file's inner doc comments and attributes. A file whose
call sits in an inline `mod name {` block is reported, not edited. A reshape is a regex
substitution over the Rust files under one directory, for call shapes a moved door changed.
"""
import argparse
import json
from pathlib import Path
import re
import sys

LIBRARY = 'workspaces/worth-query/crates/worth-query-execution/src'
CRATES = 'workspaces/worth-query/crates'
DOORS = 'worth_query_execution::court_doors_for_test::'


def court_files(root):
    crates = root / CRATES
    return sorted(f for d in sorted(crates.iterdir())
                  if d.is_dir() and d.name.startswith(('worth-query-court-', 'worth-query-testkit-'))
                  and 'execution' in d.name
                  for f in d.rglob('*.rs') if 'target' not in f.parts)


def library_files(root):
    return sorted(f for f in (root / LIBRARY).rglob('*.rs') if 'court_doors_for_test' not in f.parts[-2:-1])


def own_module_dir(root, trait_path):
    """The defining module's file and directory, so its own tree is skipped."""
    parts = trait_path.split('::')[1:-1]
    if parts and parts[-1] == 'court_doors_for_test':
        parts = parts[:-1]
    base = root / LIBRARY / Path(*parts)
    return base.with_suffix('.rs'), base


def insertion_point(text):
    point = 0
    for line in text.splitlines(keepends=True):
        if line.startswith(('//!', '#![')):
            point += len(line)
        else:
            break
    return point


def add_imports(root, spec, dry_run):
    edited, reported = [], []
    for entry in spec.get('imports', []):
        call = re.compile(r'(?:\.|::)' + re.escape(entry['method']) + r'\s*(?:::<[^;]*?>)?\s*\(')
        name = entry['trait'].rsplit('::', 1)[1]
        path = DOORS + entry['trait'].split('::', 1)[1] if entry['door'] else entry['trait']
        own_file, own_dir = own_module_dir(root, entry['trait'])
        files = court_files(root) if entry['door'] else library_files(root)
        for f in files:
            if f == own_file or own_dir in f.parents:
                continue
            text = f.read_text(encoding='utf-8')
            match = call.search(text)
            if not match or re.search(r'\buse\b[^;]*\b' + re.escape(name) + r'\b', text):
                continue
            before = text[:match.start()]
            if re.search(r'^\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+\w+\s*\{', before, re.M):
                reported.append(f'{f.relative_to(root)}: {entry["method"]} called inside an inline module')
                continue
            point = insertion_point(text)
            text = text[:point] + f'use {path};\n' + text[point:]
            edited.append(str(f.relative_to(root)))
            if not dry_run:
                f.write_text(text, encoding='utf-8', newline='\n')
    return edited, reported


def reshape(root, spec, dry_run):
    edited = []
    for entry in spec.get('reshapes', []):
        pattern = re.compile(entry['pattern'], re.S)
        for f in sorted((root / entry['under']).rglob('*.rs')):
            text = f.read_text(encoding='utf-8')
            new, count = pattern.subn(entry['replacement'], text)
            if count:
                edited.append(f'{f.relative_to(root)} ({count})')
                if not dry_run:
                    f.write_text(new, encoding='utf-8', newline='\n')
    return edited


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('spec', type=Path)
    parser.add_argument('--root', type=Path, default=Path.cwd())
    parser.add_argument('--dry-run', action='store_true')
    args = parser.parse_args(argv)
    root = args.root.resolve()
    spec = json.loads(args.spec.read_text(encoding='utf-8'))
    imported, reported = add_imports(root, spec, args.dry_run)
    reshaped = reshape(root, spec, args.dry_run)
    print(json.dumps({'imported': imported, 'reshaped': reshaped, 'reported': reported}, indent=1))
    return 1 if reported else 0


if __name__ == '__main__':
    sys.exit(main())
