"""One binary and category-first test tree, with support outside category modules."""
from collections import defaultdict
import json
from pathlib import Path

from court_partition import CATEGORIES, INTERIM
from rust_items import items
from source_rewrite import relpath
from cfg_context import rebind


def area_names(names):
    return tuple(n for i, n in enumerate(names) if not (i == 0 and n in (*INTERIM, *CATEGORIES)))


def origin_name(owner):
    name, stem = owner['cargo']['name'], owner.get('workspace_stem', 'worth')
    return (name.rsplit('-', 1)[-1] if name == stem else name.removeprefix(stem + '-')).replace('-', '_')


def target_area(unit):
    target = unit['owner']['targets'][unit['target_index']]['cargo']['name']
    package = unit['owner']['cargo']['name'].replace('-', '_')
    return () if target == package else (target,)


def module_path(unit):
    category = tuple(unit['category'].split('::')) if unit['category'] else ()
    return category + (origin_name(unit['owner']),) + target_area(unit) + area_names(unit['node'].names) + tuple(unit.get('module_context', ()))


def case_path(unit, workspace):
    name = unit['court']
    return workspace / 'crates' / name / 'tests' / name.replace('-', '_') / Path(*module_path(unit)) / 'mod.rs'


def insert_context(text, context):
    parsed = items(text)
    first = next((item.start for item in parsed if item.keyword != 'inner-attribute'), len(text))
    return text[:first] + context + text[first:]


def render(court, workspace, effects, model, unit_text, local_support=''):
    name = court['name']
    binary = workspace / 'crates' / name / 'tests' / (name.replace('-', '_') + '.rs')
    directory = binary.with_suffix('')
    scopes, mounts = defaultdict(list), {}
    for unit in court['units']:
        path = module_path(unit)
        if path in mounts:
            raise ValueError('test module collision: ' + '::'.join(path))
        mounts[path] = unit
    paths = set(scopes) | set(mounts) | {()}
    for path in list(paths):
        paths.update(path[:i] for i in range(len(path)))
    files = {path: directory.joinpath(*path, 'mod.rs') if path else binary for path in paths}
    for path in sorted(paths):
        source = files[path]
        if path in mounts:
            text = unit_text(mounts[path], source)
        else:
            text = ''.join(sorted(set(scopes[path])))
        if not path:
            text += local_support
        for child in sorted(p for p in paths if len(p) == len(path) + 1 and p[:-1] == path):
            unit = mounts.get(child)
            attributes = rebind(model.record(unit)['attributes'], model.record(unit)['features'])[0] if unit else ''
            text += attributes + '#[path = ' + json.dumps(relpath(files[child], source.parent)) + ']\npub(crate) mod ' + child[-1] + ';\n'
        effects.write(source, text)
    return binary


def disambiguate(courts):
    contexts = {'fast': 'ordinary_context', 'exhaustive': 'reference_context', 'scale': 'growth_context', 'serial': 'isolated_context'}
    for court in courts:
        grouped = defaultdict(list)
        for unit in court['units']:
            grouped[module_path(unit)].append(unit)
        for path, population in grouped.items():
            if len(population) > 1:
                for unit in population:
                    first = unit['node'].names[0] if unit['node'].names else 'root'
                    unit['module_context'] = (contexts.get(first, first + '_context'),)
