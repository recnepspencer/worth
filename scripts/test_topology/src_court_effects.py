"""Turn a source court plan into one preflighted effect program plus its review record."""
from collections import Counter, defaultdict
import re

from filesystem_path import filesystem_path
from plan_effects import Effects, relative
from rust_source import crate_uses
from src_court_plan import build
from src_emit_config import (control_callers, library_test_doors, shrink_admissions, shrink_baseline,
                             splits_toml)
from src_emit_crates import (LIBRARY_DIR, crate_sources, library_dependencies, manifest, module_files,
                             referenced_testkits)
from src_emit_production import apply_edits, production_edits, ungate_recorders
from src_multiset import multiset_of_texts

CONFIG = 'tools/boundary-check/config/'
CATEGORY_WORDS = ('fast', 'exhaustive', 'scale', 'gpu', 'process', 'oracle', 'cost', 'determinism',
                  'compile_fail', 'serial')


ALLOCATOR = ('// Allocation-count courts measure through the one binary-wide stats allocator.\n'
             '#[global_allocator]\n'
             'static TEST_ALLOCATOR: &stats_alloc::StatsAlloc<std::alloc::System> = &stats_alloc::INSTRUMENTED_SYSTEM;\n')
GLOBAL_ALLOCATOR = re.compile(r'#\s*\[\s*global_allocator\s*\]\s*static\s+\w+\s*:[^;]*;\n?')


def category_valid(path):
    """boundary-check graph_rules.rs::category_valid, spelled in Python."""
    segments = [s for s in path.removesuffix('.rs').split('/') if s != 'mod']
    for index, segment in enumerate(segments):
        if segment in ('fast', 'exhaustive', 'scale', 'gpu'):
            return False
        if index > 0 and segment in ('process', 'oracle', 'cost', 'determinism', 'compile_fail'):
            return False
        if segment == 'serial' and (index != 1 or segments[0] not in ('process', 'compile_fail')):
            return False
    return True


def strip_allocator(text):
    """A binary-wide allocator belongs to the court root, never to one module."""
    stripped = GLOBAL_ALLOCATOR.sub('', text)
    return stripped, stripped != text


def declaration_attributes(node):
    if node.declaration is None:
        return ''
    kept = [a for a in re.findall(r'#\s*\[(?:[^\[\]]|\[[^\]]*\])*\]', node.declaration.attributes)
            if not re.match(r'#\s*\[\s*(?:path\s*=|cfg\s*\(\s*test\s*\)\s*\])', a)]
    return ''.join(a + '\n' for a in kept)


def effects_for(root, lib, baseline, admissions, workspace, overrides):
    p = build(lib, root, baseline, admissions, workspace, overrides)
    model = p['model']
    effects, blockers = Effects(root), []
    crate_parts = defaultdict(list)
    allocators = []
    for (names, part), row in sorted(p['parts'].items()):
        node = model.records[names].node
        source_file = relative(node.source.resolve(), root)
        text, removed = strip_allocator(row['text'])
        if removed:
            allocators.append('::'.join(names))
        crate_parts[(row['crate'], 'testkit' if part == 'testkit' else 'court')].append(
            {'path': row['path'], 'text': text, 'attributes': declaration_attributes(node), 'names': names,
             'source_file': source_file,
             'foreign_file': source_file if row['foreign'] and names[:-1] not in p['foreign'] else None})
    files, crates = {}, []
    library_table = library_dependencies(root)
    kit_names = sorted(p['testkits'])
    for (crate, kind), parts in sorted(crate_parts.items()):
        measuring = kind == 'court' and any('INSTRUMENTED_SYSTEM' in part['text'] for part in parts)
        sources = crate_sources(crate, kind, parts, '#![recursion_limit = "256"]\n' + (ALLOCATOR if measuring else ''))
        texts = list(sources.values())
        kits = referenced_testkits(texts, kit_names)
        products = set(p['kit_products'].get(crate, set())) | set(p['courts'].get(crate, {}).get('frontier', []))
        sources[f'workspaces/worth-query/crates/{crate}/Cargo.toml'] = manifest(crate, kind, texts, library_table,
                                                                             kits, products)
        files.update(sources)
        lines = sum(t.count('\n') for t in texts)
        crates.append({'name': crate, 'kind': kind, 'lines': lines, 'testkits': sorted(kits),
                       'frontier': p['courts'].get(crate, {}).get('frontier', [])})
        if lines > 15000:
            blockers.append({'crate_over_ceiling': crate, 'lines': lines})
        prefix = f'workspaces/worth-query/crates/{crate}/tests/{crate.replace("-", "_")}/'
        for path in sources:
            if kind == 'court' and path.startswith(prefix) and not category_valid(path[len(prefix):]):
                blockers.append({'category_segment_not_top_level': path})
    seen_paths = {}
    for (names, part), row in sorted(p['parts'].items()):
        key = (row['crate'], part, row['path'])
        if key in seen_paths:
            blockers.append({'module_path_collision': '::'.join(row['path']), 'crate': row['crate'],
                             'modules': ['::'.join(seen_paths[key]), '::'.join(names)]})
        seen_paths[key] = names
    destination_of = {}
    for (names, part), row in sorted(p['parts'].items()):
        root_file, directory = module_files(row['crate'], part if part == 'testkit' else 'court')
        destination_of[(names, part)] = directory + '/' + '/'.join(row['path']) + '/mod.rs'
    relocated, foreign_files = defaultdict(list), set()
    for names, record in sorted(model.records.items()):
        node = record.node
        if node.declaration is None or node.declaration.body is not None:
            continue
        primary = (names, 'court') if (names, 'court') in p['parts'] else (names, 'testkit')
        source = node.source.resolve()
        if names in p['foreign']:
            foreign_files.add(relative(source, root))
            continue
        relocated[relative(source, root)].append(destination_of[primary])
        effects.move(source, root / destination_of[primary])
    for path, content in sorted(files.items()):
        effects.write(root / path, content)
    ledger = p['ledger']
    edits = production_edits(model, [f for f in model.moving], ledger.widen_modules, ledger.widen_items, ledger.hooks)
    hook_names = {name for _module, name in ledger.hooks}
    for source, changes in sorted(edits.items(), key=lambda kv: str(kv[0])):
        text = filesystem_path(source).read_text(encoding='utf-8')
        changes = list(changes)
        if hook_names:
            changes += [(s, e, '') for s, e in ungate_recorders(text, hook_names)]
        effects.write(source, apply_edits(text, changes))
    for blocker in sorted(ledger.cross_court):
        blockers.append({'cross_court_reference': '::'.join(blocker)})
    moved_sources = set(relocated)
    moved_inline = {(relative(model.index[f[:-1]].source.resolve(), root), f[-1]) for f in model.moving}
    config_writes(root, effects, files, moved_sources, moved_inline, relocated, model, p)
    staying_text = '\n'.join(n.own_text() for names, f in sorted(model.families.items())
                             if names not in model.moving for n in f.node.walk())
    trim_dev_dependencies(root, effects, staying_text, library_table)
    before = multiset_of_texts(model.records[n].node.own_text() for n in model.records)
    court_files = sorted([path for path in files if '/tests/' in path and path.endswith('.rs')] + sorted(foreign_files))
    program = effects.program()
    totals = Counter({'moved_families': len(model.moving), 'moved_scopes': len(model.records),
                      'crates': len(crates), 'tests': sum(before.values()),
                      'widened_modules': len(ledger.widen_modules), 'widened_items': len(ledger.widen_items),
                      'compiled_hooks': len(ledger.hooks), 'unresolved': len(ledger.unresolved),
                      'moves': len(program['moves']), 'writes': len(program['writes']), 'blockers': len(blockers)})
    return {'version': 1, 'effects': program, 'crates': crates, 'totals': dict(totals),
            'blockers': blockers, 'court_test_files': court_files,
            'test_multiset': {'\t'.join(k): v for k, v in sorted(before.items())},
            'widened_modules': sorted('::'.join(m) for m in ledger.widen_modules),
            'widened_items': sorted('::'.join(m) + '::' + n for m, n in ledger.widen_items),
            'compiled_hooks': sorted('::'.join(m) + '::' + n for m, n in ledger.hooks),
            'unresolved': sorted('::'.join(m) + '::' + str(n) for m, n in ledger.unresolved),
            'allocator_moved_from': sorted(allocators),
            'renamed_modules': sorted(f'{k} -> {v}' for k, v in p['renames'].items()),
            'categories': [{'module': '::'.join(n), 'category': c['category'], 'evidence': c['evidence']}
                           for n, c in sorted(p['categories'].items()) if c['category']]}


def config_writes(root, effects, files, moved_sources, moved_inline, relocated, model, p):
    path = root / (CONFIG + 'src-court-baseline.json')
    text, _removed = shrink_baseline(path.read_text(encoding='utf-8'), moved_sources, moved_inline)
    effects.write(path, text)
    path = root / (CONFIG + 'src-court-private-logic-audit.json')
    text, _removed = shrink_admissions(path.read_text(encoding='utf-8'), moved_sources)
    effects.write(path, text)
    path = root / (CONFIG + 'control-callers.txt')
    effects.write(path, control_callers(path.read_text(encoding='utf-8'), relocated, files, library_test_doors(model)))
    owners_of = {}
    for name, row in p['courts'].items():
        paths = sorted({'/'.join(p['placement'][n]['court'][1]) for n in row['members']})
        owners_of[name] = [x for x in paths if not any(x != y and x.startswith(y + '/') for y in paths)]
    path = root / (CONFIG + 'court-frontier-splits.toml')
    effects.write(path, splits_toml(path.read_text(encoding='utf-8'), p['courts'], owners_of))


def trim_dev_dependencies(root, effects, staying_text, library_table):
    path = root / LIBRARY_DIR / 'Cargo.toml'
    text = path.read_text(encoding='utf-8')
    head, _sep, rest = text.partition('[dev-dependencies]\n')
    if not _sep:
        return
    block, _sep2, tail = rest.partition('\n[')
    kept = []
    for line in block.splitlines():
        name = line.split('=', 1)[0].strip()
        if not name or crate_uses(staying_text, {name.replace('-', '_')}):
            kept.append(line)
    body = head + ('[dev-dependencies]\n' + '\n'.join(kept) + '\n' if kept else '')
    effects.write(path, body + ('\n[' + tail if _sep2 else ''))
