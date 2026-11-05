#!/usr/bin/env python3
"""Move ownership units (a type with all its impls, or a helper function/constant) to new modules.

Usage:
    python scripts/query_split/unit_move.py <lib.rs> <moves.json> [--report DIR] [--dry-run]

`moves.json` is `[{"item": "<crate path from domain_computation>", "to": "<module path>"}, ...]`
(the `moves` of an `ownership_simulation.py` result, with `to` as a full module path).

Items from one source file going to one destination land together in a new file
`<destination>/<source stem>.rs`, declared from the destination as a private module whose
items keep (or restate) their visibility. Then:
- the moved text keeps its doc comments and attributes; its paths are written from `crate::`,
  and every name it used from its old scope is imported by its definition's path;
- a unit takes every impl the unit rule gives it (an impl of a crate trait that sits above the
  destination stays with that trait);
- every path in the library, courts and testkits that led to a moved item now names its new
  module; a re-export of a moved item inside its old owner's top-level module is removed (no
  shim at an old path); a glob that used to bring it in gains an explicit import where its
  module's own text names it;
- a visibility restriction that no longer covers the item's users is widened to the nearest
  common ancestor and recorded; nothing becomes `pub` that was not.
`src_court_doors.py items` then regenerates the item doors for rewritten court paths.
"""
import argparse
from collections import defaultdict
import json
import os
from pathlib import Path
import re
import sys

sys.path.insert(0, str(Path(__file__).resolve().parent))
sys.path.insert(0, str(Path(__file__).resolve().parent.parent / 'test_topology'))

from item_mentions import declarations  # noqa: E402
from module_graph import LibraryModel  # noqa: E402
from module_move import LIBRARY, common, inside, removal_end, use_text  # noqa: E402
from ownership_units import header_parts, units  # noqa: E402
from path_resolution import Paths  # noqa: E402
from rust_source import replace_spans, tokens  # noqa: E402
from src_door_items import DOOR  # noqa: E402
from src_occurrences import scan  # noqa: E402
from src_scope import scope_items  # noqa: E402
from vocabulary_visibility import scope_of, stated  # noqa: E402

HELPER_KINDS = ('fn', 'const', 'static')
KEYWORDS = ('struct', 'enum', 'union', 'type', 'trait') + HELPER_KINDS


def comment_start(text, start):
    """The start of the attributes and comment lines directly above an item."""
    while start:
        line_start = text.rfind('\n', 0, start - 1) + 1
        line = text[line_start:start].strip()
        if line.startswith('//') and not line.startswith('//!') or line.startswith('#['):
            start = line_start
        else:
            break
    return start


class UnitMove:
    def __init__(self, model, moves):
        self.model, self.paths = model, Paths(model)
        self.edits = defaultdict(list)
        self.created = {}
        self.widenings = []
        self.removed_reexports = []
        self.problems = []
        self.target = {}            # (module, name) -> new module tuple
        self.span_of = {}           # (module, name) -> [(source, start, end)]
        found = {}
        for d in declarations(model, (), KEYWORDS):
            found[(d.module, d.name)] = d
        all_units, _ = units(model)
        for move in moves:
            parts = tuple(move['item'].split('::'))
            key = (parts[:-1], parts[-1])
            to = tuple(move['to'].split('::'))
            if key not in found:
                macro = self.macro_static(key)
                if macro is None:
                    self.problems.append(f'no declaration for {move["item"]}')
                    continue
                found[key] = macro
            self.target[key] = to
        self.declaration = found
        self.units = all_units

    def edit(self, path, start, end, text):
        self.edits[Path(path)].append((start, end, text))

    def macro_static(self, key):
        """A `thread_local!` block declaring `key`'s static, as a declaration-like record."""
        node = self.model.index.get(key[0])
        if node is None:
            return None
        for m in re.finditer(r'thread_local!\s*\{', node.text):
            depth, i = 0, m.end() - 1
            while i < len(node.text):
                if node.text[i] == '{':
                    depth += 1
                elif node.text[i] == '}':
                    depth -= 1
                    if depth == 0:
                        break
                i += 1
            body = node.text[m.start():i + 1]
            if re.search(r'\bstatic\s+%s\b' % re.escape(key[1]), body):
                if len(re.findall(r'\bstatic\s+\w+', body)) > 1:
                    return None
                class Macro:
                    pass
                record = Macro()
                record.module, record.name = key
                record.start = node.offset + comment_start(node.text, m.start())
                record.end = node.offset + i + 1
                return record
        return None

    # the text that moves
    def impl_spans(self, key):
        """(source, start, end) of each impl block the unit takes."""
        unit = self.units.get(key)
        if unit is None:
            return []
        destination = self.target[key]
        result = []
        for impl in unit.impls:
            trait_key = impl.trait[1] if impl.trait and impl.trait[0] == 'item' else None
            if trait_key is not None and trait_key != key and trait_key not in self.target:
                # An impl of a crate trait that is not moving stays with the trait when the
                # trait lives above the destination; otherwise it moves with the type.
                if not inside(destination, trait_key[0][:len(destination)]) and \
                        len(trait_key[0]) > 1 and trait_key[0][:2] == key[0][:2] and \
                        trait_key[0] != key[0]:
                    continue
            node = self.model.index[impl.module]
            for _a, _v, keyword, _n, start, end in scope_items(node.text):
                if keyword != 'impl':
                    continue
                body = node.text[start:end]
                header = ' '.join(header_parts(body)[3].split())[:160]
                if header == impl.header:
                    begin = comment_start(node.text, start)
                    result.append((impl.module, node.source, node.offset + begin,
                                   node.offset + removal_end(node.text, end)))
                    break
        return result

    def plan(self):
        groups = defaultdict(list)          # (source module, file, destination) -> [(start, end, key)]
        for key, to in sorted(self.target.items()):
            d = self.declaration[key]
            node = self.model.index[d.module]
            text = node.source.read_text(encoding='utf-8')
            start = comment_start(text, d.start)
            end = removal_end(text, d.end)
            groups[(d.module, node.source, to)].append((start, end, key))
            for module, source, s, e in self.impl_spans(key):
                groups[(module, source, to)].append((s, e, key))
        self.groups = groups

    def new_module(self, source, to):
        """A fresh child module name in `to` for items from `source`."""
        stem = source.stem if source.stem != 'mod' else source.parent.name
        taken = {c.names[-1] for c in self.model.index[to].children} | \
            {names[-1] for names in self.created.values() if names[:-1] == to}
        name, k = stem, 2
        while name in taken:
            name = f'{stem}_{k}'
            k += 1
        return to + (name,)

    def absolute(self, node, segments):
        found = self.paths.resolve(node, segments)
        if found is None:
            return None
        module, rest, _rooted = found
        if rest and module in self.model.surface.scopes:
            origin = self.model.surface.origin(module, rest[0])[0]
            if origin and origin[0] != '<external>' and origin[1]:
                key = (origin[0], origin[1])
                home = self.created_home(key) or origin[0]
                return ('crate',) + home + (origin[1],) + tuple(rest[1:])
        return ('crate',) + module + tuple(rest)

    def created_home(self, key):
        return self.created.get(key)

    def build(self):
        self.plan()
        # Assign each moved key its new module first: references are rewritten to it.
        files = {}
        for (module, source, to), spans in sorted(self.groups.items(), key=lambda kv: (str(kv[0][1]), kv[0][2])):
            key_group = (source, to)
            if key_group not in files:
                files[key_group] = self.new_module(source, to)
                self.created[('<file>', key_group)] = files[key_group]
            for _s, _e, key in spans:
                self.created[key] = files[key_group]
        self.files = files
        bodies = defaultdict(list)
        for (module, source, to), spans in self.groups.items():
            node = self.model.index[module]
            text = source.read_text(encoding='utf-8')
            for start, end, key in sorted(set(spans)):
                piece = text[start:end]
                bodies[files[(source, to)]].append((module, node, start - node.offset, piece))
                self.edit(source, start, end, '')
        for new, pieces in bodies.items():
            self.write_file(new, pieces)

    def write_file(self, new, pieces):
        """The destination file: imports of every name the moved text used, then the text."""
        imports = set()
        texts = []
        here = {key[1] for key, home in self.created.items() if key[0] != '<file>' and home == new}
        for module, node, offset, piece in pieces:
            rewritten, names = self.rewrite_piece(node, offset, piece, new)
            texts.append(rewritten.strip('\n'))
            for name in names - here:
                origin = self.model.surface.origin(module, name)[0] if module in self.model.surface.scopes else None
                if origin is None or origin[0] == '<external>':
                    scope = self.paths.node_scope(node)
                    if name in scope.imports:
                        segments = scope.imports[name]
                        path = self.absolute(node, segments) if segments and segments[0] in ('crate', 'super', 'self') or \
                            (segments and self.paths.head_module(node, segments[0])) else tuple(segments)
                        if path:
                            imports.add(('::'.join(path), name if path[-1] != name else None))
                    continue
                if origin[1] is None:
                    continue
                home = self.created.get((origin[0], origin[1])) or origin[0]
                if home == new:
                    continue
                imports.add(('crate::' + '::'.join(home + (origin[1],)), name if origin[1] != name else None))
        doc = f'//! Moved whole from `{"::".join(pieces[0][0])}`: the items below and every impl they own.\n'
        lines = [doc]
        for path, alias in sorted(imports):
            lines.append(f'use {path}' + (f' as {alias}' if alias else '') + ';\n')
        body = '\n\n'.join(texts) + '\n'
        source = self.module_file(new)
        self.edit(source, 0, 0, ''.join(lines) + '\n' + body)
        self.declare(new)

    def rewrite_piece(self, node, offset, piece, new):
        """The piece with its rooted paths written from `crate::`, and the bare names it uses."""
        uses, paths = scan(node.text, self.paths.module_heads(node))
        changes = []
        for path in paths:
            if not (offset <= path.start < offset + len(piece)):
                continue
            if path.segments[0] in ('super', 'self', 'crate') or self.paths.head_module(node, path.segments[0]):
                absolute = self.absolute(node, path.segments)
                if absolute is not None:
                    changes.append((path.start - offset, path.end - offset, '::'.join(absolute)))
        for start, end, inner in visibility_spans(piece):
            scope = scope_of('pub(' + inner + ')', node.names)
            if scope == 'pub':
                continue
            if not inside(new, scope):
                widened = stated(common(scope, new))
                self.widenings.append({'item': '::'.join(new) + f' @{start}', 'from': f'pub({inner})', 'to': widened})
                changes.append((start, end, widened))
            else:
                changes.append((start, end, stated(scope)))
        text = replace_spans(piece, changes) if changes else piece
        names = {t.value for t in tokens(text) if t.kind == 'ident'}
        return text, names

    def module_file(self, new):
        parent = self.model.index[new[:-1]]
        base = parent.source.parent if parent.source.name in ('mod.rs', 'lib.rs') else parent.source.with_suffix('')
        return base / (new[-1] + '.rs')

    def declare(self, new):
        parent = self.model.index[new[:-1]]
        text = parent.text
        point = 0
        for _a, _v, keyword, _n, s, e in scope_items(text):
            if keyword == 'mod':
                point = removal_end(text, e)
        self.edit(parent.source, parent.offset + point, parent.offset + point,
                  f'pub(in crate::{"::".join(new[:-1])}) mod {new[-1]};\n')

    # references elsewhere
    def moved_origin(self, node, segments):
        """The moved key a path leads to, and the segments after it, or None."""
        found = self.paths.resolve(node, segments)
        if found is None:
            return None
        module, rest, _rooted = found
        if not rest or module not in self.model.surface.scopes:
            return None
        origin = self.model.surface.origin(module, rest[0])[0]
        if not origin or origin[0] == '<external>' or not origin[1]:
            return None
        key = (origin[0], origin[1])
        if key in self.target:
            return key, tuple(rest[1:]), module
        return None

    def library(self):
        moved_spans = defaultdict(list)
        for (module, source, _to), spans in self.groups.items():
            node = self.model.index[module]
            for s, e, _k in spans:
                moved_spans[module].append((s - node.offset, e - node.offset))
        for names, node in sorted(self.model.index.items()):
            skip = [(c.declaration.start, c.declaration.end) for c in node.children if c.declaration.body]
            skip += moved_spans.get(names, [])
            uses, paths = scan(node.text, self.paths.module_heads(node))
            for use in uses:
                if any(a <= use.start < b for a, b in skip):
                    continue
                leaves, changed = [], False
                for segments, glob, alias in use.leaves:
                    hit = self.moved_origin(node, segments) if segments and not glob else None
                    if hit is None:
                        leaves.append((segments, glob, alias))
                        continue
                    key, rest, through = hit
                    changed = True
                    owner = key[0][:3] if key[0][:2] == ('domain_computation', 'primary_graph') else key[0][:2]
                    if use.visibility and inside(names, owner) and names != ('facade',) and \
                            not names[:1] == ('facade',):
                        # a re-export inside the old owner: no shim at an old path
                        self.removed_reexports.append('::'.join(names + (alias or segments[-1],)))
                        if (alias or segments[-1]) in self.own_words(node, use):
                            leaves.append((('crate',) + self.created[key] + (key[1],) + rest, False,
                                           alias if alias and alias != key[1] else None))
                            self.private_use.append((node, use))
                        continue
                    new = ('crate',) + self.created[key] + (key[1],) + rest
                    keep = alias if alias else (segments[-1] if new[-1] != segments[-1] else None)
                    leaves.append((new, False, keep))
                if changed:
                    visibility = use.visibility
                    if (node, use) in self.private_use:
                        visibility = ''
                    text = use_text(visibility, leaves) if leaves else ''
                    self.edit(node.source, node.offset + use.start, node.offset + use.end, text)
            for path in paths:
                if any(a <= path.start < b for a, b in skip):
                    continue
                hit = self.moved_origin(node, path.segments)
                if hit is None or len(path.segments) < 2:
                    continue
                key, rest, _through = hit
                new = ('crate',) + self.created[key] + (key[1],) + rest
                self.edit(node.source, node.offset + path.start, node.offset + path.end, '::'.join(new))
        self.globs()

    def own_words(self, node, use):
        text = node.text
        spans = [(c.declaration.start, c.declaration.end) for c in node.children if c.declaration.body]
        spans.append((use.start, use.end))
        return {t.value for t in tokens(text) if t.kind == 'ident' and not any(a <= t.start < b for a, b in spans)}

    def globs(self):
        """A module whose glob used to bring a moved item imports it where its own text names it."""
        moved_names = defaultdict(set)
        for key in self.target:
            moved_names[key[1]].add(key)
        for names, node in sorted(self.model.index.items()):
            if names not in self.model.surface.scopes:
                continue
            scope = self.model.surface.scopes[names]
            if not scope.globs:
                continue
            words = {t.value for t in tokens(node.own_text()) if t.kind == 'ident'}
            added = []
            for word in sorted(words & set(moved_names)):
                if word in scope.declared or word in scope.imports:
                    continue
                origin = self.model.surface.origin(names, word)[0]
                if origin and (origin[0], origin[1]) in self.target:
                    key = (origin[0], origin[1])
                    if self.created[key] == names:
                        continue
                    added.append('use crate::' + '::'.join(self.created[key] + (key[1],)) + ';\n')
            if added:
                point = 0
                for _a, _v, keyword, _n, s, e in scope_items(node.text):
                    if keyword == 'use':
                        point = removal_end(node.text, e)
                self.edit(node.source, node.offset + point, node.offset + point, ''.join(added))

    def external(self, files):
        """Court and testkit paths `worth_query_execution::[court_doors_for_test::]...::Item`."""
        old_paths = {}
        for key, home in self.created.items():
            if key[0] == '<file>':
                continue
            old_paths[key[0] + (key[1],)] = home + (key[1],)
        for path in files:
            text = path.read_text(encoding='utf-8')
            if LIBRARY not in text:
                continue
            stream, changes, i = tokens(text), [], 0
            while i < len(stream):
                token = stream[i]
                if token.value != LIBRARY:
                    i += 1
                    continue
                segments, j = [], i
                while j + 3 < len(stream) and stream[j + 1].value == ':' and stream[j + 2].value == ':' \
                        and stream[j + 3].kind == 'ident':
                    segments.append(stream[j + 3].value)
                    j += 3
                door = bool(segments) and segments[0] == DOOR
                canonical = tuple(segments[1:] if door else segments)
                for cut in range(len(canonical), 1, -1):
                    if canonical[:cut] in old_paths:
                        new = (LIBRARY,) + ((DOOR,) if door else ()) + old_paths[canonical[:cut]] + canonical[cut:]
                        changes.append((token.start, stream[j].end, '::'.join(new)))
                        break
                i = j + 1
            if changes:
                for s, e, t in changes:
                    self.edit(path, s, e, t)

    def apply(self, dry_run):
        written = 0
        for path, edits in self.edits.items():
            text = path.read_text(encoding='utf-8') if path.exists() else ''
            edits = sorted(set(edits), key=lambda e: (e[0], e[1]))
            for (a1, b1, _t1), (a2, _b2, _t2) in zip(edits, edits[1:]):
                if a2 < b1:
                    self.problems.append(f'overlapping edits in {path} at {a2}')
            new = replace_spans(text, [(s, e, t) for s, e, t in edits])
            if not dry_run:
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(new, encoding='utf-8', newline='\n')
            written += 1
        return written


def visibility_spans(text):
    for m in re.finditer(r'\bpub\(\s*((?:in\s+)?[\w:]+)\s*\)', text):
        yield m.start(), m.end(), m.group(1)


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__.split('\n')[0])
    parser.add_argument('lib', type=Path)
    parser.add_argument('moves', type=Path)
    parser.add_argument('--root', type=Path, default=Path.cwd())
    parser.add_argument('--report', type=Path)
    parser.add_argument('--dry-run', action='store_true')
    args = parser.parse_args(argv)
    model = LibraryModel(args.lib, args.root)
    moves = json.loads(args.moves.read_text())
    mover = UnitMove(model, moves)
    mover.private_use = []
    mover.build()
    mover.library()
    crates = args.root / 'workspaces/worth-query/crates'
    files = [f for d in sorted(crates.iterdir()) if d.is_dir() and d.name != 'worth-query-execution'
             for f in d.rglob('*.rs') if 'target' not in f.parts]
    mover.external(files)
    written = mover.apply(args.dry_run)
    report = {'moves': len(mover.target), 'files_created': len(mover.files), 'files_edited': written,
              'widenings': mover.widenings, 'removed_reexports': sorted(set(mover.removed_reexports)),
              'problems': mover.problems}
    if args.report:
        args.report.mkdir(parents=True, exist_ok=True)
        (args.report / f'unit-move-{args.moves.stem}.json').write_text(json.dumps(report, indent=1) + '\n')
    print(json.dumps({k: (len(v) if isinstance(v, list) else v) for k, v in report.items()}))
    return 0


if __name__ == '__main__':
    sys.exit(main())
