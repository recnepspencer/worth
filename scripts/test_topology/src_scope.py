"""Top-level items of one Rust scope with their attributes and visibility, never expanded."""
from rust_source import pairs, tokens

QUALIFIERS = ('async', 'unsafe', 'default', 'extern')


def scope_items(text):
    """(attributes, visibility, keyword, name, start, end) for each item at this scope."""
    stream = tokens(text)
    matching = pairs(stream)
    i, result = 0, []
    while i < len(stream):
        attributes, start = [], stream[i].start
        while i + 1 < len(stream) and stream[i].value == '#' and stream[i + 1].value in ('[', '!'):
            bracket = i + 2 if stream[i + 1].value == '!' else i + 1
            attributes.append(text[stream[i].start:stream[matching[bracket]].end])
            i = matching[bracket] + 1
        if i >= len(stream):
            break
        visibility = ''
        if stream[i].value == 'pub':
            visibility = 'pub'
            if i + 1 < len(stream) and stream[i + 1].value == '(':
                visibility = text[stream[i].start:stream[matching[i + 1]].end]
                i = matching[i + 1]
            i += 1
        i = skip_qualifiers(stream, i)
        keyword = stream[i].value if i < len(stream) else ''
        name = stream[i + 1].value if i + 1 < len(stream) else ''
        j = i
        while j < len(stream) and stream[j].value not in (';', '{'):
            j = matching[j] + 1 if stream[j].value in ('(', '[') else j + 1
        if j < len(stream) and stream[j].value == '{':
            j = matching[j]
            if j + 1 < len(stream) and stream[j + 1].value == ';':
                j += 1
        end = stream[min(j, len(stream) - 1)].end
        result.append((' '.join(attributes), visibility, keyword, name, start, end))
        i = j + 1
    return result


def skip_qualifiers(stream, i):
    """Step over `async`, `unsafe`, `default`, `extern "abi"` and `const fn` before the keyword."""
    while i + 1 < len(stream):
        value = stream[i].value
        if value in QUALIFIERS or stream[i].kind == 'string':
            i += 1
        elif value == 'const' and stream[i + 1].value in ('fn', 'unsafe', 'async', 'extern'):
            i += 1
        else:
            break
    return i
