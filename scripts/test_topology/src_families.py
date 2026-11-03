"""Inventory the `cfg(test)` families compiled into a library target's own source."""
from dataclasses import dataclass, field
import hashlib
import re

from filesystem_path import filesystem_path
from module_tree import read_tree, test_names
from rust_source import pairs, tokens

TEST_ATTRIBUTE = re.compile(r'#\s*\[\s*(?:[\w:]+::)?(?:test|rstest|tokio::test)\b')


def predicate_value(stream, start, end, test):
    """Three-valued cfg evaluation: True, False, or None for predicates other than `test`."""
    values, i = [], start
    matching = pairs(stream[start:end]) if start < end else {}
    while i < end:
        word = stream[i].value
        following = stream[i + 1].value if i + 1 < end else ''
        if word in ('all', 'any', 'not') and following == '(':
            close = matching[i + 1 - start] + start
            inner = predicate_list(stream, i + 2, close, test)
            if word == 'not':
                value = None if inner[0] is None else not inner[0]
            elif word == 'all':
                value = False if False in inner else (None if None in inner else True)
            else:
                value = True if True in inner else (None if None in inner else False)
            values.append(value)
            i = close + 1
        elif stream[i].kind == 'ident' and following != '=':
            values.append(test if word == 'test' else None)
            i += 1
        elif stream[i].kind == 'ident':
            values.append(None)
            i += 3
        else:
            i += 1
    return values


def predicate_list(stream, start, end, test):
    return predicate_value(stream, start, end, test) or [True]


def test_gated(attributes):
    """A cfg that is false without `test` and not false with it gates code to unit tests."""
    stream = tokens(attributes)
    matching = pairs(stream)
    for i, token in enumerate(stream):
        if token.value == 'cfg' and i + 1 < len(stream) and stream[i + 1].value == '(':
            close = matching[i + 1]
            if predicate_list(stream, i + 2, close, False)[0] is False and \
                    predicate_list(stream, i + 2, close, True)[0] is not False:
                return True
    return False


@dataclass
class Family:
    node: object
    names: tuple
    files: list
    lines: int
    tests: list = field(default_factory=list)
    macro_tests: list = field(default_factory=list)


def function_bodies(text):
    """Test-function name and canonical token hash, so whitespace moves never change identity."""
    names = set(test_names(text))
    stream = tokens(text)
    matching = pairs(stream)
    result = []
    for i, token in enumerate(stream):
        if token.value == 'fn' and i + 1 < len(stream) and stream[i + 1].value in names:
            body = next((j for j in range(i + 2, len(stream)) if stream[j].value in ('{', ';')), None)
            if body is None or stream[body].value != '{':
                continue
            canonical = ' '.join(t.value for t in stream[i:matching[body] + 1])
            result.append((stream[i + 1].value, hashlib.sha256(canonical.encode()).hexdigest()[:16]))
    return result


def macro_generated(text):
    """Local `macro_rules!` that emit `#[test]` items; their names need expansion to know."""
    stream = tokens(text)
    matching = pairs(stream)
    result = []
    for i, token in enumerate(stream):
        if token.value == 'macro_rules' and i + 3 < len(stream) and stream[i + 1].value == '!':
            opening = i + 3
            if stream[opening].value not in ('{', '(', '['):
                continue
            body = text[stream[opening].start:stream[matching[opening]].end]
            if TEST_ATTRIBUTE.search(body):
                name = stream[i + 2].value
                calls = sum(1 for j, t in enumerate(stream)
                            if t.value == name and j + 1 < len(stream) and stream[j + 1].value == '!'
                            and (j == 0 or stream[j - 1].value != 'macro_rules'))
                result.append({'macro': name, 'invocations': calls})
    return result


def node_lines(node):
    return node.text.count('\n') + (0 if node.text.endswith('\n') or not node.text else 1)


def families(lib_source):
    """Outermost test-gated modules; nested gated modules belong to their outer family."""
    tree = read_tree(lib_source)
    result = []

    def visit(node):
        declaration = node.declaration
        if declaration is not None and test_gated(declaration.attributes):
            result.append(family(node))
            return
        for child in node.children:
            visit(child)

    visit(tree)
    return tree, result


def family(node):
    files, lines, tests, macros = [], 0, [], []
    for member in node.walk():
        own = member.own_text()
        tests.extend(function_bodies(own))
        macros.extend(macro_generated(own))
        if member.declaration is not None and member.declaration.body is None:
            files.append(member.source)
            lines += node_lines(member)
    if node.declaration.body is not None:
        lines += node.text.count('\n') + 1
    return Family(node, node.names, sorted(set(files)), lines, sorted(tests), macros)


def file_text(path):
    return filesystem_path(path).read_text(encoding='utf-8')
