"""Resolve old lexical module paths into the canonical support graph."""
from rust_source import pairs, replace_spans, tokens


def use_leaves(stream):
    matching = pairs(stream)
    def descend(start, end, prefix):
        current, i = list(prefix), start
        while i < end:
            value = stream[i].value
            if value == '{':
                close = matching[i]
                yield from descend(i + 1, close, current)
                current = list(prefix)
                i = close + 1
            elif value == ',':
                if current != list(prefix):
                    yield current, None
                current = list(prefix)
                i += 1
            elif value == 'as':
                alias = stream[i + 1].value
                yield current, alias
                current = list(prefix)
                i += 2
            elif value == ':':
                i += 1
            else:
                current.append(value)
                i += 1
        if current != list(prefix):
            yield current, None
    return list(descend(0, len(stream), []))


class Bindings:
    def __init__(self, records):
        from rust_items import items
        self.records, self.direct, self.globs, self.locals = records, {}, {}, {}
        self.used_cache, self.external_cache = {}, {}
        for key, record in records.items():
            direct, globs, local = {}, [], set()
            for item in items(record['node'].own_text()):
                if item.keyword in ('fn', 'struct', 'enum', 'type', 'trait', 'const', 'static'):
                    direct[item.name] = ('case' if item.test else 'item', record, (item.name,))
                    local.add(item.name)
            for (target, names), child in records.items():
                if target == record['target'] and len(names) == len(record['node'].names) + 1 and names[:-1] == record['node'].names:
                    direct[names[-1]] = ('module', child, ())
                    local.add(names[-1])
            for item in items(record['helper']):
                if item.keyword != 'use':
                    continue
                stream = tokens(record['helper'][item.start:item.end])
                start = next(i for i, t in enumerate(stream) if t.value == 'use')
                for path, alias in use_leaves(stream[start + 1:-1]):
                    if path and path[-1] == '*':
                        globs.append(tuple(path[:-1]))
                    elif path:
                        if path[-1] == 'self':
                            path = path[:-1]
                        name = alias or path[-1]
                        if name != '_':
                            direct[name] = ('alias', record, tuple(path))
                            local.add(name)
            self.direct[key], self.globs[key], self.locals[key] = direct, globs, local

    def key(self, record):
        return record['target'], record['node'].names

    def external(self, record):
        target = record['target']
        if target in self.external_cache:
            return self.external_cache[target]
        package = record['owner']['cargo']
        names = {'std', 'core', 'alloc', 'clippy', 'rustfmt', package['name'].replace('-', '_')} | {
            (d.get('rename') or d['name']).replace('-', '_') for d in package['dependencies']}
        self.external_cache[target] = names
        return names

    def symbol(self, record, name, trail=()):
        marker = (self.key(record), name)
        if marker in trail:
            return None
        trail = trail + (marker,)
        binding = self.direct[self.key(record)].get(name)
        if binding:
            if binding[0] == 'alias':
                return self.resolve(record, binding[2], trail, imported=True)
            return binding
        found = []
        for path in self.globs[self.key(record)]:
            destination = self.resolve(record, path, trail, imported=True)
            if destination and destination[0] == 'module':
                candidate = self.symbol(destination[1], name, trail)
                if candidate:
                    found.append(candidate)
        if not found:
            return None
        return found[0]

    def resolve(self, record, names, trail=(), imported=False):
        parts = list(names)
        if not parts:
            return ('module', record, ())
        first = parts.pop(0)
        if first == 'crate':
            result = ('module', self.records[(record['target'], ())], ())
        elif first == 'self':
            result = ('module', record, ())
        elif first == 'super':
            scope = record['node'].names[:-1]
            while parts and parts[0] == 'super':
                scope = scope[:-1]
                parts.pop(0)
            result = ('module', self.records[(record['target'], scope)], ())
        elif imported and first in self.external(record):
            result = ('external', None, (first,))
        else:
            result = self.symbol(record, first, trail)
            if result is None and first in self.external(record):
                result = ('external', None, (first,))
        for part in parts:
            if result is None:
                return None
            kind, destination, suffix = result
            result = self.symbol(destination, part, trail) if kind == 'module' else (kind, destination, suffix + (part,))
        return result

    def lookup(self, record, names, absolute=False):
        result = self.resolve(record, names)
        return (result[1], result[2]) if result and result[0] != 'external' else None

    def address(self, result, current_owner):
        if result is None:
            return None
        kind, record, suffix = result
        if kind == 'external':
            return suffix
        if kind == 'case':
            if 'test_module' not in record:
                return None
            prefix = 'crate' if record['test_owner'] == current_owner else record['test_owner'].replace('-', '_')
            return (prefix,) + record['test_module'] + suffix
        canonical = record['canonical']
        owner, route = canonical['support_owner'], canonical['route']
        if kind == 'module' and (not canonical['helper'].strip() or canonical.get('active') is False):
            descendants = [r['canonical'] for (target, names), r in self.records.items()
                           if target == record['target'] and names[:len(record['node'].names)] == record['node'].names
                           and len(names) > len(record['node'].names) and r['canonical'].get('active', True)
                           and r['canonical']['helper'].strip() and r['canonical']['route'][:len(route)] == route]
            owners = {r['support_owner'] for r in descendants}
            if len(owners) != 1:
                return None
            owner = next(iter(owners))
        if not owner:
            return None
        prefix = 'crate' if owner == current_owner else owner.replace('-', '_')
        return (prefix,) + route + suffix

    def path(self, record, names, current_owner, absolute=False):
        result = self.resolve(record, names)
        return tuple(names) if result is None else self.address(result, current_owner)

    def external_globs(self, record, trail=()):
        if self.key(record) in trail:
            return []
        trail = trail + (self.key(record),)
        result = []
        for path in self.globs[self.key(record)]:
            target = self.resolve(record, path, imported=True)
            if target and target[0] == 'external':
                result.append(target[2] + ('*',))
            elif target and target[0] == 'module':
                result.extend(self.external_globs(target[1], trail))
        return result

    def used_names(self, record):
        from rust_items import items
        key = self.key(record)
        if key in self.used_cache:
            return self.used_cache[key]
        text = record['node'].own_text()
        spans = [(i.start, i.end) for i in items(text) if i.keyword == 'use']
        names = {t.value for t in tokens(text) if t.kind == 'ident' and not any(a <= t.start < b for a, b in spans)} - self.locals[key]
        self.used_cache[key] = names
        return names

    def glob_paths(self, record, names, current_owner):
        target = self.resolve(record, names, imported=True)
        if target is None:
            return []
        if target[0] == 'external':
            return [(target[2] + ('*',), None)]
        if target[0] != 'module':
            path = self.address(target, current_owner)
            return [(path + ('*',), None)] if path else []
        destination = target[1]
        result = [(p, None) for p in self.external_globs(destination)]
        for name in sorted(self.used_names(record)):
            symbol = self.symbol(destination, name)
            path = self.address(symbol, current_owner)
            if path:
                result.append((path, name if path[-1] != name else None))
        return result

    def rewrite(self, text, record, current_owner):
        stream, changes, receipts = tokens(text), [], []
        matching, uses = pairs(stream), []
        for i, token in enumerate(stream):
            if token.value != 'use':
                continue
            j = i + 1
            while j < len(stream) and stream[j].value != ';':
                j = matching[j] + 1 if stream[j].value in ('{', '(', '[') else j + 1
            if j == len(stream):
                continue
            rendered = []
            for names, alias in use_leaves(stream[i + 1:j]):
                if not names:
                    continue
                if names[-1] == '*':
                    destinations = self.glob_paths(record, names[:-1], current_owner)
                else:
                    if names[-1] == 'self':
                        names = names[:-1]
                    path = self.address(self.resolve(record, names, imported=True), current_owner)
                    destinations = [(path or tuple(names), alias)] if path else []
                    if destinations and not alias and path[-1] != names[-1]:
                        destinations = [(path, names[-1])]
                rendered.extend('::'.join(path) + (' as ' + name if name else '') for path, name in destinations)
            start, end = stream[i + 1].start, stream[j].start
            replacement = '{' + ', '.join(dict.fromkeys(rendered)) + '}'
            if text[start:end].strip() != replacement:
                changes.append((start, end, replacement))
            uses.append((token.start, stream[j].end))
        i = 0
        while i < len(stream):
            token = stream[i]
            if token.kind != 'ident' or any(a <= token.start < b for a, b in uses) or (i and stream[i - 1].value in (':', '$')):
                i += 1
                continue
            j, names = i + 1, [token.value]
            while j + 2 < len(stream) and [stream[j].value, stream[j + 1].value] == [':', ':'] and stream[j + 2].kind == 'ident':
                names.append(stream[j + 2].value)
                j += 3
            if len(names) > 1:
                path = self.path(record, names, current_owner)
                if path and path != tuple(names):
                    changes.append((token.start, stream[j - 1].end, '::'.join(path)))
                i = j
            else:
                i += 1
        for a, b, value in changes:
            receipts.append({'source': str(record['node'].source), 'scope': '::'.join(record['node'].names),
                             'offset': a, 'kind': 'rust-module-path', 'before': text[a:b], 'after': value})
        return replace_spans(text, changes), receipts
