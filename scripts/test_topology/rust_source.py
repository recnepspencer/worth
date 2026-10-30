"""Lossless Rust tokens and module declarations; offsets always address source text."""
from dataclasses import dataclass
from functools import lru_cache
import re

RAW = re.compile(r'(?:br|cr|r)(#*)"')
CHAR = re.compile(r"(?:b)?'(?:\\(?:u\{[^}]+\}|x..|.)|[^'\\\n])'")
WORD = re.compile(r'(?:r#)?[A-Za-z_][A-Za-z_0-9]*')


@dataclass(frozen=True)
class Token:
    value: str
    start: int
    end: int
    kind: str = 'punct'


@lru_cache(maxsize=512)
def tokens(text):
    """Skip comments, retain strings, and distinguish lifetimes from character literals."""
    result, pos = [], 0
    while pos < len(text):
        start = pos
        if text[pos].isspace():
            pos += 1
            continue
        if text.startswith('//', pos):
            end = text.find('\n', pos)
            pos = len(text) if end < 0 else end + 1
            continue
        if text.startswith('/*', pos):
            pos, depth = pos + 2, 1
            while depth and pos < len(text):
                if text.startswith('/*', pos):
                    depth += 1
                    pos += 2
                elif text.startswith('*/', pos):
                    depth -= 1
                    pos += 2
                else:
                    pos += 1
            if depth:
                raise ValueError('unterminated block comment')
            continue
        raw = RAW.match(text, pos) if text[pos] in 'bcr' else None
        if raw:
            close = '"' + raw[1]
            end = text.find(close, raw.end())
            if end < 0:
                raise ValueError('unterminated raw string')
            pos = end + len(close)
            result.append(Token(text[start:pos], start, pos, 'string'))
            continue
        prefix = 1 if text[pos:pos + 2] in ('b"', 'c"') else 0
        if text[pos + prefix:pos + prefix + 1] == '"':
            pos += prefix + 1
            while pos < len(text):
                if text[pos] == '\\':
                    pos += 2
                elif text[pos] == '"':
                    pos += 1
                    break
                else:
                    pos += 1
            result.append(Token(text[start:pos], start, pos, 'string'))
            continue
        char = CHAR.match(text, pos) if text[pos] in "b'" else None
        if char:
            pos = char.end()
            result.append(Token(text[start:pos], start, pos, 'char'))
            continue
        word = WORD.match(text, pos)
        if word:
            pos = word.end()
            result.append(Token(text[start:pos], start, pos, 'ident'))
        else:
            pos += 1
            result.append(Token(text[start:pos], start, pos))
    return result


def string_value(token):
    value = token.value
    raw = re.fullmatch(r'(?:br|cr|r)(#*)"(.*)"\1', value, re.S)
    if raw:
        return raw[2]
    # Paths use Rust's ordinary escapes; avoid corrupting UTF-8 with unicode_escape.
    value = value[value.index('"') + 1:-1]
    return re.sub(r'\\([\\"nrt0])', lambda m: {'n': '\n', 'r': '\r', 't': '\t',
                                               '0': '\0'}.get(m[1], m[1]), value)


def pairs(stream):
    stack, result = [], {}
    for i, token in enumerate(stream):
        if token.kind in ('string', 'char'):
            continue
        if token.value in ('{', '[', '('):
            stack.append(i)
        elif token.value in ('}', ']', ')'):
            if not stack:
                raise ValueError('unbalanced Rust delimiters')
            other = stack.pop()
            if stream[other].value != {'}': '{', ']': '[', ')': '('}[token.value]:
                raise ValueError('mismatched Rust delimiters')
            result[other] = i
    if stack:
        raise ValueError('unbalanced Rust delimiters')
    return result


@dataclass(frozen=True)
class Module:
    name: str
    start: int
    end: int
    path: str | None
    path_span: tuple | None
    body: tuple | None
    attributes: str


def modules(text):
    """Read module items at this scope, never declarations inside functions/macros."""
    stream = tokens(text)
    matching = pairs(stream)
    result, i, attrs, attr_start = [], 0, '', None
    while i < len(stream):
        token = stream[i]
        if token.value == '#' and i + 1 < len(stream) and stream[i + 1].value == '[':
            end = matching[i + 1]
            attr_start = token.start if attr_start is None else attr_start
            attrs += text[token.start:stream[end].end] + '\n'
            i = end + 1
            continue
        item_start = token.start if attr_start is None else attr_start
        if token.value == 'pub':
            i += 1
            if i < len(stream) and stream[i].value == '(':
                i = matching[i] + 1
            if i >= len(stream):
                break
            token = stream[i]
        if token.value == 'mod' and i + 2 < len(stream):
            name, opening = stream[i + 1].value, stream[i + 2]
            path_match = re.search(r'#\s*\[\s*path\s*=\s*(r#*".*?"#*|"(?:\\.|[^"\\])*")\s*\]', attrs, re.S)
            path = string_value(Token(path_match[1], 0, 0, 'string')) if path_match else None
            span = None
            if path_match:
                candidates = [t for t in stream if item_start <= t.start < token.start and t.kind == 'string']
                t = next(t for t in candidates if string_value(t) == path)
                span = (t.start, t.end)
            if opening.value == ';':
                result.append(Module(name, item_start, opening.end, path, span, None, attrs))
                i += 3
            elif opening.value == '{':
                end = stream[matching[i + 2]]
                result.append(Module(name, item_start, end.end, path, span,
                                     (opening.end, end.start), attrs))
                i = matching[i + 2] + 1
            else:
                raise ValueError('unsupported module item: ' + name)
            attrs, attr_start = '', None
            continue
        attrs, attr_start = '', None
        if token.value in ('{', '[', '('):
            i = matching[i] + 1
        else:
            i += 1
    return result


@lru_cache(maxsize=512)
def code(text):
    """Identifiers and punctuation only: comments and string contents are not imports."""
    return ' '.join(t.value for t in tokens(text) if t.kind not in ('string', 'char'))


def crate_uses(text, aliases):
    stream = tokens(text)
    used = set()
    for i, token in enumerate(stream):
        if token.kind != 'ident' or token.value not in aliases:
            continue
        following = [t.value for t in stream[i + 1:i + 3]]
        preceding = [t.value for t in stream[max(0, i - 2):i]]
        if following == [':', ':'] or (preceding and preceding[-1] in ('use', 'crate')):
            used.add(token.value)
    return used


def replace_spans(text, changes):
    previous = len(text)
    for start, end, replacement in sorted(changes, reverse=True):
        if end > previous or not 0 <= start <= end <= len(text):
            raise ValueError('overlapping or invalid source edits')
        text = text[:start] + replacement + text[end:]
        previous = start
    return text
