"""Lossless Rust call argument spans used by the test-cost codemods."""
from dataclasses import dataclass
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'test_topology'))
from rust_source import pairs, tokens


@dataclass(frozen=True)
class Call:
    name: str
    start: int
    name_end: int
    end: int
    arguments: tuple


def calls(text, names):
    stream = tokens(text)
    matching = pairs(stream)
    result = []
    for index, token in enumerate(stream[:-1]):
        if token.value not in names or stream[index + 1].value != '(':
            continue
        if index and stream[index - 1].value == 'fn':
            continue
        opening, closing = index + 1, matching[index + 1]
        arguments, first, current = [], opening + 1, opening + 1
        while current < closing:
            if stream[current].value == ',':
                if first < current:
                    arguments.append((stream[first].start, stream[current - 1].end))
                first = current + 1
            if stream[current].value in ('(', '[', '{'):
                current = matching[current] + 1
            else:
                current += 1
        if first < closing:
            arguments.append((stream[first].start, stream[closing - 1].end))
        result.append(Call(token.value, token.start, token.end, stream[closing].end, tuple(arguments)))
    return result


def arguments(text, call):
    return [text[start:end] for start, end in call.arguments]
