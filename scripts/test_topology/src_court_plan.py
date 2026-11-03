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
from src_move_model import MoveModel
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
                text = '' if names in foreign else rewriter.rewrite(model.records[names], part, kinds[names], crate)
                parts[(names, part)] = {'crate': crate, 'path': path, 'text': text, 'foreign': names in foreign}
        return {'kinds': kinds, 'testkits': testkits, 'courts': courts, 'categories': categories,
                'placement': dict(placement), 'parts': parts, 'ledger': rewriter.ledger,
                'kit_products': kit_products}

    first = placed(None)
    measured = {key: row['text'].count('\n') + 4 for key, row in first['parts'].items()}
    result = placed(measured)
    result.update({'model': model, 'foreign': foreign, 'renames': renames})
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
