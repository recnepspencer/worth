"""Rewrite moved test source so every reference names its canonical access path.

Canonical paths, in order:
- an item a bare-`pub` module of the library exports: its shortest public path;
- any other library item: its defining module path, with the module chain and the item
  widened to `pub` (each widening is recorded, never implied);
- an external re-export: the external path it re-exports;
- a moved item: its module path in the testkit or court crate that now owns it.
Glob imports become explicit imports of the names the module actually uses.
"""
from dataclasses import dataclass, field

from rust_source import replace_spans, tokens
from src_occurrences import scan, visibility_markers
from src_paths import resolve

LIBRARY = 'worth_query_execution'
COURT_KEEP = ('use', 'macro_rules')


@dataclass
class Ledger:
    """Everything a rewrite needs from production, plus what it could not resolve."""
    widen_modules: set = field(default_factory=set)
    widen_items: set = field(default_factory=set)
    hooks: set = field(default_factory=set)
    unresolved: set = field(default_factory=set)
    cross_court: set = field(default_factory=set)
    widen_imports: set = field(default_factory=set)


def public_paths(surface):
    """Defining item -> shortest public path (module tuple, exported name)."""
    best = {}
    for module in sorted(surface.public_modules, key=lambda m: (len(m), m)):
        for name, origin in sorted(surface.exports(module).items()):
            if origin not in best or len(module) < len(best[origin][0]):
                best[origin] = (module, name)
    return best


def leading_inner_attributes(text):
    stream = tokens(text)
    end, i = 0, 0
    while i + 2 < len(stream) and stream[i].value == '#' and stream[i + 1].value == '!':
        depth, j = 0, i + 2
        while j < len(stream):
            depth += {'[': 1, ']': -1}.get(stream[j].value, 0)
            if depth == 0:
                break
            j += 1
        end, i = stream[j].end, j + 1
    return text[:end], end


def part_text(record, part, disposition):
    """The source of one part of a module scope; a split keeps tests and imports in the court."""
    text = record.node.own_text()
    if record.kept_impls or record.doors:
        # Library-trait impls stay in the library; inherent impls become door traits.
        text = replace_spans(text, [(start, end, '') for start, end in record.kept_impls]
                             + [(door.start, door.end, door.replacement) for door in record.doors])
        record = record_without_kept(record, text)
    if disposition != 'split':
        return text
    inner, start = leading_inner_attributes(text)
    pieces = [inner]
    for item in record.items:
        if item.end <= start:
            continue
        macro = item.name == '!' or item.keyword == 'macro_rules'
        court = item.test or macro or item.keyword in COURT_KEEP
        testkit = not item.test and not macro
        if (part == 'court' and court) or (part == 'testkit' and testkit):
            pieces.append('\n' + text[item.start:item.end])
    return ''.join(pieces) + '\n'


def record_without_kept(record, text):
    from src_move_model import items_of
    import copy
    clone = copy.copy(record)
    clone.items = items_of(text)
    return clone


class Rewriter:
    def __init__(self, model, placement):
        self.model = model
        self.placement = placement
        self.public = public_paths(model.production)
        self.ledger = Ledger()
        self.pending_imports = set()

    def owner_part(self, names):
        """(crate, module path) that holds a moving scope's non-test items."""
        parts = self.placement[names]
        return parts.get('testkit') or parts.get('court')

    def moved(self, module, name):
        """Whether (module, name) now lives in a generated crate."""
        record = self.model.records.get(module)
        if record is None or name in record.kept_names:
            return False
        return name in record.extracted if record.extracted else True

    def item_path(self, origin, reader_crate):
        module, name = origin
        if module == '<external>':
            return name
        record = self.model.records.get(module)
        if record is not None and name in record.kept_names:
            # A kept library test door: reached at its library path, inside `pub` kept modules.
            outermost = next(i for i in range(1, len(module) + 1)
                             if module[:i] in self.model.records or i == len(module))
            self.widen_chain(module[:outermost - 1])
            return '::'.join((LIBRARY,) + module + (name,))
        if self.moved(module, name):
            crate, path = self.owner_part(module)
            if crate != reader_crate and '-court-' in crate and '-court-' in reader_crate:
                self.ledger.cross_court.add(('::'.join(module), name or ''))
            prefix = 'crate' if crate == reader_crate else crate.replace('-', '_')
            return '::'.join((prefix,) + path + ((name,) if name else ()))
        if name is None:
            self.widen_chain(module)
            return '::'.join((LIBRARY,) + module)
        found, gated = self.model.production.origin(module, name)
        target = found if found is not None else (module, name)
        if gated or (found and name in self.model.production.scopes[found[0]].gated):
            self.ledger.hooks.add(target)
        if found is None:
            self.ledger.unresolved.add(target)
        public = self.public.get(target)
        if public is not None and not gated:
            return '::'.join((LIBRARY,) + public[0] + (public[1],))
        self.widen_chain(target[0])
        self.ledger.widen_items.add(target)
        return '::'.join((LIBRARY,) + target[0] + (target[1],))

    def widen_chain(self, module):
        for depth in range(1, len(module) + 1):
            if module[:depth] not in self.model.production.public_modules:
                self.ledger.widen_modules.add(module[:depth])

    def resolve_path(self, segments, names, reader_crate, context='use'):
        resolved = resolve(segments, names, self.model.index)
        if resolved is None:
            return None
        module, rest = resolved
        if not rest:
            return self.module_path(module, reader_crate)
        origin, _gated = self.model.everything.origin(module, rest[0])
        origin = origin or (module, rest[0])
        canonical = self.item_path(origin, reader_crate)
        if context == 'expr' and canonical.rsplit('::', 1)[-1] != rest[0]:
            # An alias keeps its spelling in the body; the module imports it under that name.
            self.pending_imports.add(f'use {canonical} as {rest[0]};')
            return '::'.join(rest)
        return '::'.join((canonical,) + rest[1:])

    def alias_site(self, module, name, seen=frozenset()):
        """The module whose `use` binds `name` as an alias, following glob imports."""
        scope = self.model.everything.scopes.get(module)
        if scope is None or module in seen or name in scope.declared:
            return None
        if name in scope.imports:
            return module
        for segments in scope.globs:
            target = self.model.everything.target(segments, module)
            if target is not None and not target[1]:
                found = self.alias_site(target[0], name, seen | {module})
                if found is not None:
                    return found
        return None

    def alias_path(self, site, name, origin, reader_crate):
        """An aliased name keeps its spelling: it is reached where the alias is declared."""
        if site in self.model.records and not self.model.records[site].extracted:
            crate, path = self.owner_part(site)
            prefix = 'crate' if crate == reader_crate else crate.replace('-', '_')
            return '::'.join((prefix,) + path + (name,))
        public = self.public.get(origin)
        if public is not None and public[1] == name:
            return '::'.join((LIBRARY,) + public[0] + (name,))
        self.widen_chain(site)
        self.ledger.widen_imports.add((site, name))
        if origin[0] != '<external>' and origin[1] is not None:
            # `pub use` re-exports only a `pub` item (E0364).
            self.ledger.widen_items.add(origin)
            _found, gated = self.model.production.origin(origin[0], origin[1])
            scope = self.model.production.scopes.get(origin[0])
            if gated or (scope is not None and origin[1] in scope.gated):
                self.ledger.hooks.add(origin)
        return '::'.join((LIBRARY,) + site + (name,))

    def module_path(self, module, reader_crate):
        if module in self.model.records and not self.model.records[module].extracted:
            crate, path = self.owner_part(module)
            prefix = 'crate' if crate == reader_crate else crate.replace('-', '_')
            return '::'.join((prefix,) + path)
        self.widen_chain(module)
        return '::'.join((LIBRARY,) + module)

    def rewrite(self, record, part, disposition, reader_crate):
        text = part_text(record, part, disposition)
        names = record.names
        stream = tokens(text)
        own = {i.name for i in record.items} | {c.names[-1] for c in record.node.children}
        words = {t.value for k, t in enumerate(stream)
                 if t.kind == 'ident' and not (k and stream[k - 1].value == '.')} - own
        uses, paths = scan(text)
        changes = []
        for declaration in uses:
            lines = []
            visibility = self.visibility(declaration.visibility, reader_crate)
            for segments, glob, alias in declaration.leaves:
                lines.extend(self.use_lines(segments, glob, alias, names, reader_crate, words, visibility))
            changes.append((declaration.start, declaration.end, '\n'.join(lines)))
        self.pending_imports = set()
        for occurrence in paths:
            replacement = self.resolve_path(occurrence.segments, names, reader_crate, 'expr')
            if replacement is not None:
                changes.append((occurrence.start, occurrence.end, replacement))
        text = replace_spans(text, changes)
        if self.pending_imports:
            bound = {alias or segments[-1] for d in scan(text)[0] for segments, glob, alias in d.leaves
                     if segments and not glob}
            fresh = sorted(line for line in self.pending_imports if line.rsplit(' ', 1)[-1].rstrip(';') not in bound)
            if fresh:
                inner, end = leading_inner_attributes(text)
                text = inner + '\n' + '\n'.join(fresh) + '\n' + text[end:]
        if disposition == 'split' or record.extracted:
            text = self.with_bare_imports(text, record, reader_crate)
        text = self.with_door_imports(text, record, reader_crate)
        changes = [(start, end, self.visibility('pub(' + inner + ')', reader_crate))
                   for start, end, inner in visibility_markers(text) if inner.startswith('in ')
                   or '-testkit-' in reader_crate]
        return replace_spans(text, changes)

    def with_door_imports(self, text, record, reader_crate):
        """A module that calls a door method imports the door trait that now provides it."""
        stream = tokens(text)
        called = {stream[i + 1].value for i in range(len(stream) - 1)
                  if stream[i].value in ('.', ':') and stream[i + 1].kind == 'ident'}
        lines = []
        for names, other in sorted(self.model.records.items()):
            for door in other.doors:
                crate, path = self.owner_part(names)
                if (names == record.names and crate == reader_crate) or not called & set(door.methods):
                    continue
                prefix = 'crate' if crate == reader_crate else crate.replace('-', '_')
                lines.append(f'use {prefix}::' + '::'.join(path + (door.trait,)) + ';')
        if not lines:
            return text
        inner, end = leading_inner_attributes(text)
        return inner + '\n' + '\n'.join(sorted(set(lines))) + '\n' + text[end:]

    def with_bare_imports(self, text, record, reader_crate):
        """A fragment of a module imports the sibling names it used but no longer holds."""
        from src_move_model import items_of
        stream = tokens(text)
        heads = {t.value for k, t in enumerate(stream) if t.kind == 'ident'
                 and not (k and stream[k - 1].value in ('.', ':'))
                 and not (k + 1 < len(stream) and stream[k + 1].value == '!')}
        defined = {item.name for item in items_of(text)}
        uses, _paths = scan(text)
        bound = {alias or segments[-1] for d in uses for segments, glob, alias in d.leaves if segments and not glob}
        lines = []
        for name in sorted((heads & self.model.visible(record.names)) - defined - bound):
            path = self.resolve_path(('self', name), record.names, reader_crate)
            if path is None or path == name:
                continue
            lines.append('use ' + path + ('' if path.rsplit('::', 1)[-1] == name else f' as {name}') + ';')
        if not lines:
            return text
        inner, end = leading_inner_attributes(text)
        return inner + '\n' + '\n'.join(lines) + '\n' + text[end:]

    def visibility(self, visibility, reader_crate):
        if not visibility:
            return ''
        if '-testkit-' in reader_crate:
            return 'pub'
        return 'pub(crate)' if visibility.startswith('pub(in') else visibility

    def use_lines(self, segments, glob, alias, names, reader_crate, words, visibility):
        prefix = (visibility + ' ') if visibility else ''
        resolved = resolve(segments, names, self.model.index)
        if resolved is None:
            return [prefix + 'use ' + '::'.join(segments) + ('::*' if glob else '') + (f' as {alias}' if alias else '') + ';']
        module, rest = resolved
        if glob and not rest:
            lines = []
            for n in sorted(self.model.visible(module) & words):
                path = self.resolve_path(segments + (n,), names, reader_crate)
                lines.append(prefix + 'use ' + path + ('' if path.rsplit('::', 1)[-1] == n else f' as {n}') + ';')
            scope = self.model.everything.scopes.get(module)
            for external in (scope.globs if scope else []):
                if resolve(external, module, self.model.index) is None:
                    lines.append(prefix + 'use ' + '::'.join(external) + '::*;')
            return lines
        path = self.resolve_path(segments, names, reader_crate)
        local = alias or segments[-1]
        tail = path.rsplit('::', 1)[-1]
        named = local if local not in ('self',) else None
        suffix = f' as {named}' if named and named != tail else ''
        return [prefix + 'use ' + path + ('::*' if glob else '') + suffix + ';']

