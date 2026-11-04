"""Repeated imports the item doors made identical (E0252).

Rewriting a court's two paths to one item (a library re-export and the item's own crate path)
can leave the same `use` line twice in one file. A file-scope (column 0) single-line `use`
that exactly repeats an earlier one in the same file is dropped. A glob import (`::*`, which
may repeat) and an attributed `use` (the line before it is `#[..]`) are left alone; nothing
else changes.
"""
from pathlib import Path
import re

CRATES = 'workspaces/worth-query/crates'
USE = re.compile(r'(?:pub(?:\([^)]*\))? )?use [^;{}]+;$')


def court_files(root):
    crates = root / CRATES
    for directory in sorted(crates.iterdir()):
        if directory.name.startswith(('worth-query-court-', 'worth-query-testkit-')) and 'execution' in directory.name:
            yield from sorted(directory.rglob('*.rs'))


def without_repeats(text):
    seen, out = set(), []
    for line in text.splitlines(keepends=True):
        bare = line.rstrip('\n')
        attributed = bool(out) and out[-1].startswith('#[')
        if USE.fullmatch(bare) and not attributed and not bare.endswith('::*;'):
            if bare in seen:
                continue
            seen.add(bare)
        out.append(line)
    return ''.join(out)


def dedupe_uses(root):
    changed = 0
    for path in court_files(root):
        text = path.read_text(encoding='utf-8')
        new = without_repeats(text)
        if new != text:
            path.write_text(new, encoding='utf-8', newline='\n')
            changed += 1
    return changed
