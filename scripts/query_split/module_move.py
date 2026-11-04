#!/usr/bin/env python3
"""Move one module subtree to a new parent, whole: every type in it keeps all its impls.

Usage:
    python scripts/query_split/module_move.py <lib.rs> <old::module::path> <new::parent::path> [--name NEW]
        [--report DIR] [--dry-run]

The files move with `git mv`, keeping their relative layout (non-Rust files under the subtree's
directory, such as `include_str!` documents, move with them). Then:
- the old parent loses its `mod` declaration and the new parent gains one;
- every path in the library, courts and testkits that leads into the moved subtree (directly,
  through an imported module, or through a re-export) is rewritten to the new path; inside the
  subtree, relative paths that stay inside it are kept and `super::` paths that leave it are
  written from `crate::`;
- re-exports of moved items inside the old owner's top-level module are removed once their
  users are rewritten (no shim is left at an old path); re-exports elsewhere (the parent root,
  the facade) point at the new path;
- each visibility restriction in the moved files is restated for the new location; one that
  no longer covers its old users is widened to the nearest common ancestor and recorded.
`src_court_doors.py items` then regenerates the item doors for the rewritten court paths.
"""
import argparse
import json
import os
from pathlib import Path
import subprocess
import sys

sys.path.insert(0, str(Path(__file__).resolve().parent))

from module_graph import LibraryModel  # noqa: E402
from path_resolution import Paths  # noqa: E402
from module_tree import source_files  # noqa: E402
from rust_source import pairs, replace_spans, tokens  # noqa: E402
from src_occurrences import scan, visibility_markers  # noqa: E402
from src_paths import resolve  # noqa: E402
from src_scope import scope_items  # noqa: E402
from vocabulary_visibility import scope_of, stated  # noqa: E402

LIBRARY = 'worth_query_execution'
DOOR = 'court_doors_for_test'


def inside(module, root):
    return module[:len(root)] == root


def common(a, b):
    k = 0
    while k < min(len(a), len(b)) and a[k] == b[k]:
        k += 1
    return a[:k]


def removal_end(text, end):
    while end < len(text) and text[end] in ' \t':
        end += 1
    if end < len(text) and text[end] == '\n':
        end += 1
    return end


def item_start(text, start):
    """Extend a span start over doc comments directly above it and its own indentation."""
    line_start = text.rfind('\n', 0, start) + 1
    if not text[line_start:start].strip():
        start = line_start
    while start:
        previous = text.rfind('\n', 0, start - 1) + 1
        line = text[previous:start].strip()
        if line.startswith('//') and not line.startswith('//!'):
            start = previous
        else:
            break
    return start


def use_text(visibility, leaves):
    rendered = []
    for segments, glob, alias in leaves:
        rendered.append('::'.join(segments) + ('::*' if glob else '') + (f' as {alias}' if alias else ''))
    head = (visibility + ' ' if visibility else '') + 'use '
    return head + (rendered[0] if len(rendered) == 1 else '{' + ', '.join(rendered) + '}') + ';'


class ModuleMove:
    def __init__(self, model, old, new_parent, new_name):
        self.model, self.paths = model, Paths(model)
        self.old, self.new = old, new_parent + (new_name,)
        self.new_parent = new_parent
        self.owner = old[:3] if old[:2] == ('domain_computation', 'primary_graph') else old[:2]
        self.edits = {}
        self.widenings, self.removed_reexports = [], []
        self.dropped = {}       # module -> {name: new segments} of the re-exports removed there
        self.private = []       # (node, path, name): a removed re-export its module still uses
        self.users = set()      # modules outside the subtree whose paths now name it

    def edit(self, path, start, end, text):
        self.edits.setdefault(Path(path), []).append((start, end, text))

    # files
    def file_map(self):
        node = self.model.index[self.old]
        if node.declaration is None or node.declaration.body:
            raise SystemExit('only a file module moves: ' + '::'.join(self.old))
        root_file = node.source
        old_dir = root_file.parent if root_file.name == 'mod.rs' else root_file.with_suffix('')
        parent = self.model.index[self.new_parent]
        base = parent.source.parent if parent.source.name == 'mod.rs' or not self.new_parent \
            else parent.source.with_suffix('')
        name = self.new[-1]
        new_dir = base / name
        mapping = {root_file: new_dir / 'mod.rs' if root_file.name == 'mod.rs' else base / (name + '.rs')}
        for path in sorted(source_files(node)):
            if path == root_file:
                continue
            if old_dir not in path.parents:
                raise SystemExit(f'{path} lies outside {old_dir}; move it by hand first')
            mapping[path] = new_dir / path.relative_to(old_dir)
        if old_dir.is_dir():
            # Files the subtree's sources read (`include_str!` documents) travel with it.
            for path in sorted(old_dir.rglob('*')):
                if path.is_file() and path.suffix != '.rs':
                    mapping[path] = new_dir / path.relative_to(old_dir)
        if any(target.exists() for target in mapping.values()):
            raise SystemExit('a destination file already exists')
        return mapping

    # declarations
    def declarations(self):
        old_parent = self.model.index[self.old[:-1]]
        declaration = self.model.index[self.old].declaration
        start = old_parent.offset + item_start(old_parent.text, declaration.start)
        end = old_parent.offset + removal_end(old_parent.text, declaration.end)
        text = old_parent.source.read_text(encoding='utf-8')
        visibility = ''
        stream = tokens(text[start:end])
        for k, token in enumerate(stream):
            if token.value == 'pub':
                close = pairs(stream).get(k + 1, k) if k + 1 < len(stream) and stream[k + 1].value == '(' else k
                visibility = text[start + token.start:start + stream[close].end]
                break
            if token.value == 'mod':
                break
        self.edit(old_parent.source, start, end, '')
        scope = scope_of(visibility, self.old[:-1])
        for user in self.users:
            if scope != 'pub' and not inside(user, scope):
                scope = common(scope, user)
        if scope != 'pub' and not inside(self.new_parent, scope):
            widened = stated(common(scope, self.new_parent))
            self.widenings.append({'item': 'mod ' + '::'.join(self.new), 'from': visibility or 'private',
                                   'to': widened})
            visibility = widened
        else:
            visibility = stated(scope) if visibility else 'pub(crate)'
        parent = self.model.index[self.new_parent]
        ptext = parent.text
        point = 0
        for _a, _v, keyword, _n, s, e in scope_items(ptext):
            if keyword == 'mod':
                point = removal_end(ptext, e)
        self.edit(parent.source, parent.offset + point, parent.offset + point,
                  f'{visibility} mod {self.new[-1]};\n')

    # paths
    def rewritten(self, node, segments):
        """New segments for a path written in `node`, or None to keep it."""
        found = self.paths.resolve(node, segments)
        if found is None:
            return None
        module, rest, rooted = found
        here_inside = inside(node.names, self.old)
        if inside(module, self.old):
            if here_inside and rooted and segments[0] != 'crate':
                return None          # relative inside the moved subtree: still right
            self.users.add(node.names)
            return ('crate',) + self.new + module[len(self.old):] + tuple(rest)
        if here_inside and rooted and segments[0] == 'super':
            return ('crate',) + module + tuple(rest)
        if rest and module in self.model.surface.scopes and not here_inside:
            origin = self.model.surface.origin(module, rest[0])[0]
            if origin and origin[0] != '<external>' and origin[1] and inside(origin[0], self.old) \
                    and not inside(module, self.old):
                self.users.add(node.names)
                return self.exported(origin) + tuple(rest[1:])
        return None

    def exported(self, origin):
        """The new path of a moved item through the shallowest module of the subtree that names
        it (its own re-export), so a private inner module is never spelled out."""
        for depth in range(len(self.old), len(origin[0]) + 1):
            module = origin[0][:depth]
            if module in self.model.surface.scopes and \
                    self.model.surface.origin(module, origin[1])[0] == origin:
                return ('crate',) + self.new + module[len(self.old):] + (origin[1],)
        return ('crate',) + self.new + origin[0][len(self.old):] + (origin[1],)

    def reexport_into_subtree(self, node, use, segments):
        """A re-export inside the old owner, pointing into the subtree: removed after the move."""
        if not use.visibility or inside(node.names, self.old) or not inside(node.names, self.owner):
            return False
        found = self.paths.resolve(node, segments)
        if found is None:
            return False
        module, rest, _rooted = found
        if inside(module, self.old):
            return True
        if rest and module in self.model.surface.scopes:
            origin = self.model.surface.origin(module, rest[0])[0]
            return bool(origin) and origin[0] != '<external>' and inside(origin[0], self.old)
        return False

    def own_words(self, node, use):
        """Identifiers of the node's own text outside this `use` item and its inline children."""
        text = node.text
        spans = [(c.declaration.start, c.declaration.end) for c in node.children if c.declaration.body]
        spans.append((use.start, use.end))
        return {t.value for t in tokens(text) if t.kind == 'ident'
                and not any(a <= t.start < b for a, b in spans)}

    def private_imports(self):
        for node, path, name in self.private:
            line = use_text('', [(path, False, None if path[-1] == name else name)]) + '\n'
            point = 0
            for _a, _v, keyword, _n, s, e in scope_items(node.text):
                if keyword == 'use':
                    point = removal_end(node.text, e)
            self.edit(node.source, node.offset + point, node.offset + point, line)

    def new_path(self, node, segments):
        found = self.paths.resolve(node, segments)
        module, rest, _rooted = found
        if rest and module in self.model.surface.scopes:
            origin = self.model.surface.origin(module, rest[0])[0]
            if origin and origin[0] != '<external>' and origin[1]:
                return self.exported(origin) + tuple(rest[1:])
        return ('crate',) + self.new + module[len(self.old):] + tuple(rest)

    def globs(self):
        """A glob of a module that lost re-exports gains explicit imports of the names it used
        (all of them, at the glob's visibility, when the glob itself re-exports)."""
        for names, node in sorted(self.model.index.items()):
            if inside(names, self.old):
                continue
            uses, _paths = scan(node.text, ())
            skip = [(c.declaration.start, c.declaration.end) for c in node.children if c.declaration.body]
            scope = self.paths.node_scope(node)
            explicit = scope.declared | set(scope.imports)
            subtree = set()
            for descendant in node.walk():
                subtree |= {t.value for t in tokens(descendant.text) if t.kind == 'ident'}
            for use in uses:
                if any(a <= use.start < b for a, b in skip):
                    continue
                added = []
                for segments, glob, _alias in use.leaves:
                    if not glob or not segments:
                        continue
                    resolved = resolve(segments, names, self.model.index)
                    if resolved is None or resolved[1]:
                        continue
                    for name, path in sorted(self.dropped.get(resolved[0], {}).items()):
                        if name in explicit or (not use.visibility and name not in subtree):
                            continue
                        added.append((path, False, None if path[-1] == name else name))
                if added:
                    self.edit(node.source, node.offset + use.end, node.offset + use.end,
                              '\n' + use_text(use.visibility, added))

    def library(self):
        for names, node in sorted(self.model.index.items()):
            skip = [(c.declaration.start, c.declaration.end) for c in node.children if c.declaration.body]
            if names == self.old[:-1]:
                declaration = self.model.index[self.old].declaration
                skip.append((declaration.start, declaration.end))
            uses, paths = scan(node.text, self.paths.module_heads(node))
            for use in uses:
                if any(a <= use.start < b for a, b in skip):
                    continue
                leaves, changed, dropped = [], False, False
                for segments, glob, alias in use.leaves:
                    if segments and self.reexport_into_subtree(node, use, segments):
                        name = alias or segments[-1]
                        path = self.rewritten(node, segments) or self.new_path(node, segments)
                        self.removed_reexports.append('::'.join(names + (name,)))
                        self.dropped.setdefault(names, {})[name] = path
                        dropped = True
                        if name in self.own_words(node, use):
                            self.private.append((node, path, name))
                        continue
                    new = self.rewritten(node, segments) if segments else None
                    if new is None:
                        leaves.append((segments, glob, alias))
                    else:
                        keep = alias if alias else (segments[-1] if new[-1] != segments[-1] and not glob else None)
                        leaves.append((new, glob, keep))
                        changed = True
                if changed or dropped:
                    text = use_text(use.visibility, leaves) if leaves else ''
                    self.edit(node.source, node.offset + use.start, node.offset + use.end, text)
            for path in paths:
                if any(a <= path.start < b for a, b in skip):
                    continue
                new = self.rewritten(node, path.segments)
                if new is not None:
                    self.edit(node.source, node.offset + path.start, node.offset + path.end, '::'.join(new))
            if inside(names, self.old):
                self.visibilities(node, skip)

    def visibilities(self, node, skip):
        for start, end, inner in visibility_markers(node.text):
            if any(a <= start < b for a, b in skip):
                continue
            scope = scope_of('pub(' + inner + ')', node.names)
            if scope == 'pub':
                continue
            if inside(scope, self.old):
                # a restriction inside the subtree moves with it; only a `crate::` spelling
                # names the old place
                if inner.replace(' ', '').startswith('incrate'):
                    self.edit(node.source, node.offset + start, node.offset + end,
                              stated(self.new + tuple(scope[len(self.old):])))
                continue
            item_module = self.new + node.names[len(self.old):]
            if inside(item_module, scope):
                if inner.replace(' ', '') in ('super',) or inner.startswith('in super') or inner.startswith('in self'):
                    self.edit(node.source, node.offset + start, node.offset + end, stated(scope))
                continue
            widened = stated(common(scope, item_module))
            self.widenings.append({'item': '::'.join(node.names) + ' @' + str(start), 'from': f'pub({inner})',
                                   'to': widened})
            self.edit(node.source, node.offset + start, node.offset + end, widened)

    def external(self, files):
        for path in files:
            text = path.read_text(encoding='utf-8')
            if LIBRARY not in text:
                continue
            uses, _ = scan(text, ())
            covered = []
            for use in uses:
                covered.append((use.start, use.end))
                leaves, changed = [], False
                for segments, glob, alias in use.leaves:
                    new = self.external_path(segments)
                    if new is None:
                        leaves.append((segments, glob, alias))
                    else:
                        leaves.append((new, glob, alias))
                        changed = True
                if changed:
                    self.edit(path, use.start, use.end, use_text(use.visibility, leaves))
            stream, i = tokens(text), 0
            while i < len(stream):
                token = stream[i]
                if token.value != LIBRARY or (i >= 2 and stream[i - 1].value == ':') or \
                        any(a <= token.start < b for a, b in covered):
                    i += 1
                    continue
                segments, j = [LIBRARY], i
                while j + 3 < len(stream) and stream[j + 1].value == ':' and stream[j + 2].value == ':' \
                        and stream[j + 3].kind == 'ident':
                    segments.append(stream[j + 3].value)
                    j += 3
                new = self.external_path(tuple(segments))
                if new is not None:
                    self.edit(path, token.start, stream[j].end, '::'.join(new))
                i = j + 1

    def external_path(self, segments):
        if not segments or segments[0] != LIBRARY:
            return None
        rest = segments[1:]
        door = bool(rest) and rest[0] == DOOR
        canonical = tuple(rest[1:] if door else rest)
        if canonical[:len(self.old)] != self.old:
            return None
        return (LIBRARY,) + ((DOOR,) if door else ()) + self.new + canonical[len(self.old):]


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('lib', type=Path)
    parser.add_argument('old')
    parser.add_argument('new_parent')
    parser.add_argument('--name')
    parser.add_argument('--root', type=Path, default=Path.cwd())
    parser.add_argument('--report', type=Path)
    parser.add_argument('--dry-run', action='store_true')
    args = parser.parse_args(argv)
    root = args.root.resolve()
    old = tuple(args.old.split('::'))
    new_parent = tuple(args.new_parent.split('::')) if args.new_parent else ()
    model = LibraryModel(args.lib, root)
    move = ModuleMove(model, old, new_parent, args.name or old[-1])
    mapping = move.file_map()
    move.library()
    move.declarations()
    move.globs()
    move.private_imports()
    library_root = Path(args.lib).resolve().parent.parent
    others = [p for p in sorted((root / 'workspaces').rglob('*.rs'))
              if library_root not in p.parents and 'target' not in p.parts]
    move.external(others)
    texts = {path: replace_spans(path.read_text(encoding='utf-8'), changes) for path, changes in move.edits.items()}
    problems = []
    for path, text in texts.items():
        try:
            pairs(tokens(text))
        except ValueError as error:
            problems.append(f'{path}: {error}')
    report = {'move': '::'.join(old) + ' -> ' + '::'.join(move.new), 'files': len(mapping),
              'edited_files': len(texts), 'widenings': move.widenings,
              'removed_reexports': move.removed_reexports, 'problems': problems}
    if args.report:
        args.report.mkdir(parents=True, exist_ok=True)
        (args.report / ('module-move-' + old[-1] + '.json')).write_text(json.dumps(report, indent=1) + '\n',
                                                                         encoding='utf-8', newline='\n')
    print(json.dumps({k: (len(v) if isinstance(v, list) else v) for k, v in report.items()}))
    if problems or args.dry_run:
        return 1 if problems else 0
    for path, text in texts.items():
        path.write_text(text, encoding='utf-8', newline='\n')
    for source, target in mapping.items():
        target.parent.mkdir(parents=True, exist_ok=True)
        subprocess.run(['git', 'mv', str(source), str(target)], check=True, cwd=root)
    for source in mapping:
        directory = source.parent
        while directory != root and directory.exists() and not any(directory.iterdir()):
            directory.rmdir()
            directory = directory.parent
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
