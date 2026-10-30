"""Pool explicitly audited independent unit-returning for-case loops."""
import argparse
from pathlib import Path

from rust_calls import calls
from workspace import repository_root
from filesystem_path import filesystem_path
from rust_source import pairs, replace_spans, tokens


class SharedCaseState(ValueError):
    pass


def rewrite(text, helper):
    stream, changes, selected = tokens(text), [], []
    matching = pairs(stream)
    pooled = [(call.start, call.end) for call in calls(text, {'run'})
              if text[max(0, call.start - 40):call.start].rstrip().endswith('sweep::')]
    protected_until = -1
    for index, token in enumerate(stream):
        if token.value != 'for' or token.start < protected_until or any(
                start < token.start < end for start, end in pooled):
            continue
        if index + 3 >= len(stream) or stream[index + 2].value != 'in':
            continue
        variable = stream[index + 1].value
        current = index + 3
        while current < len(stream) and stream[current].value != '{':
            if stream[current].value in ('(', '['):
                current = matching[current] + 1
            else:
                current += 1
        if current >= len(stream):
            raise ValueError('for-case loop has no body')
        closing = matching[current]
        body = text[stream[current].end:stream[closing].start]
        if not calls(body, {helper}):
            continue
        body_tokens = [token.value for token in tokens(body)]
        if '&mut' in ''.join(body_tokens) or any(value in ('+=', '-=') for value in body_tokens) or any(
                body_tokens[i:i + 2] in (['+', '='], ['-', '=']) for i in range(len(body_tokens) - 1)):
            raise SharedCaseState('shared mutable case state requires a returned-witness reduction')
        expression = text[stream[index + 3].start:stream[current].start].strip()
        replacement = ('let _: Vec<()> = worth_store_test_support::sweep::run(\n'
                       f'    IntoIterator::into_iter({expression}).collect(),\n'
                       f'    |{variable}| {{{body}}},\n);')
        changes.append((token.start, stream[closing].end, replacement))
        selected.append({'line': text[:token.start].count('\n') + 1, 'case': variable,
                         'inputs': expression, 'helper': helper})
        protected_until = stream[closing].end
    return replace_spans(text, changes), selected


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=Path, required=True)
    parser.add_argument('--helper', required=True)
    parser.add_argument('--apply', action='store_true')
    args = parser.parse_args()
    root = repository_root(Path.cwd())
    source = args.source.resolve()
    source.relative_to(root)
    text = filesystem_path(source).read_text(encoding='utf-8')
    changed, selected = rewrite(text, args.helper)
    for row in selected:
        print(row)
    if args.apply:
        filesystem_path(source).write_text(changed, encoding='utf-8')


if __name__ == '__main__':
    main()
