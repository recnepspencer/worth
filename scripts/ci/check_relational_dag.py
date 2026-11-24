#!/usr/bin/env python3
"""Ratchet source dependencies across the ten Relational packages."""
import argparse
from collections import Counter, defaultdict
import json
from pathlib import Path
import sys
import tomllib

from relational_dag_rust import (
    Surface, alias_target, bare_pub_module, contacts, identifiers, resolved_origin,
    section, self_types, test_spans,
)
from relational_dag_sizes import check_owner_sizes
from relational_dag_workspace import WorkspaceModel, physical_owners

SOURCE = Path('crates/worth-relational/src')
CONFIG = Path('scripts/ci/relational_dag_owners.toml')
BASELINE = Path('scripts/ci/relational_split_baseline.json')
PROJECTIONS = ('production_explicit', 'production_resolved', 'inherent_impl', 'cfg_test')
TESTKIT = 'testkit'
TEST_PATHS = {('tests',), ('presentation', 'harness'), ('test_execution_policy',)}
EXTRACTED = {
    'contracts', 'storage', 'branch', 'lineage', 'mutation', 'merge',
    'indexes', 'durability', 'publication',
}


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
        self.extracted = physical_owners(root, self.owners, root / SOURCE)
        self.physical = {source: self.packages[name.replace('-', '_')]
                         for name, source in self.extracted.items()}
        physical_names = set(self.physical.values())
        for path, name in self.paths.items():
            if name in physical_names:
                raise ValueError(f'physical owner {name} cannot own Runtime path {path}')
            if name != self.seed and name not in self.owners:
                raise ValueError(f'unknown owner {name} for {path}')
        for name, owner in self.owners.items():
            for dep in owner['depends']:
                if self.owners[dep]['layer'] >= owner['layer']:
                    raise ValueError(f'cyclic owner rule {name} -> {dep}')

    def rule(self, relative):
        for prefix in prefixes(relative):
            if prefix in self.paths:
                return prefix
        raise ValueError(f'unassigned source: {SOURCE / relative}')

    def locations(self, source):
        """The virtual package and, where governed, its internal responsibility."""
        for directory, owner in self.physical.items():
            if source.is_relative_to(directory):
                relative = source.relative_to(directory)
                break
        else:
            relative = source.relative_to(self.root / SOURCE)
            owner = self.paths[self.rule(relative)]
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
        file_rules = {p: self.map.rule(p.relative_to(base)) for p in sorted(files)}
        unused_rules = self.map.paths.keys() - set(file_rules.values())
        if unused_rules:
            raise ValueError(f'unused owner rules: {", ".join(sorted(unused_rules))}')
        self.file_owners = {p: self.map.paths[rule] for p, rule in file_rules.items()}
        for directory, owner in self.map.physical.items():
            self.file_owners.update((p.resolve(), owner) for p in directory.rglob('*') if p.is_file())
        self.model = WorkspaceModel(self.root, base, self.map.extracted)
        self.locations = {names: self.map.locations(node.source)
                          for names, node in self.model.index.items()}
        unmounted = {p for p in self.file_owners if p.suffix == '.rs'} - {
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
        resolved = self.model.resolve(segments, names, self.model.index)
        if package and resolved is None:
            external = '::'.join(segments)
            return ((package, None), external), []
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
                if resolved and self.model.is_physical_forward(resolved, origin):
                    explicit = defining[0]
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
            rooted = self.model.resolve(contact.segments, names, self.model.index)
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
                    self.test_surface = Surface(
                        self.model.tree, self.model.index, (), resolver=self.model.resolve)
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


    def subordinate_test_export_tokens(self, exports):
        """Public test tokens classified by the source that defines each export."""
        result = set()
        for path in exports:
            *modules, token = path.split('::')
            if not token.endswith('ForTest'):
                continue
            origin = resolved_origin(self.model.surface, (), ('crate', *modules, token))
            if origin is None or origin[0] == '<external>':
                raise ValueError(f'cannot resolve the defining owner for {path}')
            source = self.model.index[origin[0]].source
            if self.map.locations(source)[0] != 'runtime':
                result.add(token)
        return sorted(result)


def facade_failures(audit, baseline):
    exports = audit.exports()
    frozen = set(baseline['facade_exports'])
    current = set(exports)
    unbudgeted = set(audit.subordinate_test_export_tokens(exports)) - set(
        baseline['subordinate_test_export_tokens'])
    return (
        [f'facade export added: {name}' for name in sorted(current - frozen)]
        + [f'facade export removed: {name}' for name in sorted(frozen - current)]
        + [f'unbudgeted subordinate test export: {name}' for name in sorted(unbudgeted)]
    )


def totals(directions):
    return {direction: counts.total() for direction, counts in sorted(directions.items())}


def ordinary_storage_totals(projection, directions):
    """Count the Storage cut while retaining the residency ledger separately."""
    residency = tuple(f'crates/{crate}/src/storage/residency/' for crate in
                      ('worth-relational', 'worth-relational-storage', 'worth-relational-durability'))
    counts = {}
    for direction, references in directions.items():
        caller, callee = direction.split(' -> ')
        if caller != 'storage' and not (projection == 'inherent_impl' and callee == 'storage'):
            continue
        count = sum(value for (path, _), value in references.items()
                    if not path.startswith(residency))
        if count:
            counts[direction] = count
    return counts


def check_partition(audit, baseline):
    """Require the completed package cut and its three in-crate boundaries."""
    if set(audit.map.physical.values()) != EXTRACTED:
        raise ValueError('expected all nine extracted Relational owners')
    if set(audit.map.owners) != EXTRACTED | {'runtime'}:
        raise ValueError('expected exactly ten Relational owners')
    if set(audit.map.boundaries) != {'branch', 'mutation', 'durability'}:
        raise ValueError('expected the Branch, Mutation, and Durability internal boundaries')
    mutation_probe = audit.root / baseline['probe_3_live_mutation_source']
    if audit.file_owners.get(mutation_probe) != 'mutation':
        raise ValueError('the live mutation probe must stay in the Mutation package')


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
        check_partition(audit, baseline)
        failures = facade_failures(audit, baseline)
        audit.scan()
        failures.extend(check_owner_sizes(audit, args.root / SOURCE.parent / 'examples'))
    except (OSError, ValueError, KeyError) as error:
        print(f'relational DAG: {error}', file=sys.stderr)
        return 1
    for failure in failures:
        print(f'relational DAG: {failure}', file=sys.stderr)
    failed = bool(failures)
    for projection, directions in audit.ledger.items():
        ceiling = baseline['ceilings'][projection]
        failures = compare(directions, ceiling, baseline['references'][projection])
        print(f'{projection}: {sum(totals(directions).values())} / {sum(ceiling.values())}')
        for failure in failures:
            print(f'{projection}: {failure}', file=sys.stderr)
        failed |= bool(failures)
        ordinary = ordinary_storage_totals(projection, directions)
        ordinary_ceiling = baseline['ordinary_storage_ceilings'][projection]
        print(f'{projection} ordinary storage: {sum(ordinary.values())} / '
              f'{sum(ordinary_ceiling.values())}')
        for direction, count in ordinary.items():
            limit = ordinary_ceiling.get(direction, 0)
            if count > limit:
                print(f'{projection} ordinary storage: {direction}: {count} > {limit}',
                      file=sys.stderr)
                failed = True
    for direction, counts in audit.reexports.items():
        for path, symbol in counts:
            print(f'upward pub use: {direction}: {path} :: {symbol}', file=sys.stderr)
            failed = True
    print(f'assigned source files: {len(audit.file_owners)}')
    return int(failed)


if __name__ == '__main__':
    sys.exit(main())
