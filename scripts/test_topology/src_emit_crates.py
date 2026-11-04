"""Generated court and testkit crates: manifests, roots and one `mod.rs` per module.

Every module of a generated crate is `<root dir>/<module path>/mod.rs`, and every child is
declared with an explicit `#[path]`, so file layout can never change module resolution.
A module that holds no moved part is a scaffold that only declares its children.
"""
from collections import defaultdict
from pathlib import PurePosixPath
import posixpath
import re
import tomllib

from cfg_context import rebind
from rust_source import crate_uses

CRATES = 'workspaces/worth-query/crates'
LIBRARY_DIR = CRATES + '/worth-query-execution'


def snake(name):
    return name.replace('-', '_')


def module_files(crate, kind):
    """(root file, module directory) inside the crate directory."""
    base = CRATES + '/' + crate
    if kind == 'testkit':
        return base + '/src/lib.rs', base + '/src'
    return base + '/tests/' + snake(crate) + '.rs', base + '/tests/' + snake(crate)


def module_tree(paths):
    """Every module path with its ancestors, mapped to its sorted child names."""
    children = defaultdict(set)
    for path in paths:
        for depth in range(1, len(path) + 1):
            children[path[:depth - 1]].add(path[depth - 1])
            children.setdefault(path[:depth], set())
    return {k: sorted(v) for k, v in children.items()}


def declaration(child, public, attributes=''):
    vis = 'pub ' if public else ''
    return f'{attributes}#[path = "{child}/mod.rs"]\n{vis}mod {child};\n'


def relative_to(target, directory):
    """POSIX relative path from a directory to a repository-relative target."""
    return posixpath.relpath(target, directory)


def rebase_literals(text, source_file, destination_file, moved=None):
    """Relative `include!`/`include_str!`/`include_bytes!` literals keep naming the same file.

    `moved` maps literals whose included file moves next to this file to their new name.
    """
    moved = moved or {}

    def replace(match):
        literal = match.group(2)
        if literal in moved:
            return match.group(1) + moved[literal] + match.group(3)
        if literal.startswith('/') or '://' in literal:
            return match.group(0)
        target = posixpath.normpath(posixpath.join(posixpath.dirname(source_file), literal))
        return match.group(1) + relative_to(target, posixpath.dirname(destination_file)) + match.group(3)
    return re.sub(r'(\binclude(?:_str|_bytes)?!\s*\(\s*")([^"]+)("\s*\))', replace, text)


def crate_sources(crate, kind, parts, root_attributes):
    """{relative file path: content} for one generated crate.

    A foreign part (a source another package owns and compiles) is mounted where it lies,
    with a recomputed `#[path]`; its own module declarations keep governing its children.
    """
    root_file, directory = module_files(crate, kind)
    foreign = {part['path'] for part in parts if part.get('foreign_file')}
    kept = [part for part in parts if not any(part['path'][:len(f)] == f and part['path'] != f for f in foreign)]
    by_path = {part['path']: part for part in kept}
    tree = module_tree(by_path)
    # Court targets are leaves and testkits unpublished: their module tree is open to itself.
    public = True
    files = {}
    for path, children in sorted(tree.items()):
        if path in foreign:
            continue
        own_file = root_file if not path else directory + '/' + '/'.join(path) + '/mod.rs'
        text = ''
        if path in by_path:
            part = by_path[path]
            moved = {literal: posixpath.basename(literal) for literal, _target, _text in part.get('includes', [])}
            text = rebase_literals(part['text'], part['source_file'], own_file, moved).rstrip() + '\n'
            for literal, _target, included in part.get('includes', []):
                files[posixpath.dirname(own_file) + '/' + posixpath.basename(literal)] = \
                    rebase_literals(included, part['source_file'], own_file).rstrip() + '\n'
            if kind == 'testkit':
                text, _receipts = rebind(text, (), helper=True)
        mounts = ''
        for child in children:
            child_part = by_path.get(path + (child,), {})
            attributes = child_part.get('attributes', '')
            vis = 'pub ' if public else ''
            if child_part.get('foreign_file'):
                target = relative_to(child_part['foreign_file'], posixpath.dirname(own_file))
            elif not path:
                target = ('' if kind == 'testkit' else PurePosixPath(directory).name + '/') + child + '/mod.rs'
            else:
                target = child + '/mod.rs'
            mounts += f'{attributes}#[path = "{target}"]\n{vis}mod {child};\n'
        if not path:
            header = ('//! Canonical fixtures for the installed Query execution courts.\n' if kind == 'testkit'
                      else '//! Query execution proofs grouped by their product frontier.\n')
            files[root_file] = header + root_attributes + mounts
            continue
        files[own_file] = (text + ('\n' if text and mounts else '') + mounts) or '\n'
    return files


def library_dependencies(repository):
    manifest = tomllib.loads((repository / LIBRARY_DIR / 'Cargo.toml').read_text(encoding='utf-8'))
    table = dict(manifest.get('dependencies', {}))
    table.update(manifest.get('dev-dependencies', {}))
    return table


def toml_inline(value):
    if isinstance(value, bool):
        return 'true' if value else 'false'
    if isinstance(value, str):
        return '"' + value + '"'
    if isinstance(value, list):
        return '[' + ', '.join(toml_inline(v) for v in value) + ']'
    pairs = ', '.join(f'{k} = {toml_inline(v)}' for k, v in value.items())
    return '{ ' + pairs + ' }'


def manifest(crate, kind, texts, library_table, testkits_used, products):
    aliases = {snake(name): name for name in library_table} | {snake(p): p for p in products}
    used = set()
    for text in texts:
        used |= crate_uses(text, set(aliases))
    rows = {'worth-query-execution': {'path': '../worth-query-execution'}}
    for alias in sorted(used):
        name = aliases[alias]
        if name == 'worth-query-execution':
            continue
        rows[name] = library_table.get(name, {'path': '../' + name})
    for kit in sorted(testkits_used):
        if kit != crate:
            rows[kit] = {'path': '../' + kit}
    lines = ['[package]', f'name = "{crate}"', 'autotests = false', 'edition.workspace = true',
             'rust-version.workspace = true', 'version.workspace = true', 'license.workspace = true',
             'publish = false', '']
    if kind == 'testkit':
        lines += ['[lib]', 'test = false', 'doctest = false', '', '[dependencies]']
    else:
        lines += ['[dev-dependencies]']
    lines += [f'{name} = {toml_inline(spec)}' for name, spec in sorted(rows.items())]
    if kind != 'testkit':
        lines += ['', '[[test]]', f'name = "{snake(crate)}"', f'path = "tests/{snake(crate)}.rs"']
    lines += ['', '[lints]', 'workspace = true', '']
    return '\n'.join(lines)


def referenced_testkits(texts, testkits):
    found = set()
    for text in texts:
        for kit in testkits:
            if re.search(r'\b' + snake(kit) + r'\b', text):
                found.add(kit)
    return found
