"""One canonical owner and lexical context for relocated test helper definitions."""
from collections import defaultdict
import re

from court_partition import CATEGORIES, INTERIM
from module_bindings import Bindings, use_leaves
from rust_items import items, partition
from rust_source import code, tokens
from support_readers import assign_readers

CONTEXT = {'fast': 'ordinary_context', 'exhaustive': 'reference_context', 'scale': 'growth_context',
           'serial': 'isolated_context', 'compile_fail': 'boundary_context', 'process': 'process_context',
           'oracle': 'reference_context', 'cost': 'growth_context', 'determinism': 'deterministic_context'}


class SupportModel:
    def __init__(self, owners, stem, workspace):
        self.records, self.canonical = {}, []
        self.stem, self.workspace = stem, workspace
        shared = {}
        for owner in owners:
            for index, target in enumerate(owner['targets']):
                identity = owner['cargo']['name'] + '::' + target['cargo']['name']
                for node in target['tree'].walk():
                    helper, test = partition(node.own_text())
                    exports = {item.name for item in items(helper) if item.keyword in ('fn', 'struct', 'enum', 'type', 'trait', 'const', 'static')}
                    imports = []
                    for item in items(helper):
                        if item.keyword == 'use':
                            ts = tokens(helper[item.start:item.end])
                            start = next(i for i, t in enumerate(ts) if t.value == 'use')
                            for path, alias in use_leaves(ts[start + 1:-1]):
                                imports.append(path)
                                if path and path[-1] != '*':
                                    exports.add(alias or path[-1])
                    record = {'node': node, 'owner': owner, 'target': identity, 'target_index': index,
                              'helper': helper, 'test': test, 'exports': exports, 'imports': imports,
                              'readers': set(),
                              'features': sorted(set(owner.get('effective_features', ())) | set(target['cargo'].get('required-features', ())))}
                    ancestors = [self.records[(identity, node.names[:n])] for n in range(len(node.names))]
                    record['attributes'] = ''.join(r['node'].declaration.attributes for r in ancestors if r['node'].declaration)
                    for ancestor in ancestors:
                        for item in items(ancestor['node'].own_text()):
                            if item.keyword == 'inner-attribute':
                                attribute = ancestor['node'].own_text()[item.start:item.end]
                                if re.match(r'#\s*!\s*\[\s*cfg(?:_attr)?\b', attribute):
                                    record['attributes'] += re.sub(r'^(#\s*)!', r'\1', attribute) + '\n'
                    if node.declaration:
                        record['attributes'] += node.declaration.attributes
                    record['attributes'] = re.sub(r'#\s*\[\s*path\s*=\s*(?:r#*".*?"#*|"(?:\\.|[^"\\])*")\s*\]\s*', '', record['attributes'], flags=re.S)
                    context_free = not re.search(r'\b(?:crate|super)\s*:\s*:', code(helper))
                    cache_key = (owner['cargo']['name'], node.source, node.offset, record['attributes'], tuple(record['features'])) if context_free else (identity, node.names)
                    canonical = shared.get(cache_key)
                    if canonical is None:
                        route = self.route(record, context_free)
                        canonical = dict(record, route=route, support_owner=None, users=[])
                        canonical['canonical'] = canonical
                        shared[cache_key] = canonical
                        self.canonical.append(canonical)
                    canonical['users'].append(record)
                    canonical['readers'].update(record['readers'])
                    record['canonical'] = canonical
                    self.records[(identity, node.names)] = record
        self.bindings = Bindings(self.records)

    def bind(self, units):
        assign_readers(self, units)
        self.canonical.sort(key=lambda r: (r['support_owner'] or '', r['route'], str(r['node'].source)))
        self.ensure_routes()

    def route(self, record, context_free):
        names = record['node'].names
        origin = record['owner']['cargo']['name'].removeprefix('worth-').replace('-', '_')
        target = record['owner']['targets'][record['target_index']]['cargo']['name']
        if context_free and names:
            names = min((names,), key=lambda n: (len(n), n))
        if not names:
            return (target,) if target != record['owner']['cargo']['name'].replace('-', '_') else (origin + '_context', 'root')
        if names[0] in CONTEXT:
            rest = names[1:]
            return (rest[0], CONTEXT[names[0]]) + rest[1:] if rest else (origin + '_context', CONTEXT[names[0]])
        if len(record['owner']['targets']) > 1 and not context_free:
            return (target,) + names
        return names

    def ensure_routes(self):
        groups = defaultdict(list)
        for record in self.canonical:
            groups[(record['support_owner'], record['route'])].append(record)
        for (_owner, route), group in groups.items():
            if len(group) <= 1:
                continue
            for record in group:
                target = record['owner']['targets'][record['target_index']]['cargo']['name']
                origin = record['owner']['cargo']['name'].removeprefix('worth-').replace('-', '_')
                record['route'] = (origin + '_' + route[0],) + route[1:] if len({r['owner']['cargo']['name'] for r in group}) > 1 else route + (target,)
        seen = set()
        for record in self.canonical:
            key = (record['support_owner'], record['route'])
            if key in seen:
                raise ValueError('support responsibility collision: ' + repr(key))
            seen.add(key)

    def record(self, unit, node=None):
        return self.records[(unit['target'], (node or unit['node']).names)]

    def import_context(self, unit, node=None):
        record = self.record(unit, node)
        canonical = record['canonical']
        if not record['helper'].strip() or not canonical['support_owner'] or canonical.get('active') is False:
            return ''
        if not any(item.keyword not in ('use', 'inner-attribute') for item in items(record['helper'])):
            return ''
        prefix = 'crate' if canonical['support_owner'] == unit['court'] else canonical['support_owner'].replace('-', '_')
        return '#[allow(unused_imports)] use ' + '::'.join((prefix,) + canonical['route']) + '::*;\n'
