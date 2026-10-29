"""Orchestrate inventory, frontier planning, source ownership and reviewable effects."""
from collections import Counter
import json
from pathlib import Path
import tomllib

from cargo_inventory import aliases, dependency_rows, dev_drops, inventory, metadata
from court_partition import CEILING, area_units, classification, group_courts
from court_roots import area_names, origin_name, render, target_area
from fixer_work import collect
import fixture_manifests
from frontier_adapter import frontiers, surface
from manifest_edit import crate_manifest, product_manifest, workspace_manifest
from module_tree import Node, source_files, tree_text
from plan_effects import Effects, relative
from process_effects import helpers
from rust_source import crate_uses
from rust_paths import ModulePaths
from selector_edit import selectors
from source_rewrite import rewritten_file
from store_partitions import partitions, scheduled_areas
from support_ownership import allocate, relevant_support, footprint
from testkit_graph import prepare


def build(manifest, repository, data=None, ceiling=CEILING, module_ceiling=None):
    repository, manifest = repository.resolve(), manifest.resolve()
    data = data or metadata(manifest)
    module_ceiling = module_ceiling or ceiling
    workspace = Path(data['workspace_root']).resolve()
    if workspace != manifest.parent:
        raise ValueError('manifest is not a workspace root')
    stem = 'worth-store' if workspace.name == 'worth-store' else 'worth'
    owners = inventory(data, repository)
    compilation_sources = {node.source for owner in owners for target in owner['targets'] for node in target['tree'].walk()}
    roster, mapping = partitions(repository) if stem == 'worth-store' else ({}, [])
    scheduled = scheduled_areas(repository) if stem == 'worth-store' else []
    units, targets, selections = [], [], {}
    nodes = {n['id']: n for n in data['resolve']['nodes']}
    for owner in owners:
        package = owner['cargo']
        for index, target in enumerate(owner['targets']):
            identity = package['name'] + '::' + target['cargo']['name']
            full_key, refined_key = identity + ':available', identity + ':refined'
            edges = nodes[package['id']]['deps']
            candidate_ids = [e['pkg'] for e in edges if any(k['kind'] in (None, 'dev') for k in e['dep_kinds'])]
            used = crate_uses(tree_text(target['tree']), aliases(package))
            selected = [e['pkg'] for e in edges if e['name'] in used]
            selections[full_key] = set(candidate_ids) | {package['id']}
            selections[refined_key] = set(selected) | {package['id']}
            targets.append({'package': package['name'], 'name': target['cargo']['name'],
                            'source': relative(Path(target['cargo']['src_path']), repository),
                            'available_key': full_key, 'refined_key': refined_key,
                            'named_crates': sorted(used), 'required_features': target['cargo'].get('required-features', [])})
            scheduled_for_target = [r for r in scheduled if r['package'] == package['name'] and r['target'] == target['cargo']['name']]
            population, _support = area_units(target['tree'], module_ceiling, [r['module'] for r in scheduled_for_target])
            process_helpers = helpers(target['tree'])
            for unit in population:
                unit.update({'owner': owner, 'target_index': index, 'target': identity})
                unit['support'] = relevant_support(unit)
                unit_text = tree_text(unit['node']) + '\n' + '\n'.join(a.own_text() for a in unit['ancestors'])
                unit_text += '\n' + '\n'.join(tree_text(s) for s in unit['support'])
                unit['used'] = crate_uses(unit_text, aliases(package))
                unit['key'] = identity + '::' + '::'.join(unit['node'].names)
                selections[unit['key']] = {package['id']} | {e['pkg'] for e in edges if e['name'] in unit['used']}
                unit['category'], unit['evidence'], unit['questions'] = classification(
                    unit['node'], roster.get((package['name'], target['cargo']['name'])), process_helpers)
                for scheduled_row in scheduled_for_target:
                    if unit['node'].names == scheduled_row['module']:
                        unit['category'] = 'cost'
                        unit['evidence'].append('scheduled ignored module: ' + scheduled_row['path'].relative_to(repository).as_posix())
                names = area_names(unit['node'].names)
                unit['owner_area'] = '/'.join((origin_name(owner),) + target_area(unit) + (names or (target['cargo']['name'],)))
                unit['dependencies'] = dependency_rows(package, unit['used'], own=True)
                unit['dependencies'][0]['features'] = sorted(target['cargo'].get('required-features', []))
                unit['footprint'] = footprint(unit, compilation_sources)
                units.append(unit)
    computed = frontiers(data, selections)
    for target in targets:
        target['available_frontier'] = computed[target.pop('available_key')]
        target['refined_frontier'] = computed[target.pop('refined_key')]
    for unit in units:
        unit['frontier'] = computed[unit['key']]
    courts, splits = group_courts(units, stem, surface, ceiling)
    assignments, kits = allocate(units, owners, stem, workspace, repository)
    oversized = {c['name'] for c in courts if sum(
        len(p.read_bytes().splitlines()) for p, a in assignments.items()
        if a['owner'] == c['name'] and p.suffix == '.rs') > ceiling}
    if oversized:
        for unit in units:
            if unit['court'] in oversized:
                unit['footprint'].update(n.source for support in unit['support'] for n in support.walk())
        courts, splits = group_courts(units, stem, surface, ceiling)
        assignments, kits = allocate(units, owners, stem, workspace, repository)
    kit_aliases = prepare(kits, assignments, owners, ceiling)
    module_paths = ModulePaths(units, assignments, kit_aliases)
    effects = Effects(repository)
    rewrites = materialize_sources(assignments, owners, kits, kit_aliases, effects, module_paths)
    workspace_value = tomllib.loads(manifest.read_text(encoding='utf-8'))
    rows_by_court = {c['name']: [d for u in c['units'] for d in u['dependencies']] for c in courts}
    kit_rows = kit_dependencies(kits, assignments, owners, repository)
    for name, kit in sorted(kits.items()):
        destination = workspace / 'crates' / name
        for reader in sorted(kit['readers']):
            rows_by_court[reader].append(local_dependency(name, destination))
        roots = ''.join('#[path = ' + json.dumps(str(assignments[n.source]['destination'].relative_to(destination / 'src')).replace('\\', '/')) + ']\npub mod ' + alias + ';\n'
                        for n, alias in kit['roots'])
        effects.write(destination / 'src/lib.rs', roots)
        effects.write(destination / 'Cargo.toml', crate_manifest(name, kit_rows[name], destination, workspace_value, True))
    for court in courts:
        destination = workspace / 'crates' / court['name']
        def unit_text(unit, target):
            node = unit['node']
            if unit['own_only']:
                node = Node(node.source, node.names, node.own_text())
            text, records = rewritten_file(node, target, unit['owner'], assignments, kit_aliases)
            if unit['own_only']:
                for record in records:
                    record['scope'] = '::'.join(node.names)
                    record.pop('offset', None)
            rewrites.extend(records)
            text, records = module_paths.rewrite(text, unit['owner'], unit['node'].source)
            rewrites.extend(records)
            return text
        def context_text(text, unit, source):
            text, records = module_paths.rewrite(text, unit['owner'], source)
            rewrites.extend(records)
            return text
        render(court, workspace, assignments, kit_aliases, effects, unit_text, context_text)
        effects.write(destination / 'Cargo.toml', crate_manifest(court['name'], rows_by_court[court['name']], destination, workspace_value))
    drops, disabled_libraries = [], []
    for owner in owners:
        package_manifest = Path(owner['cargo']['manifest_path'])
        removed = dev_drops(owner)
        drops.append({'package': owner['cargo']['name'], 'dependencies': removed,
                      'evidence': 'No named crate reference anywhere in src, examples or benches (conservative unit-test superset).'})
        has_unit = any('#[test]' in p.read_text(encoding='utf-8').replace(' ', '') or '#[cfg(test)]' in p.read_text(encoding='utf-8').replace(' ', '')
                       for p in (owner['root'] / 'src').rglob('*.rs'))
        library = any('lib' in t['kind'] or 'rlib' in t['kind'] or 'proc-macro' in t['kind'] for t in owner['cargo']['targets'])
        if library and not has_unit:
            disabled_libraries.append(owner['cargo']['name'])
        updated = product_manifest(package_manifest.read_text(encoding='utf-8'), removed, library and not has_unit)
        effects.write(package_manifest, updated)
    members = [str((workspace / 'crates' / name).relative_to(workspace)).replace('\\', '/')
               for name in sorted([c['name'] for c in courts] + list(kits))]
    effects.write(manifest, workspace_manifest(manifest.read_text(encoding='utf-8'), members))
    register(effects, repository, workspace, stem, splits)
    selector_changes = selectors(effects, repository, workspace, courts, roster)
    work = collect(courts, assignments, repository)
    program = effects.program()
    court_rows, kit_report = reports(courts, kits, assignments, program, workspace, repository, rows_by_court, kit_rows)
    blockers = []
    for row in court_rows + kit_report:
        if row['rust_lines'] > ceiling:
            blockers.append({'kind': 'source-ceiling', 'owner': row['name'], 'lines': row['rust_lines'],
                             'reason': 'An indivisible module/support ownership exceeds the ceiling.'})
    if blockers and module_ceiling > 400:
        return build(manifest, repository, data, ceiling, max(400, module_ceiling // 2))
    rewrites = sorted({json.dumps({**r, 'source': relative(Path(r['source']), repository)}, sort_keys=True)
                       for r in rewrites})
    counts = Counter(row['kind'] for rows in work.values() for row in rows)
    executables_before = sum(t['test'] for o in owners for t in o['cargo']['targets'])
    return {'version': 1, 'workspace_manifest': relative(manifest, repository), 'stem': stem,
            'authority': ['docs/coding-guidelines/iteration_laws.md', 'test_convention_v3.md §§2–5'],
            'targets': targets, 'courts': court_rows, 'testkits': kit_report, 'splits': splits,
            'category_mapping': mapping or [{'from': k, 'to': v or 'ordinary'} for k, v in sorted(__import__('court_partition').INTERIM.items())],
            'path_rewrites': [json.loads(r) for r in rewrites], 'dev_dependencies_to_drop': drops,
            'selector_changes': selector_changes, 'fixer_work_by_court': work, 'blockers': blockers,
            'report_only': [{'package': o['cargo']['name'], 'targets': [
                {'name': t['name'], 'kind': t['kind'], 'source': relative(Path(t['src_path']), repository)}
                for t in o['cargo']['targets'] if any(k in ('bench', 'example', 'bin') for k in t['kind'])]}
                for o in owners],
            'disabled_library_harnesses': sorted(disabled_libraries),
            'totals': {'courts': len(courts), 'testkits': len(kits), 'executables_before': executables_before,
                       'executables_after': executables_before - len(targets) + len(courts) - len(disabled_libraries),
                       'integration_executables_before': len(targets), 'integration_executables_after': len(courts),
                       'lines_moved': sum(len(p.read_bytes().splitlines()) for p in assignments if p.suffix == '.rs'),
                       'files_moved': len(assignments), 'work_items': sum(counts.values()),
                       'work_items_by_kind': dict(sorted(counts.items()))}, 'effects': program}


def local_dependency(name, path):
    return {'alias': name, 'package': name, 'path': str(path), 'features': [],
            'default_features': True, 'target': None, 'requirement': None}


def materialize_sources(assignments, owners, kits, aliases, effects, module_paths):
    all_nodes = {}
    for owner in owners:
        for target in owner['targets']:
            for node in target['tree'].walk():
                if not node.offset:
                    all_nodes.setdefault(node.source, (node, owner))
    rewrites = []
    for source, assignment in sorted(assignments.items()):
        effects.move(source, assignment['destination'])
        if source.name == 'Cargo.toml':
            text, records = fixture_manifests.rewrite(source, assignment['destination'], assignments)
            rewrites.extend(records)
            if records:
                effects.write(assignment['destination'], text, source.read_bytes())
        if source not in all_nodes:
            continue
        node, owner = all_nodes[source]
        kit = assignment['owner'] if assignment['owner'] in kits else None
        text, records = rewritten_file(node, assignment['destination'], owner, assignments, aliases, kit)
        rewrites.extend(records)
        text, path_records = module_paths.rewrite(text, owner, source, kit)
        rewrites.extend(path_records)
        if records or path_records:
            effects.write(assignment['destination'], text, source.read_bytes())
    return rewrites


def kit_dependencies(kits, assignments, owners, repository):
    result = {}
    by_name = {o['cargo']['name']: o for o in owners}
    for name, kit in sorted(kits.items()):
        rows = []
        for node in kit['nodes']:
            owner = by_name[assignments[node.source]['package']]
            used = crate_uses(node.text, aliases(owner['cargo']))
            rows.extend(dependency_rows(owner['cargo'], used, own=True))
            for child in node.children:
                assignment = assignments.get(child.source)
                if assignment and assignment['owner'] in kits and assignment['owner'] != name:
                    rows.append(local_dependency(assignment['owner'], next(p for p in assignment['destination'].parents if p.parent.name == 'crates')))
        result[name] = rows
    return result


def register(effects, root, workspace, stem, splits):
    path = root / 'tools/boundary-check/config/test-workspaces.txt'
    text = path.read_text(encoding='utf-8') if path.exists() else ''
    workspace_name = relative(workspace, root) or '.'
    row = workspace_name + ' | ' + stem
    if row not in text.splitlines():
        effects.write(path, text.rstrip() + '\n' + row + '\n')
    if splits:
        path = root / 'tools/boundary-check/config/court-frontier-splits.toml'
        text = path.read_text(encoding='utf-8') if path.exists() else ''
        for split in splits:
            text += '\n[[splits]]\n' + ''.join(key + ' = ' + json.dumps(value) + '\n' for key, value in split.items())
        effects.write(path, text)


def reports(courts, kits, assignments, program, workspace, root, court_deps, kit_deps):
    writes = {row['path']: row['content'] for row in program['writes']}
    def row(name, deps):
        prefix = relative(workspace / 'crates' / name, root) + '/'
        files = {relative(a['destination'], root): p.read_text(encoding='utf-8')
                 for p, a in assignments.items() if a['owner'] == name and p.suffix == '.rs'}
        files.update({p: text for p, text in writes.items() if p.startswith(prefix) and p.endswith('.rs')})
        return {'name': name, 'rust_lines': sum(len(text.splitlines()) for text in files.values()),
                'dependencies': sorted({d['alias'] for d in deps}), 'files': sorted(files)}
    court_rows = []
    for court in courts:
        report = row(court['name'], court_deps[court['name']])
        report.update({'frontier': court['frontier'], 'categories': sorted({u['category'] or 'ordinary' for u in court['units']}),
                       'units': [{'source': relative(u['node'].source, root), 'old_module': '::'.join(u['node'].names),
                                  'owner_area': u['owner_area'], 'category': u['category'] or 'ordinary',
                                  'evidence': u['evidence'], 'named_crates': sorted(u['used'])} for u in court['units']]})
        court_rows.append(report)
    return court_rows, [{**row(name, kit_deps[name]), 'readers': sorted(kit['readers'])} for name, kit in sorted(kits.items())]
