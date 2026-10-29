"""Rebind test support predicates to the original package's resolved build context."""
from rust_source import pairs, replace_spans, string_value, tokens


def rebind(text, features, helper=False):
    stream, changes = tokens(text), []
    matching = pairs(stream)
    for i, token in enumerate(stream):
        if token.value not in ('cfg', 'cfg_attr'):
            continue
        opening = i + 2 if i + 1 < len(stream) and stream[i + 1].value == '!' else i + 1
        if opening >= len(stream) or stream[opening].value != '(':
            continue
        end = matching[opening]
        if token.value == 'cfg_attr':
            j = opening + 1
            while j < end:
                if stream[j].value == ',':
                    end = j
                    break
                j = matching[j] + 1 if stream[j].value in ('(', '[', '{') else j + 1
        j = opening + 1
        while j < end:
            current = stream[j]
            if current.value == 'feature' and j + 2 < end and stream[j + 1].value == '=' and stream[j + 2].kind == 'string':
                value = 'all()' if string_value(stream[j + 2]) in features else 'any()'
                changes.append((current.start, stream[j + 2].end, value, 'feature-namespace'))
                j += 3
            elif helper and current.value == 'test':
                changes.append((current.start, current.end, 'all()', 'test-context'))
                j += 1
            else:
                j += 1
    unique = sorted(set(changes))
    receipts = [{'offset': a, 'before': text[a:b], 'after': value, 'kind': kind}
                for a, b, value, kind in unique]
    return replace_spans(text, [(a, b, value) for a, b, value, _kind in unique]), receipts
