"""Resolve Relational source imports across the physical package boundary."""
from collections import defaultdict
import tomllib

from relational_dag_rust import LibraryModel, RelationalProductionView, Surface, bare_pub_module
from src_families import families
from src_paths import resolve, use_trees


class WorkspaceModel(LibraryModel):
    def __init__(self, root, runtime, extracted):
        self.repository = root
        self.roots = {name.replace('-', '_'): source
                      for name, source in extracted.items()}
        self.index = {}
        test_roots = []
        for crate, source in [('', runtime), *self.roots.items()]:
            tree, test_families = families(source / 'lib.rs')
            prefix = (crate,) if crate else ()
            if not crate:
                self.tree = tree
            for node in tree.walk():
                if not node.source.is_relative_to(source):
                    raise ValueError(f'module escapes its package: {node.source}')
                node.names = prefix + node.names
                previous = self.index.setdefault(node.names, node)
                if previous is not node:
                    raise ValueError(f'duplicate module: {node.names}')
            test_roots.extend(prefix + family.names for family in test_families)
        self.module_aliases = defaultdict(dict)
        physical_globs = defaultdict(list)
        for names, node in self.index.items():
            for segments, glob, alias in use_trees(node.own_text()):
                if not segments or segments[0] not in self.roots or segments not in self.index:
                    continue
                if glob:
                    physical_globs[names].append(segments)
                else:
                    self.module_aliases[names][alias or segments[-1]] = segments
        for names, targets in physical_globs.items():
            for target in targets:
                parent = self.index[target]
                for child in parent.children:
                    if bare_pub_module(parent, child):
                        self.module_aliases[names].setdefault(child.names[-1], child.names)
        self.surface = Surface(self.tree, self.index, test_roots, resolver=self.resolve)
        self.views = {names: RelationalProductionView(node) for names, node in self.index.items()
                      if not self.surface.in_family(names)}
        self._references = {}

    def resolve(self, segments, current, index):
        if not segments:
            return None
        head, *rest = segments
        prefix = current[:1] if current and current[0] in self.roots else ()
        if head == 'crate':
            base = prefix
            if prefix and rest and rest[0] != 'internal_port':
                port = prefix + ('internal_port',)
                if port + (rest[0],) in index or rest[0] in self.module_aliases[port]:
                    base = port
        elif head in self.roots:
            base = (head,)
        elif head in ('super', 'self'):
            base, position = current, 0
            while position < len(segments) and segments[position] in ('super', 'self'):
                if segments[position] == 'super':
                    if base == prefix:
                        return None
                    base = base[:-1]
                position += 1
            rest = segments[position:]
        elif current + (head,) in index or head in self.module_aliases[current]:
            base, rest = current, segments
        else:
            return resolve(segments, current, index)
        for i, part in enumerate(rest):
            direct = base + (part,)
            if direct in index:
                base = direct
            elif part in self.module_aliases[base]:
                base = self.module_aliases[base][part]
            else:
                return base, tuple(rest[i:])
        return base, ()


def physical_owners(root, owners, runtime):
    """Check declared Relational dependencies and return extracted source roots."""
    extracted = {owner['crate']: root / 'crates' / owner['crate'] / 'src'
                 for owner in owners.values() if owner.get('physical')}
    if not extracted:
        return {}
    relational = {entry['crate'] for entry in owners.values()} | {'worth-testkit-relational'}
    for owner in owners.values():
        crate = owner['crate']
        if crate not in extracted and crate != runtime.parent.name:
            continue
        source = extracted.get(crate, runtime)
        manifest = tomllib.loads((source.parent / 'Cargo.toml').read_text())
        actual = set()
        for table in (manifest, *manifest.get('target', {}).values()):
            for kind in ('dependencies', 'dev-dependencies', 'build-dependencies'):
                for alias, spec in table.get(kind, {}).items():
                    name = spec.get('package', alias) if isinstance(spec, dict) else alias
                    if name not in relational or name == 'worth-testkit-relational' and source == runtime:
                        continue
                    if kind != 'dependencies':
                        raise ValueError(f'{crate}: {name} must be an ordinary dependency')
                    actual.add(name)
        expected = {owners[name]['crate'] for name in owner['depends']} & extracted.keys()
        if actual != expected:
            raise ValueError(f'{crate}: Relational dependencies {sorted(actual)} != {sorted(expected)}')
        if crate in extracted and 'features' in manifest:
            raise ValueError(f'{crate}: extracted owners must have one build')
    return extracted
