"""Assign original test files to their new case/support source and emit path-only bodies."""
from collections import defaultdict

from court_roots import case_path, insert_context, module_path
from module_tree import Node, test_names
from rust_source import tokens
from source_rewrite import rewritten_file
from cfg_context import rebind


def assign_sources(model, units, assignments, support_roots):
    cases, helpers, all_nodes = defaultdict(list), defaultdict(list), {}
    for record in model.records.values():
        all_nodes.setdefault(record['node'].source, []).append(record)
    for unit in units:
        record = model.record(unit)
        unit['case_path'] = case_path(unit, model.workspace)
        record['test_owner'], record['test_path'], record['test_module'] = unit['court'], unit['case_path'], module_path(unit)
        cases[unit['node'].source].append(unit)
    for population in support_roots.values():
        for record in population:
            helpers[record['node'].source].append(record)
    retired = []
    for source, records in sorted(all_nodes.items()):
        if source not in assignments:
            continue
        row = assignments[source]
        if cases[source]:
            unit = min(cases[source], key=lambda u: (-len(model.record(u)['test']), u['owner_area'], u['court']))
            row.update(destination=unit['case_path'], owner=unit['court'], kind='test-source')
        elif helpers[source]:
            record = min(helpers[source], key=lambda r: (len(r['node'].names), r['route']))
            row.update(destination=record['declarations'], owner=record['support_owner'], kind='support-source')
        else:
            assert not test_names(source.read_text(encoding='utf-8')), source
            retired.append(source)
    for source in retired:
        assignments.pop(source)
    return retired


def case_text(model, unit, destination, assignments, rewrites):
    record = model.record(unit)
    text, receipts = model.bindings.rewrite(record['test'], record, unit['court'])
    rewrites.extend(receipts)
    text, receipts = model.executables.rewrite(text, record['node'].source)
    rewrites.extend(receipts)
    text, receipts = rebind(text, record['features'])
    rewrites.extend({**r, 'source': str(record['node'].source), 'scope': '::'.join(record['node'].names)} for r in receipts)
    node = Node(record['node'].source, record['node'].names, text)
    text, receipts = rewritten_file(node, destination, record['owner'], assignments)
    rewrites.extend(receipts)
    return insert_context(text, model.import_context(unit))
