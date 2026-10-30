"""Category evidence and deterministic area partitioning, independent of effects."""
from collections import defaultdict
import re
from court_packing import assign_file_weights, whole_areas, first_fit_decreasing, area_suffix

from module_tree import test_names, tree_text
from rust_source import code
from process_effects import calls, direct_effect
from test_macros import opaque_items

CEILING = 15_000
INTERIM = {'fast': '', 'exhaustive': 'oracle', 'scale': 'cost',
           'serial': 'process::serial', 'compile_fail': 'compile_fail'}
CATEGORIES = ('process', 'oracle', 'cost', 'determinism', 'compile_fail')


def measured_lines(population):
    files = set().union(*(u['footprint'] for u in population))
    # Every selected module needs a two-line mount plus a generated ancestor scope.
    wrappers = {u['owner_area'].split('/')[0] + '/' + '/'.join(u['node'].names[:i])
                for u in population for i in range(len(u['node'].names) + 1)}
    return sum(len(p.read_text(encoding='utf-8').splitlines()) for p in files) + 2 * len(wrappers)


def classification(node, store_partition=None, process_helpers=()):
    text = tree_text(node)
    value = code(text)
    old = next((n for n in node.names if n in INTERIM or n in CATEGORIES), None)
    evidence, questions = [], []
    category = INTERIM.get(old, old or '')
    if old:
        evidence.append('module segment: ' + old)
    if store_partition:
        mapping = {'scenario': '', 'ui': 'compile_fail', 'formal': 'oracle'}
        category = mapping.get(store_partition, category)
        evidence.append('Store runner partition: ' + store_partition)
    process = bool(direct_effect(value) or calls(value, process_helpers)
                   or re.search(r'\b(?:reexecute_isolated_test|run_isolated_test|spawn_child|run_child|spawn_process)\s*\(', value))
    if process and category != 'compile_fail':
        category = 'process::serial' if old == 'serial' else 'process'
        evidence.append('process command or isolation helper in module tree')
    elif old == 'exhaustive':
        model = bool(re.search(r'\b(?:let|use|mod)\s+(?:mut\s+)?(?:reference\w*|\w*reference|oracle\w*)\b'
                               r'|\b(?:Reference\w*|\w*sweep\w*)\s*(?::\s*:|\()', value))
        ignored = bool(re.search(r'#\s*\[\s*ignore\b', text))
        slow = bool(re.search(r'\b(?:Instant|elapsed|sleep|Duration)\b', value))
        if not model and ignored and slow:
            category = 'cost'
            evidence.append('ignored slow test; no reference model or sweep signal')
        elif not model:
            questions.append('exhaustive segment has no syntactic reference-model or sweep evidence; confirm oracle/cost')
    if not old and not store_partition:
        questions.append('no interim category segment or Store runner partition; confirm ordinary/category')
    if 'CARGO_MANIFEST_DIR' in text and re.search(r'\b(?:File|OpenOptions|create_dir_all|sync_all)\b', value):
        questions.append('filesystem consumer: establish whether the claim needs real durability (process category)')
    return category, evidence, questions


def has_tests(node):
    return any(test_names(n.own_text()) or opaque_items(n.own_text()) for n in node.walk())


def area_units(tree, ceiling=CEILING, forced_areas=()):
    """Descend only on module boundaries. A source file's own tests stay together."""
    result, support = [], []

    def descend(node, ancestors):
        if not has_tests(node):
            support.append((node, ancestors))
            return
        category = node.names and node.names[-1] in (*INTERIM, *CATEGORIES)
        lines = sum(len(n.own_text().splitlines()) for n in node.walk())
        forced = any(tuple(area[:len(node.names)]) == node.names and tuple(area) != node.names for area in forced_areas)
        if node.children:
            if test_names(node.own_text()) or opaque_items(node.own_text()):
                # Root tests cannot be copied to their children. Retain this unit,
                # with its child test modules delegated to independent units.
                result.append({'node': node, 'ancestors': ancestors, 'own_only': True})
            for child in node.children:
                descend(child, ancestors + [node])
        else:
            result.append({'node': node, 'ancestors': ancestors, 'own_only': False})

    descend(tree, [])
    return result, support


def group_courts(units, stem, naming, ceiling=CEILING):
    assign_file_weights(units)
    grouped = defaultdict(list)
    for unit in units:
        grouped[(tuple(unit['frontier']), tuple(unit.get('required_features', ())), unit.get('custom_harness', ''))].append(unit)
    courts, splits = [], []
    signatures = defaultdict(set)
    for frontier, features, custom in grouped:
        signatures[frontier].add((features, custom))
    for (frontier, features, custom), population in sorted(grouped.items()):
        surface = naming(stem, frontier)
        base = stem + '-court-' + surface
        total = sum(u['packing_lines'] for u in population)
        areas = whole_areas(population, ceiling)
        pieces = first_fit_decreasing(areas, ceiling)
        for piece in pieces:
            area = area_suffix(piece['areas'][0]['path'], surface, [naming(stem, [f]) for f in frontier]) if len(pieces) > 1 or len(signatures[frontier]) > 1 else ''
            members = [u for a in piece['areas'] for u in a['units']]
            name = base + ('-' + area if area else '')
            court = {'name': name, 'frontier': list(frontier), 'units': members, 'area': area,
                     'packing_lines': piece['lines'], 'packing_areas': [a['path'] for a in piece['areas']],
                     'required_features': list(features), 'custom_harness': custom}
            courts.append(court)
            for member in members:
                member['court'] = name
            if area:
                splits.append({'court': name, 'frontier': list(frontier), 'area': area,
                               'owners': sorted({u['owner_area'] for u in members}), 'delegates': []})
    for split in splits:
        split['owners'] = [owner for owner in split['owners'] if not any(
            owner != parent and owner.startswith(parent + '/') for parent in split['owners'])]
        split['delegates'] = sorted({child[len(owner) + 1:] for owner in split['owners']
                                     for other in splits if other['court'] != split['court']
                                     and other['frontier'] == split['frontier']
                                     for child in other['owners'] if child.startswith(owner + '/')})
    return courts, splits
