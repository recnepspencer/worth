"""Orchestrate inventory, frontier planning, source ownership and reviewable effects."""
from binary_constraints import report as binary_constraints
from executable_paths import ExecutablePaths
from collections import Counter
import json
from pathlib import Path
import tomllib

from cargo_inventory import aliases, dependency_rows, dev_drops, inventory, metadata
from court_partition import CEILING, area_units, classification, group_courts
from court_roots import area_names, origin_name, render, target_area, disambiguate
from fixer_work import collect
import fixture_manifests
from frontier_adapter import frontiers, surface
from manifest_edit import crate_manifest, product_manifest, workspace_manifest
from module_tree import Node, source_files, tree_text
from plan_effects import Effects, relative
from process_effects import helpers
from registered_paths import relocate
from source_catalogues import relocate as relocate_catalogues
from rust_source import crate_uses
from support_model import SupportModel
from support_emit import layout_support, canonical_roots, binary_support_imports
from case_emit import assign_sources, case_text
from court_plan_dependencies import local_dependency
from selector_edit import selectors
from source_rewrite import rewritten_file
from store_partitions import partitions, scheduled_areas
from support_ownership import allocate, relevant_support, footprint


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
        owner['effective_features'] = nodes[package['id']]['features']
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
                unit.update({'owner': owner, 'target_index': index, 'target': identity,
                             'required_features': tuple(target['cargo'].get('required-features', [])),
                             'custom_harness': target['cargo']['name'] if any(t.get('harness') is False and t.get('name') == target['cargo']['name'] for t in owner['manifest'].get('test', [])) else ''})
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
    disambiguate(courts)
    model = SupportModel(owners, units, courts, stem, workspace)
    support_roots = layout_support(model, kits)
    model.executables = ExecutablePaths(stem, workspace, owners)
    for record in model.canonical:
        if record.get('active') and record['support_owner']:
            model.executables.scan(record['helper'], record['support_owner'])
    for unit in units:
        model.executables.scan(model.record(unit)['test'], unit['court'])
    retired = assign_sources(model, units, assignments, support_roots)
    active_kits = {a['owner'] for a in assignments.values() if '-testkit' in a['owner']} | {name for name in support_roots if '-testkit' in name}
    kits = {name: row for name, row in kits.items() if name in active_kits}
    effects = Effects(repository)
    rewrites = []
    for source, assignment in sorted(assignments.items()):
        effects.move(source, assignment['destination'])
        if source.name == 'Cargo.toml':
            text, receipts = fixture_manifests.rewrite(source, assignment['destination'], assignments)
            rewrites.extend(receipts)
            if receipts:
                effects.write(assignment['destination'], text, source.read_bytes())
    for source in retired:
        effects.retire_scaffold(source)
    rewrites.extend(relocate_catalogues(owners, assignments, effects))
    workspace_value = tomllib.loads(manifest.read_text(encoding='utf-8'))
    support_rows = canonical_roots(model, support_roots, assignments, effects, rewrites)
    rows_by_court = {c['name']: [d for u in c['units'] for d in u['dependencies']] + support_rows[c['name']] for c in courts}
    kit_rows = {name: support_rows[name] for name in kits}
    if model.executables.used:
        executable = model.executables
        for reader in executable.readers:
            (kit_rows if reader in kits else rows_by_court)[reader].append(local_dependency(executable.name, executable.directory))
        executable.emit(effects)
        kits[executable.name] = {'name': executable.name, 'readers': {u['court'] for u in units}, 'files': [], 'nodes': []}
        kit_rows[executable.name] = []
    for name, kit in sorted(kits.items()):
        destination = workspace / 'crates' / name
        for reader in sorted(kit['readers']):
            rows_by_court[reader].append(local_dependency(name, destination))
        if name not in support_roots and name != model.executables.name:
            effects.write(destination / 'src/lib.rs', '// Fixture assets are owned by this testkit.\n')
        effects.write(destination / 'Cargo.toml', crate_manifest(name, kit_rows[name], destination, workspace_value, True))
    for court in courts:
        destination = workspace / 'crates' / court['name']
        def unit_text(unit, target):
            return case_text(model, unit, target, assignments, rewrites)
        render(court, workspace, effects, model, unit_text,
               binary_support_imports(court['name'], support_roots.get(court['name'], [])))
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
    registry_rewrites, unresolved_registrations = relocate(effects, repository, assignments)
    rewrites.extend(registry_rewrites)
    selector_changes = selectors(effects, repository, workspace, courts, roster, model.executables.prerequisites())
    work = collect(courts, assignments, repository)
    for registration in unresolved_registrations:
        for source in registration['sources']:
            assignment = assignments[repository / source]
            for court in assignment['readers']:
                work[court].append({'kind': 'registration-ruling', 'source': source,
                                    'reason': registration['registry'] + ' registers directory prefix ' + registration['prefix']
                                    + '; its files now have separate owners. Choose exact callers without widening authority.',
                                    'symbols': []})
    program = effects.program()
    court_rows, kit_report = reports(courts, kits, assignments, program, workspace, repository, rows_by_court, kit_rows)
    blockers = []
    for row in court_rows + kit_report:
        if row['rust_lines'] > ceiling:
            blockers.append({'kind': 'source-ceiling', 'owner': row['name'], 'lines': row['rust_lines'],
                             'reason': 'An indivisible module/support ownership exceeds the ceiling.'})
    rewrites = sorted({json.dumps({**r, 'source': relative(Path(r['source']), repository)}, sort_keys=True)
                       for r in rewrites})
    counts = Counter(row['kind'] for rows in work.values() for row in rows)
    executables_before = sum(t['test'] for o in owners for t in o['cargo']['targets'])
    return {'version': 1, 'workspace_manifest': relative(manifest, repository), 'stem': stem,
            'authority': ['docs/coding-guidelines/iteration_laws.md', 'test_convention_v3.md §§2–5'],
            'compiled_source_map': sorted([{'destination': relative(u['case_path'], repository), 'source': relative(u['node'].source, repository), 'old_module': '::'.join(u['node'].names)} for u in units] + [{'destination': relative(r['declarations'], repository), 'source': relative(r['node'].source, repository), 'old_module': '::'.join(r['node'].names)} for r in model.canonical if 'declarations' in r], key=lambda r: r['destination']),
            'targets': targets, 'binary_constraints': binary_constraints(targets, courts), 'courts': court_rows, 'testkits': kit_report, 'splits': splits,
            'category_mapping': mapping or [{'from': k, 'to': v or 'ordinary'} for k, v in sorted(__import__('court_partition').INTERIM.items())],
            'path_rewrites': [json.loads(r) for r in rewrites], 'dev_dependencies_to_drop': drops,
            'selector_changes': selector_changes, 'executable_prerequisites': model.executables.prerequisites(), 'fixer_work_by_court': work, 'blockers': blockers,
            'report_only': [{'package': o['cargo']['name'], 'targets': [
                {'name': t['name'], 'kind': t['kind'], 'source': relative(Path(t['src_path']), repository)}
                for t in o['cargo']['targets'] if any(k in ('bench', 'example', 'bin') for k in t['kind'])]}
                for o in owners],
            'disabled_library_harnesses': sorted(disabled_libraries),
            'totals': {'courts': len(courts), 'testkits': len(kits), 'executables_before': executables_before,
                       'executables_after': executables_before - len(targets) + len(courts) - len(disabled_libraries),
                       'integration_executables_before': len(targets), 'integration_executables_after': len(courts),
                       'lines_moved': sum(len(p.read_bytes().splitlines()) for p in assignments if p.suffix == '.rs'),
                       'retired_scaffold_lines': sum(len(p.read_bytes().splitlines()) for p in retired if p.suffix == '.rs'),
                       'files_moved': len(assignments), 'work_items': sum(counts.values()),
                       'work_items_by_kind': dict(sorted(counts.items()))}, 'effects': program}


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
                       'packing_lines': court['packing_lines'], 'packing_areas': [list(p) for p in court['packing_areas']],
                       'required_features': court['required_features'], 'custom_harness': court['custom_harness'],
                       'units': [{'source': relative(u['node'].source, root), 'new_source': relative(u['case_path'], root), 'old_module': '::'.join(u['node'].names),
                                  'owner_area': u['owner_area'], 'category': u['category'] or 'ordinary',
                                  'evidence': u['evidence'], 'named_crates': sorted(u['used'])} for u in court['units']]})
        court_rows.append(report)
    return court_rows, [{**row(name, kit_deps[name]), 'readers': sorted(kit['readers'])} for name, kit in sorted(kits.items())]
