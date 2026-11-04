"""Item declarations of one library module tree and the names each item's text mentions.

A declaration is a top-level `struct`, `enum`, `union`, `type` alias, `trait`, `const` or `fn`
of a production module, with its attributes and the `//` / `///` comment lines directly above it.
Its mentions are the paths, capitalised names and called names it uses (for a function, also
its body's `use` leaves and the values naming a function, const or static), resolved in its
defining module's scope the way `module_graph` resolves references: through imports, globs and
rooted paths to the defining item. Generic parameters, `Self`, primitives and field names are
never mentions. `mention_paths` and `function_mentions` also read impl blocks.
"""
from dataclasses import dataclass, field
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parent.parent / 'test_topology'))

from rust_source import pairs, tokens  # noqa: E402
from src_families import test_gated  # noqa: E402
from src_occurrences import scan  # noqa: E402
from src_paths import resolve  # noqa: E402
from src_scope import scope_items  # noqa: E402

TYPE_KEYWORDS = ('struct', 'enum', 'union', 'type', 'trait')
NOT_MENTIONS = {'Self', 'self', 'super', 'crate', 'dyn', 'impl', 'where', 'for', 'mut', 'const',
                'pub', 'in', 'as', 'fn', 'unsafe', 'extern', 'struct', 'enum', 'union', 'type',
                'trait', 'static', 'async', 'move', 'ref', 'let', 'if', 'else', 'match', 'return'}


@dataclass
class Declaration:
    module: tuple
    name: str
    keyword: str
    visibility: str
    source: Path
    start: int          # absolute offset in `source`, leading comments included
    end: int
    attributes: str
    text: str
    generics: frozenset = frozenset()
    mentions: list = field(default_factory=list)   # [(segments, start, end) in text]

    @property
    def key(self):
        return self.module, self.name


def comment_start(text, start):
    """Extend an item start over the `//` and `///` lines directly above it (not `//!`), and
    over its own line's indentation."""
    line_start = text.rfind('\n', 0, start) + 1
    if not text[line_start:start].strip():
        start = line_start
    while start:
        line_start = text.rfind('\n', 0, start - 1) + 1
        line = text[line_start:start].strip()
        if line.startswith('//') and not line.startswith('//!'):
            start = line_start
        else:
            break
    return start


def generic_parameters(stream, matching, i):
    """Names a generic parameter list opening at stream[i] (`<`) declares; lifetimes excluded."""
    names, depth, expect = set(), 0, True
    j = i
    while j < len(stream):
        value = stream[j].value
        if value in ('(', '['):
            j = matching[j] + 1
            continue
        if value == '<':
            depth += 1
            expect = depth == 1
        elif value == '>':
            depth -= 1
            if depth == 0:
                break
        elif value == ',' and depth == 1:
            expect = True
        elif depth == 1 and expect and stream[j].kind == 'ident':
            if value == 'const' and j + 1 < len(stream):
                names.add(stream[j + 1].value)
                j += 1
            elif not (j and stream[j - 1].value == "'"):
                names.add(value)
            expect = False
        elif value == "'" and depth == 1:
            expect = False
        j += 1
    return names


def attribute_tokens(stream, matching):
    """Indices of tokens inside `#[...]` / `#![...]` attributes."""
    inside = set()
    for i, token in enumerate(stream[:-1]):
        if token.value == '#':
            bracket = i + 2 if stream[i + 1].value == '!' else i + 1
            if bracket < len(stream) and stream[bracket].value == '[' and bracket in matching:
                inside.update(range(i, matching[bracket] + 1))
    return inside


def mention_paths(text, own_name, generics):
    """(segments, start, end) for each path the text uses: multi-segment paths, capitalised
    names and called (`name(` / `name!`) names; called names inside attributes are not calls."""
    stream = tokens(text)
    attributes = attribute_tokens(stream, pairs(stream))
    result, i = [], 0
    while i < len(stream):
        token = stream[i]
        if token.kind != 'ident' or token.value in NOT_MENTIONS and token.value not in ('crate', 'super', 'self'):
            i += 1
            continue
        after_colons = i >= 2 and stream[i - 1].value == ':' and stream[i - 2].value == ':'
        after_dot = i and stream[i - 1].value == '.'
        lifetime = i and stream[i - 1].value == "'"
        declared = i and stream[i - 1].value in ('fn', 'struct', 'enum', 'union', 'type', 'trait', 'const')
        if after_colons or after_dot or lifetime or declared:
            i += 1
            continue
        segments, j = [token.value], i
        while j + 3 < len(stream) and stream[j + 1].value == ':' and stream[j + 2].value == ':' \
                and stream[j + 3].kind == 'ident':
            segments.append(stream[j + 3].value)
            j += 3
        i = j + 1
        if len(segments) == 1:
            name = segments[0]
            turbofish = i + 2 < len(stream) and stream[i].value == ':' and stream[i + 1].value == ':' \
                and stream[i + 2].value == '<'
            called = (i < len(stream) and stream[i].value in ('!', '(') or turbofish) and j not in attributes
            if name in generics or name == own_name or name in NOT_MENTIONS:
                continue
            if not (name[0].isupper() or called):
                continue
        elif segments[0] in generics or segments[0] == 'Self':
            continue
        result.append((tuple(segments), token.start, stream[j].end))
    return result


def split_fields(stream, matching, opening, close):
    """[start, end) token ranges of the comma-separated fields between two delimiters."""
    ranges, start, depth, k = [], opening + 1, 0, opening + 1
    while k < close:
        value = stream[k].value
        if value in ('(', '[', '{'):
            k = matching[k] + 1
            continue
        if value == '<':
            depth += 1
        elif value == '>' and stream[k - 1].value != '-':
            depth -= 1
        elif value == ',' and depth == 0:
            ranges.append((start, k))
            start = k + 1
        k += 1
    if start < close:
        ranges.append((start, close))
    return ranges


def variant_names(text):
    """Token starts of the variant names an enum declaration declares."""
    stream = tokens(text)
    matching = pairs(stream)
    opening = next((k for k, t in enumerate(stream) if t.value == '{'), None)
    if opening is None:
        return set()
    starts = set()
    for start, end in split_fields(stream, matching, opening, matching[opening]):
        k = start
        while k < end and stream[k].value == '#':
            k = matching[k + 1] + 1
        if k < end:
            starts.add(stream[k].start)
    return starts


def value_names(text, own_name, generics):
    """(name, start, end) for lowercase names a function body uses as values (`.map_err(f)`),
    not after `.`, `::`, `let`, `fn`, `mut` or a lifetime tick, and not before `:` or `(`."""
    stream = tokens(text)
    result = []
    for i, token in enumerate(stream):
        if token.kind != 'ident' or not token.value[0].islower() or token.value in NOT_MENTIONS \
                or token.value in generics or token.value == own_name:
            continue
        before = stream[i - 1].value if i else ''
        after = stream[i + 1].value if i + 1 < len(stream) else ''
        after_after = stream[i + 2].value if i + 2 < len(stream) else ''
        if before in ('.', "'", 'let', 'fn', 'mut', 'ref') or (before == ':' and i >= 2 and stream[i - 2].value == ':'):
            continue
        if after == '(' or after == '!' or (after == ':' and after_after != ':') or (after == ':' and after_after == ':'):
            continue
        result.append(((token.value,), token.start, token.end))
    return result


def function_mentions(model, module, body, name, generics, mentions):
    """A function's mentions: its paths outside `use` items, every leaf of the `use` items in
    its body, and the lowercase values that name a function, const or static in its module."""
    uses, _paths = scan(body, ())
    spans = [(u.start, u.end) for u in uses]
    kept = [m for m in mentions if not any(a <= m[1] < b for a, b in spans)]
    leaves = [(tuple(segments), use.start, use.end) for use in uses
              for segments, glob, _alias in use.leaves if segments and not glob]
    values = []
    for value, start, end in value_names(body, name, generics):
        if any(a <= start < b for a, b in spans):
            continue
        found = lookup(model, module, value)
        if found is not None and found[0] == 'item':
            kind = model.surface.scopes.get(found[1][0], None)
            if kind is not None and kind.kinds.get(found[1][1]) in ('fn', 'const', 'static'):
                values.append((value, start, end))
    return kept + leaves + values


def declarations(model, parent, keywords=TYPE_KEYWORDS + ('const',)):
    """Every production declaration of `keywords` in `parent`'s descendants (the parent root
    excluded); consts are kept because a moved type may need one in an array length."""
    result = []
    for names in sorted(model.views):
        if names[:len(parent)] != parent or len(names) == len(parent):
            continue
        node = model.index[names]
        text = node.text
        for attributes, visibility, keyword, name, start, end in scope_items(text):
            if keyword not in keywords or test_gated(attributes):
                continue
            begin = comment_start(text, start)
            body = text[begin:end]
            stream = tokens(text[start:end])
            matching = pairs(stream)
            position = next(k for k, t in enumerate(stream) if t.value == name and k and
                            stream[k - 1].value == keyword)
            generics = frozenset(generic_parameters(stream, matching, position + 1)
                                 if position + 1 < len(stream) and stream[position + 1].value == '<'
                                 else ())
            declaration = Declaration(names, name, keyword, visibility, node.source,
                                      node.offset + begin, node.offset + end, attributes, body,
                                      generics)
            declaration.mentions = mention_paths(body, name, generics)
            if keyword == 'enum':
                variants = variant_names(body)
                declaration.mentions = [m for m in declaration.mentions if m[1] not in variants]
            if keyword == 'fn':
                declaration.mentions = function_mentions(model, names, body, name, generics,
                                                         declaration.mentions)
            result.append(declaration)
    return result


def lookup(model, module, segments):
    """('item', (module, name)), ('module', module), ('external', path) or None for one path."""
    surface, index = model.surface, model.index
    head = segments[0]
    if head in ('crate', 'super', 'self') or module + (head,) in index:
        resolved = resolve(segments, module, index)
        if resolved is None:
            return None
        target, rest = resolved
        if not rest:
            return 'module', target
        found = surface.origin(target, rest[0])[0]
        return classify(found)
    found = surface.origin(module, head)[0]
    if found is None:
        return None
    if found[0] == '<external>':
        # The surface reads an import through an aliased module (`use crate::a as m; use m::X;`)
        # as external: follow the alias.
        segments_in = surface.scopes[module].imports.get(head) if module in surface.scopes else None
        if segments_in and segments_in[0] not in ('crate', 'super', 'self') and \
                not segments_in[0] in ('std', 'core', 'alloc') and segments_in[0] != head:
            alias = lookup(model, module, (segments_in[0],))
            if alias is not None and alias[0] == 'module':
                rest = tuple(segments_in[1:]) + tuple(segments[1:])
                return lookup(model, alias[1], ('self',) + rest) if rest else alias
        return 'external', found[1] + ('::' + '::'.join(segments[1:]) if len(segments) > 1 else '')
    if found[1] is None and len(segments) > 1:
        resolved = resolve(('self',) + tuple(segments[1:]), found[0], index)
        if resolved is None:
            return None
        target, rest = resolved
        if not rest:
            return 'module', target
        return classify(surface.origin(target, rest[0])[0])
    return classify(found)


def absolute_route(model, module, segments):
    """`segments` as written in `module`, rooted at `crate::` (or the external path it names)."""
    head = segments[0]
    if head in ('crate', 'super', 'self') or module + (head,) in model.index:
        resolved = resolve(segments, module, model.index)
        if resolved is None:
            return '::'.join(segments)
        return 'crate::' + '::'.join(resolved[0] + resolved[1])
    found = model.surface.origin(module, head)[0] if module in model.surface.scopes else None
    if found is None:
        return '::'.join(segments)
    if found[0] == '<external>':
        return '::'.join((found[1],) + tuple(segments[1:]))
    if found[1] is None:
        return 'crate::' + '::'.join(found[0] + tuple(segments[1:]))
    return 'crate::' + '::'.join(found[0] + (found[1],) + tuple(segments[1:]))


def classify(found):
    if found is None:
        return None
    if found[0] == '<external>':
        return 'external', found[1]
    if found[1] is None:
        return 'module', found[0]
    return 'item', found
