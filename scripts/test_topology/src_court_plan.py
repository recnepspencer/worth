"""Plan the move of a library's court-shaped source tests into testkit and court crates.

Dispositions and placement iterate to a fixpoint: a support scope read from another court
crate becomes testkit support, which can change sizes and therefore placement. Categories
come from the existing classifier over each court scope's own tree, plus reviewed overrides.
"""
from collections import defaultdict
from pathlib import Path

from cargo_inventory import metadata
from court_partition import CEILING, classification
from frontier_adapter import frontiers
from rust_source import tokens
from src_hooks import all_gated_seeds, classify, method_seeds, staying_support
from src_move_model import MoveModel, add_extracted
from src_ownership import dispositions
from src_placement import named_products, place_courts, place_testkits
from src_rewrite import Rewriter

LIBRARY_PACKAGE = 'worth-query-execution'
# Generated scaffold modules and mounts are charged against the ceiling as one reserve.
SCAFFOLD_RESERVE = 300


def product_aliases(data):
    members = set(data['workspace_members'])
    result = {}
    for package in data['packages']:
        if package['id'] in members and '-court-' not in package['name'] and '-testkit' not in package['name'] \
                and 'fixtures' not in Path(package['manifest_path']).parts:
            result[package['name'].replace('-', '_')] = package['name']
    return result


def build(lib, repository, baseline, admissions, workspace_manifest, overrides, ceiling=CEILING):
    model = MoveModel(lib, repository, baseline, admissions)
    data = metadata(workspace_manifest)
    aliases = product_aliases(data)
    ids = {p['name']: p['id'] for p in data['packages']}

    def products(names):
        return named_products(model, names, aliases) | {LIBRARY_PACKAGE}

    memo = {}

    def frontier_of_products(selected):
        key = '+'.join(sorted(selected))
        if key not in memo:
            memo[key] = frontiers(data, {key: {ids[p] for p in selected}})[key]
        return memo[key]

    library_root = Path(lib).resolve().parent.parent
    foreign = {n for n, r in model.records.items() if not r.node.source.resolve().is_relative_to(library_root)}
    renames = overrides.get('renames', {})
    category_overrides = overrides.get('categories', {})

    def placed(measured):
        model.measured = measured
        unit_of = {names: record.family for names, record in model.records.items()}
        for _round in range(6):
            kinds = dispositions(model, lambda n: unit_of[n])
            testkits = place_testkits(model, kinds, products, frontier_of_products, ceiling - SCAFFOLD_RESERVE)
            kit_of = {n: crate for crate, members in testkits.items() for n in members}
            kit_products = {crate: set().union(*(products(n) for n in members)) for crate, members in testkits.items()}

            def frontier_of(family, names):
                selected = set()
                for n in names:
                    selected |= products(n)
                    for _reference, origin in model.records[n].uses:
                        if origin and origin[0] in kit_of:
                            selected |= kit_products[kit_of[origin[0]]]
                return frontier_of_products(selected)

            courts = place_courts(model, kinds, frontier_of, ceiling - SCAFFOLD_RESERVE)
            court_of = {n: crate for crate, row in courts.items() for n in row['members']}
            stable = all(unit_of.get(n) == court_of.get(n, kit_of.get(n)) for n in model.records)
            unit_of = {n: court_of.get(n, kit_of.get(n)) for n in model.records}
            if stable:
                break
        categories = {}
        for names in sorted(court_of):
            override = category_overrides.get('::'.join(names))
            category, evidence, questions = classification(model.records[names].node)
            categories[names] = {'category': override if override is not None else category,
                                 'evidence': evidence + (['reviewed override'] if override is not None else []),
                                 'questions': [] if override is not None else questions}
        placement = defaultdict(dict)
        for names, crate in court_of.items():
            prefix = tuple(s for s in categories[names]['category'].split('::') if s)
            placement[names]['court'] = (crate, prefix + hoisted(renamed(names, renames)))
        for names, crate in kit_of.items():
            placement[names]['testkit'] = (crate, renamed(names, renames))
        rewriter = Rewriter(model, placement)
        parts = {}
        for names in sorted(model.records):
            for part, (crate, path) in sorted(placement[names].items()):
                record = model.records[names]
                text = '' if names in foreign else rewriter.rewrite(record, part, kinds[names], crate)
                includes = [] if names in foreign or (kinds[names] == 'split' and part == 'testkit') else [
                    (literal, target, rewriter.rewrite(included_record(record, target), part, 'court', crate))
                    for literal, target in record.includes]
                parts[(names, part)] = {'crate': crate, 'path': path, 'text': text, 'foreign': names in foreign,
                                        'includes': includes}
        return {'kinds': kinds, 'testkits': testkits, 'courts': courts, 'categories': categories,
                'placement': dict(placement), 'parts': parts, 'ledger': rewriter.ledger,
                'kit_products': kit_products}

    prepare(model, library_root)
    forced, excluded = set(), set()
    method_edges = {tuple(m.split('::')): [tuple(f.split('::')) for f in fs]
                    for m, fs in overrides.get('compiler_method_edges', {}).items()}
    dev = dev_only_aliases(Path(lib).resolve().parent.parent / 'Cargo.toml')
    for _attempt in range(6):
        first = placed(None)
        called = called_methods(model)
        method_items, method_families = method_seeds(model, called)
        referenced = set(first['ledger'].hooks) | set(first['ledger'].widen_items) | method_items
        # Staying support modules compile only when the moved code calls into them.
        classes = classify(model, all_gated_seeds(model) | referenced, staying_support(model) | method_families,
                           referenced, dev, method_edges)
        readers = extracted_readers(model, classes) | (classes.family_reads - set(model.moving))
        new = {'::'.join(f) for f in readers} - forced
        anchored = {'::'.join(f) for f in classes.anchored} - excluded
        if not new and not anchored:
            break
        forced |= new
        excluded |= anchored
        model = MoveModel(lib, repository, baseline, admissions, sorted(forced), sorted(excluded))
        prepare(model, library_root)
        foreign = {n for n, r in model.records.items() if not r.node.source.resolve().is_relative_to(library_root)}
    for module, items in sorted(classes.extracted.items()):
        add_extracted(model, module, items)
    first = placed(None)
    measured = {key: row['text'].count('\n') + 4 + sum(t.count('\n') + 1 for _l, _t, t in row['includes'])
                for key, row in first['parts'].items()}
    result = placed(measured)
    result.update({'model': model, 'foreign': foreign, 'renames': renames, 'hooks': classes,
                   'called': called, 'forced': sorted(forced), 'referenced': referenced,
                   'anchored': sorted(excluded), 'dev_aliases': sorted(dev)})
    return result


def prepare(model, library_root):
    """Mark foreign-mounted scopes and the library impls that must stay in the library."""
    from src_door_traits import door_traits
    from src_kept_impls import mark_kept_impls
    for record in model.records.values():
        record.foreign = not record.node.source.resolve().is_relative_to(library_root)
    mark_kept_impls(model)
    door_traits(model)


def included_record(record, target):
    """An included file rewritten in the context of the module that includes it."""
    from src_families import file_text
    from src_move_model import ExtractedNode, ModuleRecord, items_of
    text = file_text(target)
    node = ExtractedNode(record.node, text)
    node.source = target
    return ModuleRecord(record.names, record.family, node, items_of(text))


def dev_only_aliases(manifest_path):
    """Crates only the library's own tests may name: dev-dependencies that are not dependencies."""
    import tomllib
    manifest = tomllib.loads(manifest_path.read_text(encoding='utf-8'))
    normal = set(manifest.get('dependencies', {}))
    return {name.replace('-', '_') for name in manifest.get('dev-dependencies', {}) if name not in normal}


def called_methods(model):
    """Method names moved code calls with `.name`."""
    called = set()
    for record in model.records.values():
        stream = tokens(record.node.own_text())
        called |= {stream[i + 1].value for i in range(len(stream) - 1)
                   if stream[i].value == '.' and stream[i + 1].kind == 'ident'}
    return called


def extracted_readers(model, classes):
    """Staying families whose resolved references reach moved, extracted or orphaned items.

    Resolution, not identifier matching: a common name (`map`) must not drag families along.
    """
    from src_families import ExpandedView
    from src_paths import references
    removed = {(module, name) for group in (classes.extracted, classes.orphaned)
               for module, items in group.items() for name, _spans in items}

    def moved_away(origin):
        record = model.records.get(origin[0])
        return origin in removed or (record is not None and not record.extracted
                                     and origin[1] not in record.kept_names)
    moving = set(model.moving)
    result = set()
    for family, record in model.families.items():
        if family in moving:
            continue
        for node in record.node.walk():
            for reference in references(ExpandedView(node), model.index):
                if reference.item is None:
                    continue
                origin, _gated = model.everything.origin(reference.module, reference.item)
                if moved_away(origin or (reference.module, reference.item)):
                    result.add(family)
    return result


CATEGORY_WORDS = ('fast', 'exhaustive', 'scale', 'gpu', 'process', 'oracle', 'cost', 'determinism',
                  'compile_fail', 'serial')


def renamed(names, renames):
    """Apply reviewed module renames to a path (a renamed module renames its descendants)."""
    result = list(names)
    for module, segment in renames.items():
        target = tuple(module.split('::'))
        if tuple(names[:len(target)]) == target:
            result[len(target) - 1] = segment
    return tuple(result)


def hoisted(names):
    """Interim category segments leave the court path: the category is its top-level segment."""
    return tuple(s for s in names if s not in CATEGORY_WORDS)
