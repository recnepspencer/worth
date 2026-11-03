"""Token-level analysis of the call sites the member doors rewrite.

Everything works on `rust_source.tokens`, which already skips comments and keeps strings and
character literals whole, and on `pairs` for bracket matching, so comments, strings, braces,
arrows and commas never mislead a scan. Offsets are character offsets into the decoded text;
`Source.char_offset` maps a diagnostic's byte offset to one.
"""
from rust_source import pairs, tokens

KEYWORDS = {'mut', 'ref', 'move', 'return', 'in', 'if', 'else', 'match', 'let', 'as', 'box', 'break',
            'yield', 'while', 'for', 'loop', 'unsafe', 'async', 'await', 'fn', 'impl', 'dyn', 'where'}
COMPOUND = ('+', '-', '*', '/', '%', '|', '&', '^')


class Source:
    def __init__(self, text):
        self.text = text
        self.stream = tokens(text)
        self.close = pairs(self.stream)
        self.open = {v: k for k, v in self.close.items()}
        self.at = {t.start: i for i, t in enumerate(self.stream)}

    def char_offset(self, byte_offset):
        return len(self.text.encode('utf-8')[:byte_offset].decode('utf-8'))

    def index(self, char_offset):
        """Index of the token that starts at `char_offset`."""
        return self.at.get(char_offset)

    def value(self, i):
        return self.stream[i].value if 0 <= i < len(self.stream) else None

    def adjacent(self, i, j):
        return self.stream[i].end == self.stream[j].start

    def span_text(self, first, last):
        return self.text[self.stream[first].start:self.stream[last].end]


def postfix_start(source, dot):
    """Index of the first token of the postfix expression whose `.` is token `dot`."""
    j, start = dot - 1, dot
    while j >= 0:
        value = source.value(j)
        if value in (')', ']') and j in source.open:
            start = source.open[j]
            j = start - 1
            if source.value(j) in (')', ']', '?') or (source.stream[j].kind == 'ident'
                                                    and source.value(j) not in KEYWORDS):
                continue  # a call or index: its callee comes next
            break         # a parenthesized primary
        if value == '?':
            j -= 1
            continue
        if source.stream[j].kind == 'ident' and value not in KEYWORDS:
            start = j
            if source.value(j - 1) == '.' and source.value(j - 2) != '.':
                j -= 2
                continue
            if source.value(j - 1) == ':' and source.value(j - 2) == ':':
                j -= 3
                continue
        break
    return start


def assigned_after(source, field):
    """Whether the place ending at token `field` (with any index groups after) is assigned."""
    k = field + 1
    while source.value(k) == '[' and k in source.close:
        k = source.close[k] + 1
    value = source.value(k)
    if value == '=' and source.value(k + 1) != '=':
        return True
    if value in COMPOUND and source.value(k + 1) == '=' and source.adjacent(k, k + 1):
        return True
    return value in ('<', '>') and source.value(k + 1) == value and source.value(k + 2) == '='


def borrowed_mut(source, first):
    return source.value(first - 1) == 'mut' and source.value(first - 2) == '&'


def field_sites(source, field_char):
    """(insert, (start, end)) for `(*expr.f_for_test())`: where `(*` goes, the field token span."""
    field = source.index(field_char)
    if field is None or source.value(field - 1) != '.':
        return None
    first = postfix_start(source, field - 1)
    token = source.stream[field]
    return source.stream[first].start, (token.start, token.end), field, first


def split_top(source, open_index):
    """Token index ranges of the top-level comma-separated parts inside a bracket group."""
    close = source.close[open_index]
    parts, depth, start, k = [], 0, open_index + 1, open_index + 1
    while k < close:
        value = source.value(k)
        if k in source.close:
            k = source.close[k] + 1  # nested groups hold no top-level commas
            continue
        if value == '<':
            depth += 1
        elif value == '>' and not (source.value(k - 1) == '-' and source.adjacent(k - 1, k)):
            depth = max(depth - 1, 0)
        elif value == ',' and depth == 0:
            parts.append((start, k - 1))
            start = k + 1
        k += 1
    if start <= close - 1:
        parts.append((start, close - 1))
    return [(a, b) for a, b in parts if a <= b]


def enclosing_brace(source, i):
    """Index of the innermost `{` whose group contains token `i`."""
    j = i - 1
    while j >= 0:
        value = source.value(j)
        if value in (')', ']', '}') and j in source.open:
            j = source.open[j] - 1
            continue
        if value == '{':
            return j
        j -= 1
    return None


def path_before(source, i):
    """(first index, path text) of the `a::b::C` path that ends at token `i - 1`."""
    j = i - 1
    if j < 0 or source.stream[j].kind != 'ident':
        return None
    while source.value(j - 1) == ':' and source.value(j - 2) == ':' and source.stream[j - 3].kind == 'ident':
        j -= 3
    return j, source.span_text(j, i - 1)


def literal_parts(source, brace):
    """[(field, value text | None)] of a struct literal, and the functional-update base text."""
    fields, base = [], None
    for a, b in split_top(source, brace):
        if source.value(a) == '.' and source.value(a + 1) == '.':
            base = source.span_text(a + 2, b)
            continue
        if source.value(a + 1) == ':' and source.value(a + 2) != ':':
            fields.append((source.value(a), source.span_text(a + 2, b)))
        elif a == b:
            fields.append((source.value(a), None))
        else:
            return None
    return fields, base


def is_pattern(source, path_first, brace):
    before = source.value(path_first - 1)
    after = source.value(source.close[brace] + 1)
    if before in ('let', '|') or (before == '(' and source.value(path_first - 2) in ('Some', 'Ok', 'Err')):
        return True
    return after == '=' and source.value(source.close[brace] + 2) == '>' or \
        (after == '=' and source.value(source.close[brace] + 2) != '=' and before != '=')
