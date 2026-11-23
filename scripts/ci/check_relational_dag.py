#!/usr/bin/env python3
"""Ratchet source dependencies before Relational's virtual owners become packages."""
import argparse
from collections import Counter, defaultdict
import json
from pathlib import Path
import sys
import tomllib

from relational_dag_rust import (
    LibraryModel, RelationalProductionView, Surface, alias_target, bare_pub_module, contacts,
    identifiers, resolve, resolved_origin, section, self_types, test_spans,
)

SOURCE = Path('crates/worth-relational/src')
CONFIG = Path('scripts/ci/relational_dag_owners.toml')
BASELINE = Path('scripts/ci/relational_split_baseline.json')
PROJECTIONS = ('production_explicit', 'production_resolved', 'inherent_impl', 'cfg_test')
TESTKIT = 'testkit'
TEST_PATHS = {('tests',), ('presentation', 'harness'), ('test_execution_policy',)}


def prefixes(path):
    """Try the exact file, then its module spelling and physical ancestors."""
    yield path.as_posix()
    if path.name == 'mod.rs':
        path = path.parent
    elif path.suffix == '.rs':
        path = path.with_suffix('')
    if path != Path('.'):
        yield path.as_posix()
    yield from (p.as_posix() for p in path.parents if p != Path('.'))


class OwnerMap:
    def __init__(self, root, config):
        self.root = root
        self.seed = config['court_seed']
        self.owners = config['owners']
        self.paths = config['paths']
        self.boundaries = {b['owner']: b for b in config['boundaries']}
        self.packages = {owner['crate'].replace('-', '_'): name
                         for name, owner in self.owners.items()}
        self.packages['worth_testkit_relational'] = TESTKIT
        for path, name in self.paths.items():
            if name != self.seed and name not in self.owners:
                raise ValueError(f'unknown owner {name} for {path}')
        for name, owner in self.owners.items():
            for dep in owner['depends']:
                if self.owners[dep]['layer'] >= owner['layer']:
                    raise ValueError(f'cyclic owner rule {name} -> {dep}')

    def assignment(self, source):
        return self.assignment_relative(source.relative_to(self.root / SOURCE))

    def assignment_relative(self, relative):
        for prefix in prefixes(relative):
            if prefix in self.paths:
                return self.paths[prefix]
        raise ValueError(f'unassigned source: {SOURCE / relative}')

    def locations(self, source):
        """The virtual package and, where governed, its internal responsibility."""
        relative = source.relative_to(self.root / SOURCE)
        owner = self.assignment_relative(relative)
        if owner not in self.boundaries:
            return owner, None
        boundary = self.boundaries[owner]
        for prefix in prefixes(relative):
            for name, paths in boundary['paths'].items():
                if prefix in paths:
                    return owner, name
        return owner, None

    def forbidden(self, source, target):
        caller, lane = source
        callee, target_lane = target
        if caller != callee:
            if caller in self.owners and callee not in self.owners[caller]['depends']:
                return f'{caller} -> {callee}'
        elif lane and target_lane and lane != target_lane:
            if [lane, target_lane] not in self.boundaries[caller]['allowed']:
                return f'{caller}/{lane} -> {callee}/{target_lane}'
        return None


class Audit:
    def __init__(self, root, config, *, unmounted_at_baseline=()):
        self.root = root.resolve()
        self.map = OwnerMap(self.root, config)
        base = self.root / SOURCE
        files = {p.resolve() for p in base.rglob('*') if p.is_file()}
        self.file_owners = {p: self.map.assignment(p) for p in sorted(files)}
        self.model = LibraryModel(base / 'lib.rs', self.root, production_view=RelationalProductionView)
        self.locations = {names: self.map.locations(node.source)
                          for names, node in self.model.index.items()}
        unmounted = {p for p in files if p.suffix == '.rs'} - {
            node.source for node in self.model.index.values()
        }
        for path in sorted(unmounted):
            if path.relative_to(self.root).as_posix() not in unmounted_at_baseline:
                raise ValueError(f'unmounted source: {path.relative_to(self.root)}')
        self.ledger = {key: defaultdict(Counter) for key in PROJECTIONS}
        self.reexports = defaultdict(Counter)
        self.test_surface = None

    def scan(self):
        for names, node in self.model.index.items():
            location = self.locations[names]
            if location[0] == self.map.seed:
                continue
            path = self.model.relative(node.source)
            own = node.own_text()
            view = self.model.views.get(names)
            if view is None:
                self.test_contacts(names, path, own)
            else:
                self.production(names, path, view.own_text())
                spans = test_spans(own)
                if spans:
                    self.test_contacts(names, path, section(own, spans, keep=True))
            self.impls(names, path, own)

    def paths(self, names, contact, words=None):
        segments = contact.segments
        package = self.map.packages.get(segments[0])
        if package:
            external = '::'.join(segments)
            return ((package, None), external), []
        resolved = resolve(segments, names, self.model.index)
        explicit = None
        if resolved is not None:
            module, rest = resolved
            explicit = self.locations[module], '::'.join(module + rest)
        origin = resolved_origin(self.model.surface, names, segments)
        defining = []
        if origin is not None and origin[0] != '<external>':
            module, item = origin
            if contact.glob:
                defining = [self.definition(target, symbol)
                            for name, (target, symbol) in self.model.surface.exports(module).items()
                            if target != '<external>' and (words is None or name in words)]
            else:
                defining = [self.definition(module, item)]
        return explicit, defining

    def definition(self, module, item):
        return self.locations[module], '::'.join(module + ((item,) if item else ()))

    def production(self, names, path, text):
        source = self.locations[names]
        for contact in contacts(text):
            explicit, defining = self.paths(names, contact)
            if explicit:
                self.record('production_explicit', source, path, *explicit)
            if contact.public:
                self.check_reexport(source, path, explicit, defining)
        for _, target, item, _ in self.model.resolved_references(names):
            self.record('production_resolved', source, path, *self.definition(target, item), unique=True)

    def test_contacts(self, names, path, text):
        source = self.locations[names]
        if self.map.owners[source[0]]['layer'] >= 6:
            return
        imports = {ref.alias: ref.segments for ref in contacts(text) if ref.alias}
        words = identifiers(self.model.index[names].own_text())
        for contact in contacts(text, imports):
            explicit, defining = self.paths(names, contact, None if contact.public else words)
            rooted = resolve(contact.segments, names, self.model.index)
            if rooted and any(rooted[0][:len(prefix)] == prefix for prefix in TEST_PATHS):
                explicit = (self.map.seed, None), '::'.join(rooted[0] + rooted[1])
                defining = []
            if contact.public:
                self.check_reexport(source, path, explicit, defining)
            if explicit:
                self.record('cfg_test', source, path, *explicit)
            for target, symbol in set(defining):
                if explicit is None or target != explicit[0]:
                    self.record('cfg_test', source, path, target, symbol)

    def check_reexport(self, source, path, explicit, defining):
        for callee, symbol in set(defining + ([explicit] if explicit else [])):
            direction = self.map.forbidden(source, callee)
            if direction:
                self.reexports[direction][(path, symbol)] += 1

    def impls(self, names, path, text):
        source = self.locations[names]
        for segments in self_types(text):
            surface = self.model.surface
            if names not in surface.scopes:
                if self.test_surface is None:
                    self.test_surface = Surface(self.model.tree, self.model.index, ())
                surface = self.test_surface
            origin = self.type_origin(surface, names, segments)
            if not origin or origin[0] == '<external>' or origin[1] is None:
                continue
            target, symbol = self.definition(*origin)
            if source[0] != target[0]:
                self.add('inherent_impl', f'{source[0]} -> {target[0]}', (path, symbol))

    def type_origin(self, surface, names, segments):
        seen = set()
        origin = resolved_origin(surface, names, segments)
        while origin and origin[0] != '<external>' and origin not in seen:
            module, name = origin
            if surface.scopes[module].kinds.get(name) != 'type':
                break
            seen.add(origin)
            target = alias_target(surface.index[module].own_text(), name)
            if not target:
                break
            origin = resolved_origin(surface, module, target)
        return origin

    def record(self, projection, source, path, target, symbol, *, unique=False):
        direction = self.map.forbidden(source, target)
        if direction:
            self.add(projection, direction, (path, symbol), unique)

    def add(self, projection, direction, key, unique=False):
        if unique:
            self.ledger[projection][direction][key] = 1
        else:
            self.ledger[projection][direction][key] += 1

    def exports(self):
        """Public facade names, including explicit aliases, wildcards and module exports."""
        surface = self.model.surface

        def walk(module, prefix, active):
            scope = surface.scopes[module]
            for export, (target, item) in surface.exports(module).items():
                name = f'{prefix}::{export}'
                yield name
                if item is None and target not in active:
                    yield from walk(target, name, active | {target})
            # Keep explicit named exports even when their origin is macro-defined.
            for export in scope.public_imports - scope.gated:
                yield f'{prefix}::{export}'
            parent = surface.index[module]
            for child in parent.children:
                if child.names in surface.scopes and bare_pub_module(parent, child):
                    name = f'{prefix}::{child.names[-1]}'
                    yield name
                    yield from walk(child.names, name, active | {child.names})

        return sorted(set(walk(('facade',), 'facade', {('facade',)})))


def totals(directions):
    return {direction: counts.total() for direction, counts in sorted(directions.items())}


def compare(directions, ceilings, rows):
    """Each direction and each named reference can only shrink."""
    failures = []
    for direction, counts in sorted(directions.items()):
        limit = ceilings.get(direction, 0)
        if counts.total() > limit:
            failures.append(f'{direction}: {counts.total()} > {limit}')
        permitted = {(path, symbol): count for path, symbol, count in rows.get(direction, [])}
        for (path, symbol), count in sorted(counts.items()):
            previous = permitted.get((path, symbol), 0)
            if count > previous:
                failures.append(f'{direction}: {path} :: {symbol}: {count} > {previous}')
    return failures


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, default=Path(__file__).resolve().parents[2])
    args = parser.parse_args()
    try:
        config = tomllib.loads((args.root / CONFIG).read_text())
        baseline = json.loads((args.root / BASELINE).read_text())
        audit = Audit(args.root, config)
        audit.scan()
    except (OSError, ValueError, KeyError) as error:
        print(f'relational DAG: {error}', file=sys.stderr)
        return 1
    failed = False
    for projection, directions in audit.ledger.items():
        ceiling = baseline['ceilings'][projection]
        failures = compare(directions, ceiling, baseline['references'][projection])
        print(f'{projection}: {sum(totals(directions).values())} / {sum(ceiling.values())}')
        for failure in failures:
            print(f'{projection}: {failure}', file=sys.stderr)
        failed |= bool(failures)
    for direction, counts in audit.reexports.items():
        for path, symbol in counts:
            print(f'upward pub use: {direction}: {path} :: {symbol}', file=sys.stderr)
            failed = True
    print(f'assigned source files: {len(audit.file_owners)}')
    return int(failed)


if __name__ == '__main__':
    sys.exit(main())
