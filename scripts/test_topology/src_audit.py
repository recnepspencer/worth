#!/usr/bin/env python3
"""Classify each in-source test family by the private access it needs (Python 3.11+).

Usage: python scripts/test_topology/src_audit.py <crate>/src/lib.rs --out <dir>
"""
from collections import Counter
import json
from pathlib import Path, PurePosixPath

from rust_source import tokens
from src_families import families
from src_paths import Reference, module_index, references
from src_surface import Surface

DECISIONS = {
    'public-surface': 'move as is',
    'runtime-root-public': 'must move; public surface already suffices',
    'runtime-root-private': 'must move; blocked on an owner API or testkit ruling for its private needs',
    'private-logic': 'stays a unit test of private logic',
    'admitted': 'owner-ruled private-logic admission; stays',
    'support': 'test support with no tests; follows its readers',
}


def relative(path, root):
    return PurePosixPath(Path(path).resolve().relative_to(root.resolve()).as_posix()).as_posix()


def audit(lib_source, repository, baseline_path, audit_path):
    repository = Path(repository).resolve()
    tree, found = families(Path(lib_source))
    index = module_index(tree)
    roots = [f.names for f in found]
    surface = Surface(tree, index, roots)
    baseline = json.loads(Path(baseline_path).read_text(encoding='utf-8'))
    admissions = json.loads(Path(audit_path).read_text(encoding='utf-8'))['audits']
    owner_of = {}
    for family in found:
        for node in family.node.walk():
            owner_of[node.names] = family.names
    rows = []
    for family in found:
        rows.append(family_row(family, repository, index, surface, owner_of, baseline, admissions))
    rows.sort(key=lambda r: r['module'])
    return report(rows, repository, lib_source)


def family_row(family, repository, index, surface, owner_of, baseline, admissions):
    files = [relative(f, repository) for f in family.files]
    declaring = relative(family.node.source if family.node.declaration.body is not None
                         else index[family.names[:-1]].source, repository)
    sources = set(files) | ({declaring} if family.node.declaration.body is not None else set())
    roots = sorted({r for row in baseline if row['source'] in sources and in_family_row(row, family, files)
                    for r in row['roots']})
    names = {name for name, _digest in family.tests}
    admitted = sorted({a['test'] for a in admissions
                       if a['source'] in sources and a['test'].rsplit('::', 1)[-1] in names})
    needs, support_edges = Counter(), set()
    identifiers = set()
    for node in family.node.walk():
        own = node.own_text()
        identifiers |= member_calls(own)
        for reference in references(node, index):
            for target in expand(reference, identifiers_of(own), surface):
                owner = owner_of.get(target.module)
                if owner == family.names:
                    continue
                if owner is not None:
                    support_edges.add('::'.join(owner))
                    continue
                kind = surface.classify(target.module, target.item)
                needs[(kind, '::'.join(target.module + ((target.item,) if target.item else ())))] += 1
    members = sorted(identifiers & surface.gated_members)
    private = sorted({path for (kind, path) in needs if kind == 'private'})
    hooks = sorted({path for (kind, path) in needs if kind == 'hook'} | {'.' + m for m in members})
    public = sorted({path for (kind, path) in needs if kind == 'public'})
    unresolved = sorted({path for (kind, path) in needs if kind == 'unresolved'})
    decision = decide(family, roots, private, hooks, admitted)
    return {'module': '::'.join(family.names), 'declared_in': declaring, 'files': files,
            'lines': family.lines, 'tests': len(family.tests),
            'test_identities': [list(t) for t in family.tests],
            'macro_tests': family.macro_tests, 'runtime_roots': roots, 'admitted_tests': admitted,
            'private_needs': private, 'test_hooks': hooks, 'unresolved': unresolved, 'public_uses': len(public),
            'support_from': sorted(support_edges), 'decision': decision}


def in_family_row(row, family, files):
    if row['source'] in files:
        return True
    return row['module'].split('::')[-1] == family.names[-1]


def member_calls(text):
    stream = tokens(text)
    return {t.value for i, t in enumerate(stream) if t.kind == 'ident' and i and stream[i - 1].value in ('.', ':')}


def identifiers_of(text):
    return {t.value for t in tokens(text) if t.kind == 'ident'}


def expand(reference, identifiers, surface):
    """A glob import stands for the declared names of its module that the reader mentions."""
    if not reference.glob:
        return [reference]
    scope = surface.scopes.get(reference.module)
    if scope is None:
        return [Reference(reference.module, None)]
    return [Reference(reference.module, name) for name in sorted(scope.names & identifiers)]


def decide(family, roots, private, hooks, admitted):
    if not family.tests and not family.macro_tests:
        return 'support'
    if admitted and len(admitted) >= len(family.tests):
        return 'admitted'
    blocked = private or hooks
    if roots:
        return 'runtime-root-private' if blocked else 'runtime-root-public'
    return 'private-logic' if blocked else 'public-surface'


def report(rows, repository, lib_source):
    totals = Counter()
    for row in rows:
        totals[row['decision'] + ':families'] += 1
        totals[row['decision'] + ':lines'] += row['lines']
        totals[row['decision'] + ':tests'] += row['tests']
    return {'version': 1, 'library': relative(lib_source, repository),
            'decisions': DECISIONS, 'families': rows, 'totals': dict(sorted(totals.items()))}


def markdown(plan):
    lines = ['# In-source test family audit', '', 'Library: `' + plan['library'] + '`', '',
             '| Decision | Families | Tests | Lines |', '|---|---:|---:|---:|']
    for decision in DECISIONS:
        t = plan['totals']
        lines.append(f"| {decision} | {t.get(decision + ':families', 0)} | {t.get(decision + ':tests', 0)} | {t.get(decision + ':lines', 0)} |")
    lines += ['', '| Family | Decision | Tests | Lines | Roots | Private | Hooks |', '|---|---|---:|---:|---:|---:|---:|']
    for row in sorted(plan['families'], key=lambda r: (-r['lines'], r['module'])):
        lines.append(f"| `{row['module']}` | {row['decision']} | {row['tests']} | {row['lines']} | "
                     f"{len(row['runtime_roots'])} | {len(row['private_needs'])} | {len(row['test_hooks'])} |")
    lines += unlock_section(plan)
    return '\n'.join(lines) + '\n'


def unlock_section(plan, limit=40):
    """Blocked courts by what blocks them, and the needs that block the most lines."""
    blocked = [r for r in plan['families'] if r['decision'] == 'runtime-root-private']
    shapes = Counter()
    weight, families_needing = Counter(), Counter()
    for row in blocked:
        shape = ('hooks' if row['test_hooks'] else '') + ('+' if row['test_hooks'] and row['private_needs'] else '') + \
                ('private items' if row['private_needs'] else '')
        shapes[shape] += row['lines']
        for need in row['test_hooks']:
            weight['hook ' + need] += row['lines']
            families_needing['hook ' + need] += 1
        for need in row['private_needs']:
            weight['private ' + need] += row['lines']
            families_needing['private ' + need] += 1
    lines = ['', '## Blocked courts', '', '| Blocked by | Lines |', '|---|---:|']
    lines += [f'| {shape} | {count} |' for shape, count in sorted(shapes.items())]
    lines += ['', f'## Needs blocking the most court lines (top {limit})', '',
              '| Need | Families | Lines blocked |', '|---|---:|---:|']
    for need, count in sorted(weight.items(), key=lambda kv: (-kv[1], kv[0]))[:limit]:
        lines.append(f'| `{need}` | {families_needing[need]} | {count} |')
    return lines


def main(argv=None):
    import argparse
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('lib', type=Path, help='library root source, e.g. <crate>/src/lib.rs')
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--root', type=Path, default=Path.cwd())
    parser.add_argument('--baseline', type=Path, default=Path('tools/boundary-check/config/src-court-baseline.json'))
    parser.add_argument('--admissions', type=Path,
                        default=Path('tools/boundary-check/config/src-court-private-logic-audit.json'))
    args = parser.parse_args(argv)
    root = args.root.resolve()
    plan = audit(root / args.lib, root, root / args.baseline, root / args.admissions)
    args.out.mkdir(parents=True, exist_ok=True)
    (args.out / 'src-family-audit.json').write_text(json.dumps(plan, indent=1, sort_keys=True) + '\n',
                                                     encoding='utf-8', newline='\n')
    (args.out / 'src-family-audit.md').write_text(markdown(plan), encoding='utf-8', newline='\n')
    print(json.dumps(plan['totals'], sort_keys=True))
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
