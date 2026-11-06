"""Rename a court crate, or merge one court into another, everywhere it is named.

Usage:
    python scripts/test_topology/court_rename.py rename OLD NEW [--frontier CRATE...]
    python scripts/test_topology/court_rename.py merge OLD INTO
    python scripts/test_topology/court_rename.py rename OLD NEW --unsplit
    python scripts/test_topology/court_rename.py prune

`--unsplit` removes the court's split registration (a court alone on its frontier is not a
split and takes the plain frontier name). `prune` drops every delegate that no court sharing the
split's frontier owns, as boundary-check judges ownership.

`rename` moves the crate and its test tree with `git mv`, renames the package and its test
target, and replaces the exact old names (kebab and snake case, never as a prefix of a longer
name) in the boundary configs and consumer docs; `--frontier` sets its split registration's
frontier. `merge` moves OLD's test tree into INTO's (module declarations of shared `mod.rs`
files are united; any other shared file stops the merge), unites dev-dependencies and split
owners and delegates, removes OLD, and points every reference at INTO.
"""
import argparse
from pathlib import Path
import re
import subprocess

CRATES = 'workspaces/worth-query/crates'
SPLITS = 'tools/boundary-check/config/court-frontier-splits.toml'
REFERENCES = ('tools/boundary-check/config', 'docs', 'workspaces/worth-query/crates/worth-query/docs')


def snake(name):
    return name.replace('-', '_')


def git(*args):
    subprocess.run(['git', *args], check=True)


def references(root):
    for base in REFERENCES:
        for path in sorted((root / base).rglob('*')):
            if path.is_file() and path.suffix in ('.txt', '.toml', '.json', '.md'):
                yield path


def replace_names(root, old, new):
    """Exact old names (kebab, snake) -> new in every reference file; returns files changed."""
    patterns = [(re.compile(rf'(?<![\w-]){re.escape(old)}(?![\w-])'), new),
                (re.compile(rf'(?<!\w){re.escape(snake(old))}(?!\w)'), snake(new))]
    changed = 0
    for path in references(root):
        text = path.read_text(encoding='utf-8')
        updated = text
        for pattern, replacement in patterns:
            updated = pattern.sub(replacement, updated)
        if updated != text:
            path.write_text(updated, encoding='utf-8', newline='\n')
            changed += 1
    return changed


def split_blocks(text):
    """[(start, end, court)] of the `[[splits]]` blocks."""
    starts = [m.start() for m in re.finditer(r'(?m)^\[\[splits\]\]\n', text)] + [len(text)]
    blocks = []
    for a, b in zip(starts, starts[1:]):
        court = re.search(r'(?m)^court = "([^"]+)"', text[a:b])
        blocks.append((a, b, court.group(1) if court else None))
    return blocks


def list_field(block, key):
    match = re.search(rf'(?m)^{key} = \[(.*)\]$', block)
    return re.findall(r'"([^"]+)"', match.group(1)) if match else []


def set_list(block, key, values):
    rendered = ', '.join(f'"{v}"' for v in values)
    return re.sub(rf'(?m)^{key} = \[.*\]$', f'{key} = [{rendered}]', block)


def rename(root, old, new, frontier):
    crates = root / CRATES
    git('mv', str(crates / old), str(crates / new))
    tests = crates / new / 'tests'
    git('mv', str(tests / snake(old)), str(tests / snake(new)))
    git('mv', str(tests / f'{snake(old)}.rs'), str(tests / f'{snake(new)}.rs'))
    entry = tests / f'{snake(new)}.rs'
    entry.write_text(entry.read_text(encoding='utf-8').replace(f'"{snake(old)}/', f'"{snake(new)}/'), encoding='utf-8')
    manifest = crates / new / 'Cargo.toml'
    text = manifest.read_text(encoding='utf-8')
    text = text.replace(f'name = "{old}"', f'name = "{new}"').replace(f'name = "{snake(old)}"', f'name = "{snake(new)}"')
    text = text.replace(f'path = "tests/{snake(old)}.rs"', f'path = "tests/{snake(new)}.rs"')
    manifest.write_text(text, encoding='utf-8')
    changed = replace_names(root, old, new)
    if frontier:
        splits = root / SPLITS
        text = splits.read_text(encoding='utf-8')
        for a, b, court in reversed(split_blocks(text)):
            if court == new:
                text = text[:a] + set_list(text[a:b], 'frontier', frontier) + text[b:]
        splits.write_text(text, encoding='utf-8')
    return changed


def unsplit(root, court):
    splits = root / SPLITS
    text = splits.read_text(encoding='utf-8')
    for a, b, name in reversed(split_blocks(text)):
        if name == court:
            text = text[:a] + text[b:]
    splits.write_text(text, encoding='utf-8')


def under(child, owner):
    return child == owner or child.startswith(owner + '/')


def prune(root):
    """Drop delegates no same-frontier peer owns; returns the number dropped."""
    splits = root / SPLITS
    text = splits.read_text(encoding='utf-8')
    blocks = split_blocks(text)
    fields = {court: (list_field(text[a:b], 'frontier'), list_field(text[a:b], 'owners')) for a, b, court in blocks}
    dropped = 0
    for a, b, court in reversed(blocks):
        block = text[a:b]
        frontier, owners = fields[court]
        peers = [o for c, (f, o) in fields.items() if c != court and f == frontier]
        kept = [d for d in list_field(block, 'delegates')
                if any(under(child, f'{owner}/{d}') for owner in owners for peer in peers for child in peer)]
        dropped += len(list_field(block, 'delegates')) - len(kept)
        text = text[:a] + set_list(block, 'delegates', kept) + text[b:]
    splits.write_text(text, encoding='utf-8')
    return dropped


def merged_mod(ours, theirs):
    """`ours` with every line of `theirs` that declares a module or its #[path] it lacks."""
    lines = ours.rstrip('\n').split('\n')
    present = set(lines)
    incoming = theirs.rstrip('\n').split('\n')
    for i, line in enumerate(incoming):
        if line in present or not re.match(r'(#\[path = |pub mod |mod |pub\(crate\) mod )', line):
            continue
        if line.startswith('#[path') and i + 1 < len(incoming) and incoming[i + 1] in present:
            continue
        lines.append(line)
        present.add(line)
    return '\n'.join(lines) + '\n'


def merge(root, old, into):
    crates = root / CRATES
    source, target = crates / old / 'tests' / snake(old), crates / into / 'tests' / snake(into)
    for path in sorted(p for p in source.rglob('*') if p.is_file()):
        dest = target / path.relative_to(source)
        if dest.exists():
            if path.name != 'mod.rs':
                raise SystemExit(f'both courts have {dest.relative_to(root)}')
            dest.write_text(merged_mod(dest.read_text(encoding='utf-8'), path.read_text(encoding='utf-8')), encoding='utf-8')
            git('rm', '-q', str(path))
        else:
            dest.parent.mkdir(parents=True, exist_ok=True)
            git('mv', str(path), str(dest))
    entry_old, entry = crates / old / 'tests' / f'{snake(old)}.rs', target.parent / f'{snake(into)}.rs'
    entry.write_text(merged_mod(entry.read_text(encoding='utf-8'),
                                entry_old.read_text(encoding='utf-8').replace(f'"{snake(old)}/', f'"{snake(into)}/')),
                     encoding='utf-8')
    manifest_old, manifest = crates / old / 'Cargo.toml', crates / into / 'Cargo.toml'
    deps = re.findall(r'(?m)^[\w-]+ = \{ path = .*\}$', manifest_old.read_text(encoding='utf-8'))
    text = manifest.read_text(encoding='utf-8')
    head, tail = text.split('[dev-dependencies]\n', 1)
    body, rest = re.match(r'((?:[\w-]+ = \{ path = .*\}\n)*)(.*)', tail, re.S).groups()
    present = set(body.splitlines())
    body = ''.join(line + '\n' for line in sorted(present | set(deps), key=str.lower))
    manifest.write_text(head + '[dev-dependencies]\n' + body + rest, encoding='utf-8')
    git('rm', '-q', '-r', str(crates / old))
    splits = root / SPLITS
    text = splits.read_text(encoding='utf-8')
    blocks = {court: (a, b) for a, b, court in split_blocks(text)}
    old_block = text[slice(*blocks[old])]
    a, b = blocks[into]
    block = text[a:b]
    for key in ('owners', 'delegates'):
        block = set_list(block, key, sorted(set(list_field(block, key)) | set(list_field(old_block, key))))
    text = text[:a] + block + text[b:]
    oa, ob = split_blocks(text)[[c for _, _, c in split_blocks(text)].index(old)][:2]
    text = text[:oa] + text[ob:]
    splits.write_text(text, encoding='utf-8')
    return replace_names(root, old, into)


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('step', choices=('rename', 'merge', 'prune'))
    parser.add_argument('old', nargs='?')
    parser.add_argument('new', nargs='?')
    parser.add_argument('--frontier', nargs='*', default=[])
    parser.add_argument('--unsplit', action='store_true')
    parser.add_argument('--root', type=Path, default=Path('.'))
    args = parser.parse_args(argv)
    root = args.root.resolve()
    if args.step == 'prune':
        print(f'{prune(root)} delegates dropped')
    elif args.step == 'rename':
        print(f'{rename(root, args.old, args.new, args.frontier)} reference files updated')
        if args.unsplit:
            unsplit(root, args.new)
    else:
        print(f'{merge(root, args.old, args.new)} reference files updated')


if __name__ == '__main__':
    main()
