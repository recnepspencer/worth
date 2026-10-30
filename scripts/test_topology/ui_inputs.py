"""Refine compile-boundary inputs from the actual programs in referenced UI trees."""
from cargo_inventory import aliases, dependency_rows
from module_tree import Node, literal_references
from plan_effects import relative
from rust_source import crate_uses
from support_ownership import referenced_files
from support_readers import compilation_inputs


def refine(model, units, targets, selections, nodes, repository):
    inputs = compilation_inputs(model, units)
    by_target = {}
    for unit in units:
        owner, used = unit['owner'], unit['used']
        package = owner['cargo']
        population = inputs[unit['key']]
        unit['support_inputs'] = population
        unit['scope'] = model.record(unit)
        for helper in population:
            used.update(crate_uses(helper['helper'], aliases(package)))
        programs = set()
        if unit['category'] == 'compile_fail':
            record = model.record(unit)
            fragments = [(record['node'], record['test'])] + [(r['node'], r['helper']) for r in population]
            for original, text in fragments:
                node = Node(original.source, original.names, text)
                for reference in literal_references(node, owner['root']):
                    programs.update(p for p in referenced_files(reference['path'])
                                    if p.suffix == '.rs' and p.is_relative_to(owner['root'] / 'tests/ui'))
        evidence = []
        for source in sorted(programs):
            names = crate_uses(source.read_text(encoding='utf-8'), aliases(package))
            used.update(names)
            evidence.append({'kind': 'ui-program', 'source': relative(source, repository),
                             'named_crates': sorted(names)})
        unit['source_evidence'] = evidence
        unit['footprint'].update(programs)
        unit['dependencies'] = dependency_rows(package, used, own=True)
        unit['dependencies'][0]['features'] = list(unit['required_features'])
        edges = nodes[package['id']]['deps']
        selections[unit['key']] = {package['id']} | {e['pkg'] for e in edges if e['name'] in used}
        by_target.setdefault(unit['target'], set()).update(used)
    for target in targets:
        identity = target['package'] + '::' + target['name']
        names = by_target.get(identity, set()) | set(target['named_crates'])
        target['named_crates'] = sorted(names)
        package = next(u['owner']['cargo'] for u in units if u['target'] == identity)
        selections[target['refined_key']] = {package['id']} | {
            e['pkg'] for e in nodes[package['id']]['deps'] if e['name'] in names}
