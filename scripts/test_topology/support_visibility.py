"""Promote relocated test support, including fields and inherent associated items."""
from rust_items import items
from rust_source import pairs, replace_spans, tokens


def promote(text, visibility='pub'):
    changes = []
    def public(start, end):
        if text[start:end].strip() != visibility:
            changes.append((start, end, visibility + ' '))

    def scope(start, end, trait_impl=False):
        part = text[start:end]
        for item in items(part):
            if item.keyword not in ('inner-attribute', 'impl', 'macro') and not trait_impl:
                public(start + item.visibility[0], start + item.visibility[1])
            if item.keyword == 'struct' and item.body:
                fields(start + item.body[0], start + item.body[1])
            elif item.keyword == 'mod' and item.body:
                scope(start + item.body[0], start + item.body[1])
            elif item.keyword == 'impl' and item.body:
                header = part[item.start:item.body[0]]
                is_trait = any(t.value == 'for' for t in tokens(header))
                scope(start + item.body[0], start + item.body[1], is_trait)

    def fields(start, end):
        stream = tokens(text[start:end])
        matching = pairs(stream)
        i = 0
        while i < len(stream):
            while i < len(stream) and stream[i].value == '#':
                i = matching[i + 1] + 1
            if i >= len(stream):
                break
            first = i
            if stream[i].value == 'pub':
                i += 1
                if i < len(stream) and stream[i].value == '(':
                    i = matching[i] + 1
            public(start + stream[first].start, start + (stream[i].start if i != first else stream[first].start))
            angle = 0
            while i < len(stream):
                value = stream[i].value
                if value in ('(', '[', '{'):
                    i = matching[i] + 1
                    continue
                if value == '<':
                    angle += 1
                elif value == '>':
                    angle = max(0, angle - 1)
                if value == ',' and not angle:
                    i += 1
                    break
                i += 1

    scope(0, len(text))
    records = [{'offset': a, 'before': text[a:b], 'after': value, 'kind': 'support-visibility'}
               for a, b, value in sorted(set(changes))]
    return replace_spans(text, sorted(set(changes))), records
