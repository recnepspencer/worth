"""The text of one part of a moving module scope: the testkit part, the court part, or all of it.

A split module keeps its tests (and its imports and test-emitting macros) in the court part and
its support items in the testkit part. Library-trait impls stay in the library; inherent impls
of library types become door traits.
"""
import re

from rust_source import replace_spans, tokens


def leading_inner_attributes(text):
    """Leading `//!` doc lines, `/*! */` blocks and `#![...]` attributes, which must stay first."""
    import re
    position = 0
    while True:
        match = re.match(r'\s*(?://![^\n]*\n?|/\*!.*?\*/)', text[position:], re.S)
        if match:
            position += match.end()
            continue
        stream = tokens(text[position:])
        if len(stream) > 2 and stream[0].value == '#' and stream[1].value == '!':
            depth, j = 0, 2
            while j < len(stream):
                depth += {'[': 1, ']': -1}.get(stream[j].value, 0)
                if depth == 0:
                    break
                j += 1
            position += stream[j].end
            continue
        break
    return text[:position], position


def part_text(record, part, disposition):
    """The source of one part of a module scope; a split keeps tests and imports in the court."""
    text = record.node.own_text()
    if record.kept_impls or record.doors:
        # Library-trait impls stay in the library; inherent impls become door traits.
        text = replace_spans(text, [(start, end, '') for start, end in record.kept_impls]
                             + [(door.start, door.end, door.replacement) for door in record.doors])
        record = record_without_kept(record, text)
    if disposition != 'split':
        return text
    inner, start = leading_inner_attributes(text)
    pieces = [inner]
    from src_families import macro_generated
    test_macros = {m['macro'] for m in macro_generated(text)}
    for item in record.items:
        if item.end <= start:
            continue
        # Test-emitting macros stay with the tests; other item macros declare support names
        # and go to the testkit; other `macro_rules!` are textual, so both parts keep them.
        body = text[item.start:item.end]
        emits_tests = any(re.search(r'\b' + m + r'\s*!', body) for m in test_macros)
        macro = item.name == '!' or item.keyword == 'macro_rules'
        court = item.test or (macro and (emits_tests or item.keyword == 'macro_rules')) or item.keyword == 'use'
        testkit = not item.test and (not macro or not emits_tests)
        if (part == 'court' and court) or (part == 'testkit' and testkit):
            pieces.append('\n' + text[item.start:item.end])
    return ''.join(pieces) + '\n'


def record_without_kept(record, text):
    from src_move_model import items_of
    import copy
    clone = copy.copy(record)
    clone.items = items_of(text)
    return clone
