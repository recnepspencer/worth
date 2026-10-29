"""Rebase crate-relative module paths from the recorded old-to-new module map."""
from court_roots import module_path, area_names, origin_name, target_area
from rust_source import tokens, replace_spans


class ModulePaths:
    def __init__(self, units, assignments, aliases):
        self.maps = {}
        self.sources = {}
        candidates = {}
        for unit in units:
            owner = unit['target']
            for node in list(unit['node'].walk()) + unit['ancestors'] + [n for s in unit['support'] for n in s.walk()]:
                self.sources.setdefault(node.source, set()).add(owner)
            routes = self.maps.setdefault(owner, {})
            old = unit['node'].names
            if old:
                routes[old] = ('crate',) + module_path(unit)
            for size in range(1, len(old)):
                prefix = old[:size]
                category = tuple(unit['category'].split('::')) if unit['category'] else ()
                new = ('crate',) + category + (origin_name(unit['owner']),) + target_area(unit) + area_names(prefix)
                candidates.setdefault((owner, prefix), set()).add(new)
            for support in unit['support']:
                for node in support.walk():
                    assignment = assignments.get(node.source)
                    if assignment and node.source in aliases:
                        routes[node.names] = (assignment['owner'].replace('-', '_'),) + tuple(aliases[node.source].split('::'))
                    elif len(node.names) == 1 and target_area(unit):
                        category = tuple(unit['category'].split('::')) if unit['category'] else ()
                        routes[node.names] = ('crate',) + category + (origin_name(unit['owner']),) + target_area(unit) + node.names
        for (owner, prefix), options in sorted(candidates.items()):
            if len(options) == 1:
                self.maps[owner].setdefault(prefix, next(iter(options)))

    def rewrite(self, text, owner, source, current_kit=None):
        routes = {}
        for target in sorted(self.sources.get(source, ())):
            routes.update(self.maps[target])
        stream, changes, records = tokens(text), [], []
        for i, token in enumerate(stream):
            if token.value != 'crate' or (i and stream[i - 1].value == '$'):
                continue
            j, names, ends = i + 1, [], []
            while j + 2 < len(stream) and stream[j].value == ':' and stream[j + 1].value == ':' and stream[j + 2].kind == 'ident':
                names.append(stream[j + 2].value)
                ends.append(stream[j + 2].end)
                j += 3
            match = next((tuple(names[:n]) for n in range(len(names), 0, -1) if tuple(names[:n]) in routes), None)
            if not match:
                continue
            destination = routes[match]
            if current_kit and destination[0] == current_kit.replace('-', '_'):
                destination = ('crate',) + destination[1:]
            replacement = '::'.join(destination)
            end = ends[len(match) - 1]
            before = text[token.start:end]
            if before != replacement:
                changes.append((token.start, end, replacement))
                records.append({'source': str(source), 'occurrence': token.start,
                                'before': before, 'after': replacement, 'kind': 'rust-module-path'})
        return replace_spans(text, changes), records
