#!/usr/bin/env python3
"""Sealed test doors between worth-query-execution and its moved courts and testkits.

Usage:
    python scripts/test_topology/src_court_doors.py items [--root .] [--report DIR]
    python scripts/test_topology/src_court_doors.py members CHECK.json [--root .] [--report DIR]

`items` re-exports every library item the courts and testkits name, once, from
`worth_query_execution::court_doors_for_test::<path>` (src_door_items), makes it `pub` with
its modules `pub(crate)`, and rewrites the references. `members` reads `cargo check`
diagnostics (E0624 private methods, E0616 private fields, E0451/E0423 struct literals) and
writes `<name>_for_test` inherent doors in a `court_doors_for_test.rs` sibling of each owning
module, then rewrites the call sites at the diagnostic spans (src_door_members). Both steps
are deterministic and idempotent: a second run on the result finds nothing to do. Each writes
a sorted JSON report and a Markdown summary (with the authority-bearing doors listed apart).
"""
import argparse
import json
from pathlib import Path

from module_tree import read_tree
from rust_source import replace_spans
from src_door_items import DOOR, door_files, plan_items, visibility_edits
from src_surface import Surface

LIB = 'workspaces/worth-query/crates/worth-query-execution/src/lib.rs'
CRATES = 'workspaces/worth-query/crates'
AUTHORITY = ('Authority', 'authority', 'mint')


def moved_files(root):
    crates = root / CRATES
    return sorted(f for d in sorted(crates.iterdir())
                  if d.name.startswith(('worth-query-court-', 'worth-query-testkit-')) and 'execution' in d.name
                  for f in d.rglob('*.rs'))


def rewritten(text, rewrites):
    changes = []
    for reference, module, name, target in rewrites:
        head = 'worth_query_execution'
        if reference.through_door:
            continue
        if reference.glob:
            line_start = text.rfind('\n', 0, reference.start) + 1
            line_end = text.index(';', reference.end) + 1
            prefix = text[line_start:reference.start]
            lines = [f'{prefix}{head}::{DOOR}::' + '::'.join(module + (n,)) + ';' for n in target]
            changes.append((line_start, line_end, '\n'.join(lines)))
        elif target is not None and target[0] in ('public', 'external'):
            old = head + '::' + '::'.join(module + (name,))
            new = head + '::' + '::'.join(target[1]) if target[0] == 'public' else target[1]
            if text.startswith(old, reference.start):
                changes.append((reference.start, reference.start + len(old), new))
        elif target is not None or name is None:
            at = reference.start + len(head)
            changes.append((at, at, '::' + DOOR))
    return replace_spans(text, sorted(changes))


def items(root, report):
    tree = read_tree(root / LIB)
    index = {node.names: node for node in tree.walk()}
    surface = Surface(tree, index, [])
    plan = plan_items(moved_files(root), index, surface)
    for path, rewrites in plan.rewrites.items():
        text = path.read_text(encoding='utf-8')
        new = rewritten(text, rewrites)
        if new != text:
            path.write_text(new, encoding='utf-8', newline='\n')
    for source, changes in visibility_edits(plan.doors, index).items():
        text = Path(source).read_text(encoding='utf-8')
        Path(source).write_text(replace_spans(text, changes), encoding='utf-8', newline='\n')
    library = (root / LIB).parent
    for rel, text in door_files(plan.doors, DOOR).items():
        target = library / rel
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(text, encoding='utf-8', newline='\n')
    lib = (root / LIB).read_text(encoding='utf-8')
    declaration = f'#[doc(hidden)]\n#[path = "{DOOR}/mod.rs"]\npub mod {DOOR};\n'
    if declaration not in lib:
        (root / LIB).write_text(lib.rstrip('\n') + '\n' + declaration, encoding='utf-8', newline='\n')
    doors = sorted('::'.join(m + (n,)) + ' -> ' + '::'.join(o[0] + (o[1],))
                   for m, entries in plan.doors.items() for n, o in entries.items())
    write_report(report, 'items', {
        'item_doors': doors,
        'authority_doors': [d for d in doors if any(w in d.split(' -> ')[1] for w in AUTHORITY)],
        'unresolved': sorted('::'.join(m + (n,)) for m, n in set(plan.unresolved)),
    })
    return len(doors)


def write_report(report, name, data):
    if report is None:
        return
    report.mkdir(parents=True, exist_ok=True)
    (report / f'src-court-doors-{name}.json').write_text(json.dumps(data, indent=1, sort_keys=True) + '\n',
                                                         encoding='utf-8', newline='\n')
    lines = [f'# Court doors: {name}', '']
    for key, values in sorted(data.items()):
        lines += [f'## {key} ({len(values)})', ''] + [f'- `{v}`' for v in values] + ['']
    (report / f'src-court-doors-{name}.md').write_text('\n'.join(lines), encoding='utf-8', newline='\n')


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('step', choices=('items', 'members'))
    parser.add_argument('check', nargs='?', type=Path)
    parser.add_argument('--root', type=Path, default=Path.cwd())
    parser.add_argument('--report', type=Path)
    args = parser.parse_args(argv)
    root = args.root.resolve()
    if args.step == 'items':
        print(f'{items(root, args.report)} item doors')
    else:
        from src_door_members import members
        print(json.dumps(members(root, args.check, args.report, write_report), sort_keys=True))
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
