"""Struct literals over private fields become `Type::new_for_test(..)` calls.

A literal `Path { a: x, b }` turns into `Path::new_for_test(x, b)` with the arguments in the
struct's declaration order; a tuple constructor `Path(x)` into `Path::new_for_test(x)`.
Literals with a functional update (`..base`) and struct patterns are left for a hand fix.
"""
import re

from src_door_members_text import split_top

NAMED = re.compile(r'(?:fields? `(\w+)`.*? of struct|of struct) `(?:[\w:]*::)?(\w+)')


def constructed_struct(diagnostic):
    match = NAMED.search(diagnostic['message'])
    if match:
        return match.group(2)
    for child in diagnostic['children']:
        named = re.search(r'struct named `(\w+)`', child['message'])
        if named:
            return named.group(1)
    for span in diagnostic['spans']:
        if span['is_primary'] and span['text']:
            line = span['text'][0]
            word = line['text'][line['highlight_start'] - 1:line['highlight_end'] - 1]
            return word.split('::')[-1] or None
    return None


def literal_rewrite(data, at, code, shape):
    """(start, end, replacement) bytes for the literal around byte `at`, or None."""
    if shape is None:
        return None
    text = data.decode('utf-8')
    char = len(data[:at].decode('utf-8'))
    if code == 'E0423':
        match = re.match(r'[\w:]+', text[char:])
        if match is None or not text[char + match.end():].startswith('('):
            return None
        end = len(text[:char + match.end()].encode('utf-8'))
        return (end, end, '::new_for_test', tuple(shape['fields']))
    depth, i = 0, char
    while i >= 0:
        if text[i] == '}':
            depth += 1
        elif text[i] == '{':
            if depth == 0:
                break
            depth -= 1
        i -= 1
    if i < 0:
        return None
    path = re.search(r'([\w:]+)\s*$', text[:i])
    if path is None:
        return None
    before = text[:path.start()].rstrip()
    depth, j = 0, i
    while j < len(text):
        depth += {'{': 1, '}': -1}.get(text[j], 0)
        if depth == 0:
            break
        j += 1
    after = text[j + 1:].lstrip()
    if re.search(r'\b(let|if let|while let)\s*$', before) or before.endswith('|') or after.startswith('=>') \
            or re.match(r'=(?!=)', after):
        return None
    values = {}
    for part in split_top(text[i + 1:j]):
        part = part.strip()
        if not part:
            continue
        if part.startswith('..'):
            return None
        name, colon, value = part.partition(':')
        values[name.strip()] = value.strip() if colon else name.strip()
    if set(values) != set(shape['fields']):
        return None
    # Arguments keep the literal's order: struct literals evaluate in written order.
    arguments = ', '.join(values.values())
    start = len(text[:path.start(1)].encode('utf-8'))
    end = len(text[:j + 1].encode('utf-8'))
    return (start, end, f'{path.group(1)}::new_for_test({arguments})', tuple(values))
