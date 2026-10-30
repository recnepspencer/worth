"""Retain test source, lexical context and literal include ownership together."""
from collections import defaultdict, deque
from dataclasses import dataclass
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'test_topology'))
from filesystem_path import filesystem_path
from module_tree import read_tree, literal_references
from rust_items import items
from rust_source import tokens


@dataclass(frozen=True)
class TestSource:
    package: dict
    path: Path
    text: str
    context: str


def identifiers(text):
    return {token.value for token in tokens(text) if token.kind == 'ident'}


def definitions(text, offset=0):
    for item in items(text):
        yield item, text, offset
        if item.keyword in ('impl', 'trait') and item.body:
            start, end = item.body
            yield from definitions(text[start:end], offset + start)


class SourceSelection:
    def __init__(self, package):
        self.package = package
        self.root = package['manifest'].parent
        self.texts, self.contexts = {}, defaultdict(set)
        self.spans, self.edges = defaultdict(set), defaultdict(list)
        self.functions = defaultdict(list)
        self.pending, self.referenced, self.visited = deque(), set(), set()

    def selected(self, source, start, end):
        return any(left <= start and end <= right for left, right in self.spans[source])

    def mark(self, source, start, end):
        if self.selected(source, start, end):
            return
        self.spans[source].add((start, end))
        for name in identifiers(self.texts[source][start:end]) - self.referenced:
            self.referenced.add(name)
            self.pending.append(name)
        for left, right, target, target_start, target_end in self.edges[source]:
            if start <= left and right <= end:
                if target is None:
                    raise ValueError(f'unresolved include! in selected test source: {source}:{left}')
                self.mark(target, target_start, target_end)

    def edge(self, source, start, end, target, target_start=0, target_end=0):
        self.edges[source].append((start, end, target, target_start, target_end))
        if self.selected(source, start, end):
            if target is None:
                raise ValueError(f'unresolved include! in selected test source: {source}:{start}')
            self.mark(target, target_start, target_end)

    def visit(self, node, testing, ancestors=(), active_includes=(), included_source=False):
        source = node.source
        text = self.texts.setdefault(source, filesystem_path(source).read_text(encoding='utf-8-sig'))
        self.contexts[source].update((*ancestors, node.text))
        testing = testing or bool(node.declaration and 'test' in node.declaration.attributes)
        identity = (source, node.offset, len(node.text), node.names, testing)
        if identity in self.visited:
            return
        self.visited.add(identity)
        if testing:
            self.mark(source, node.offset, node.offset + len(node.text))
        try:
            declarations = list(definitions(node.text, node.offset))
        except ValueError:
            stream = tokens(node.text)
            item_words = {'pub', 'fn', 'mod', 'impl', 'trait', 'struct', 'enum', 'union', 'use',
                          'const', 'static', 'type', 'extern', 'unsafe', 'async', 'macro_rules', '#'}
            if not included_source or not stream or stream[0].value in item_words:
                raise
            # Included expressions are selected through their owning span.
            declarations = []
        for item, fragment, offset in declarations:
            start, end = offset + item.start, offset + item.end
            header = fragment[item.start:item.body[0] if item.body else item.end]
            if item.test or ('cfg' in header and 'test' in header):
                self.mark(source, start, end)
            if item.keyword == 'fn':
                self.functions[item.name].append((source, start, end))
        context = (*ancestors, node.text)
        for child in node.children:
            self.visit(child, testing, context, active_includes)
            declaration = child.declaration
            self.edge(source, node.offset + declaration.start, node.offset + declaration.end,
                      child.source, child.offset, child.offset + len(child.text))
        included_spans = set()
        for reference in literal_references(node, self.root):
            if reference['kind'] != 'include':
                continue
            start, end = reference['span']
            included_spans.add(start)
            target = reference['path']
            if target == source or target in active_includes:
                raise ValueError(f'include cycle at {target}')
            included = read_tree(target, names=node.names, module_directory=target.parent)
            self.visit(included, False, context, (*active_includes, source), included_source=True)
            self.edge(source, start, end, target, 0, len(self.texts[target]))
        stream = tokens(node.text)
        shadowed = any([part.value for part in stream[index:index + 3]] == ['macro_rules', '!', 'include']
                       for index in range(len(stream)))
        if not shadowed:
            for index, token in enumerate(stream[:-3]):
                if token.value == 'include' and [part.value for part in stream[index + 1:index + 3]] == ['!', '(']:
                    operand = stream[index + 3]
                    if node.offset + operand.start not in included_spans:
                        self.edge(source, node.offset + token.start, node.offset + operand.end, None)

    def documents(self, roots):
        for path, testing in roots:
            self.visit(read_tree(path), testing)
        # This conservative same-target closure retains ordinary named helper
        # bodies referenced by tests, without treating all production as tests.
        while self.pending:
            for source, start, end in self.functions[self.pending.popleft()]:
                self.mark(source, start, end)
        result = []
        for path, selected in sorted(self.spans.items()):
            if not selected:
                continue
            text = self.texts[path]
            masked = ['\n' if character == '\n' else ' ' for character in text]
            for start, end in selected:
                masked[start:end] = text[start:end]
            result.append(TestSource(self.package, path, ''.join(masked), '\n'.join(sorted(self.contexts[path]))))
        return result


def documents(package, roots):
    return SourceSelection(package).documents(roots)
