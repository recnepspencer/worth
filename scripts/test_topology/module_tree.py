"""Resolve a target's Rust module tree with source provenance and cfg retained."""
from dataclasses import dataclass, field
from functools import lru_cache
from pathlib import Path
import re

from rust_source import code, modules, tokens, string_value, pairs, replace_spans


@dataclass
class Node:
    source: Path
    names: tuple
    text: str
    children: list = field(default_factory=list)
    declaration: object = None
    offset: int = 0
    _own: str | None = field(default=None, repr=False)

    def walk(self):
        yield self
        for child in self.children:
            yield from child.walk()

    def own_text(self):
        if self._own is not None:
            return self._own
        result, pos = [], 0
        for child in self.children:
            item = child.declaration
            result.append(self.text[pos:item.start])
            pos = item.end
        result.append(self.text[pos:])
        self._own = ''.join(result)
        return self._own


def read_tree(source, names=(), active=(), declaration=None, text=None, offset=0,
              module_directory=None, depth=0):
    source = source.resolve()
    if depth > 100:
        raise ValueError('module recursion at ' + str(source))
    text = source.read_text(encoding='utf-8') if text is None else text
    node = Node(source, names, text, declaration=declaration, offset=offset)
    ordinary = module_directory or (source.parent if not names or source.name == 'mod.rs'
                                     else source.with_suffix(''))
    path_base = module_directory or source.parent
    for item in modules(text):
        if item.body:
            start, end = item.body
            child = read_tree(source, names + (item.name,), active, item,
                              text[start:end], offset + start, ordinary / item.name, depth + 1)
        else:
            if item.path:
                target = path_base / item.path
            else:
                options = [ordinary / (item.name + '.rs'), ordinary / item.name / 'mod.rs']
                found = [p for p in options if p.is_file()]
                if len(found) != 1:
                    raise ValueError(f'{source}: module {item.name} has {len(found)} source candidates')
                target = found[0]
            target = target.resolve()
            if target in active:
                raise ValueError('module cycle at ' + str(target))
            child = read_tree(target, names + (item.name,), active + (source,), item,
                              depth=depth + 1)
        node.children.append(child)
    return node


@lru_cache(maxsize=4096)
def test_names(text):
    # cfg_attr, rstest and async test attributes retain their original text.
    return re.findall(r'#\s*\[(?:[^\[\]]|\[[^\]]*\])*\b(?:test|rstest)\b[^\]]*\]'
                      r'(?:\s*#\[[^\]]*\])*\s*(?:pub(?:\([^)]*\))?\s+)?'
                      r'(?:async\s+)?fn\s+(\w+)', text)


def without_tests(text):
    """Retain ancestor context without copying its tests into descendant courts."""
    names = set(test_names(text))
    if not names:
        return text
    stream = tokens(text)
    matching = pairs(stream)
    changes, boundary, i = [], 0, 0
    while i < len(stream):
        token = stream[i]
        if token.value == 'fn' and i + 1 < len(stream) and stream[i + 1].value in names:
            body = next(j for j in range(i + 2, len(stream)) if stream[j].value in ('{', ';'))
            if stream[body].value == '{':
                end = matching[body]
                changes.append((boundary, stream[end].end, ''))
                i, boundary = end + 1, stream[end].end
                continue
        if token.value in ('{', '[', '('):
            # Attributes belong to the following item, not the previous one.
            is_attribute = token.value == '[' and i and stream[i - 1].value == '#'
            end = matching[i]
            if token.value == '{' and not is_attribute:
                boundary = stream[end].end
            i = end + 1
        else:
            if token.value == ';':
                boundary = token.end
            i += 1
    return replace_spans(text, changes)


def source_files(node):
    return {n.source for n in node.walk()}


def tree_text(node):
    return '\n'.join(path.read_text(encoding='utf-8') for path in sorted(source_files(node)))


def literal_references(node, package_root):
    """Locate literal filesystem consumers. Dynamic operands remain explicit work."""
    result = []
    stream = tokens(node.text)
    for i, token in enumerate(stream):
        if token.value in ('include', 'include_str', 'include_bytes') and i + 3 < len(stream):
            if [t.value for t in stream[i + 1:i + 3]] != ['!', '(']:
                continue
            value = stream[i + 3]
            if value.kind == 'string':
                result.append({'kind': token.value, 'path': (node.source.parent / string_value(value)).resolve(),
                               'span': (node.offset + value.start, node.offset + value.end)})
    # Literal manifest-relative operands cover join(), Path::new(), concat! and trybuild globs.
    manifest_reads = 'CARGO_MANIFEST_DIR' in node.text
    for i, token in enumerate(stream):
        if token.kind != 'string':
            continue
        value = string_value(token)
        if value.startswith(('tests/', '/tests/')):
            path = package_root / value.lstrip('/')
            if path.exists() or '*' in value:
                result.append({'kind': 'manifest-path' if manifest_reads else 'test-path',
                               'path': path.resolve(), 'span': (node.offset + token.start, node.offset + token.end),
                               'leading_slash': value.startswith('/')})
    return result


def private_context(node):
    value = code(node.own_text())
    return {'crate_paths': sorted(set(re.findall(r'\bcrate\s+:\s+:\s+(\w+)', value))),
            'parent_paths': bool(re.search(r'\bsuper\s+:\s+:', value)),
            'restricted_visibility': bool(re.search(r'pub\s*\(\s*(?:crate|super)', value))}
