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
        self.records = records

    def lookup(self, record, names, absolute=False):
        target, scope = record['target'], record['node'].names
        parts = list(names)
        if parts and parts[0] == 'crate':
            bases, parts = [()], parts[1:]
        elif parts and parts[0] == 'self':
            bases, parts = [scope], parts[1:]
        elif parts and parts[0] == 'super':
            levels = 0
            while parts and parts[0] == 'super':
                levels += 1
                parts.pop(0)
            bases = [scope[:max(0, len(scope) - levels)]]
        else:
            bases = [scope[:n] for n in range(len(scope), -1, -1)]
        for base in bases:
            path = base + tuple(parts)
            for length in range(len(path), -1, -1):
                destination = self.records.get((target, path[:length]))
                if destination is None:
                    continue
                suffix = path[length:]
                if suffix and suffix[0] not in destination['exports'] and not absolute:
                    continue
                if not destination['helper'].strip() or destination['canonical'].get('active') is False:
                    if not suffix:
                        return destination, ()
                    continue
                return destination, suffix
        return None

    def path(self, record, names, current_owner, absolute=False):
        found = self.lookup(record, names, absolute)
        if found is None:
            return tuple(names)
        destination, suffix = found
        canonical = destination['canonical']
        if not canonical['helper'].strip() or canonical.get('active') is False:
            return None
        prefix = 'crate' if canonical['support_owner'] == current_owner else canonical['support_owner'].replace('-', '_')
        return (prefix,) + canonical['route'] + suffix

    def rewrite(self, text, record, current_owner):
        stream, changes, receipts = tokens(text), [], []
        matching = pairs(stream)
        uses = []
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
                if names and names[-1] == 'self':
                    names = names[:-1]
                glob = names and names[-1] == '*'
                path = self.path(record, names[:-1] if glob else names, current_owner,
                                 names[0] in ('crate', 'self', 'super')) if names else None
                if path is None:
                    continue
                if glob:
                    path += ('*',)
                if not alias and not glob and names and path[-1] != names[-1]:
                    alias = names[-1]
                rendered.append('::'.join(path) + (' as ' + alias if alias else ''))
            start, end = stream[i + 1].start, stream[j].start
            replacement = '{' + ', '.join(rendered) + '}'
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
                absolute = names[0] in ('crate', 'self', 'super')
                path = self.path(record, names, current_owner, absolute)
                if path and path != tuple(names):
                    changes.append((token.start, stream[j - 1].end, '::'.join(path)))
                i = j
            else:
                i += 1
        for a, b, value in changes:
            receipts.append({'source': str(record['node'].source), 'scope': '::'.join(record['node'].names),
                             'offset': a, 'kind': 'rust-module-path', 'before': text[a:b], 'after': value})
        return replace_spans(text, changes), receipts
