"""Rewrite compiler-session calls into prebuilt denials and ordinary examples."""
from dataclasses import dataclass, field
import re

from rust_calls import arguments, calls
from rust_items import items
from rust_source import pairs, replace_spans, tokens


INITIALIZER = re.compile(r'\blet\s+(\w+)\s*=\s*trybuild\s*::\s*TestCases\s*::\s*new\s*\(\s*\)\s*;')
PARAMETER = re.compile(r'\b(\w+)\s*:\s*&\s*trybuild\s*::\s*TestCases\b')
TEMPORARY = re.compile(r'trybuild\s*::\s*TestCases\s*::\s*new\s*\(\s*\)\s*\.\s*$')


@dataclass
class Cutover:
    text: str
    pass_calls: list = field(default_factory=list)
    moved_tests: list = field(default_factory=list)
    moved_denial_tests: list = field(default_factory=list)
    parameterized_helpers: set = field(default_factory=set)
    denials: int = 0


def expression_start(text, call):
    prefix = text[:call.start]
    match = re.search(r'((?:\b\w+\s*::\s*)*\b\w+)\s*\.\s*$', prefix)
    if match:
        return match.start(), match[1]
    temporary = TEMPORARY.search(prefix)
    return (temporary.start(), '<temporary>') if temporary else (None, None)


def call_statement(text, call):
    prefix = text[:call.start]
    match = re.search(r'(?:\b\w+\s*::\s*)*$', prefix)
    start = match.start() if match else call.start
    end = call.end
    tail = re.match(r'\s*;', text[end:])
    if tail is None:
        raise ValueError('compiler session runner must be a statement')
    return start, end + tail.end()


def empty_literal_loops(text):
    stream, changes = tokens(text), []
    matching = pairs(stream)
    for index, token in enumerate(stream[:-4]):
        if token.value != 'for' or stream[index + 2].value != 'in' or stream[index + 3].value != '[':
            continue
        opening = matching[index + 3] + 1
        if stream[opening].value != '{':
            continue
        closing = matching[opening]
        if closing == opening + 1:
            changes.append((token.start, stream[closing].end, ''))
    return replace_spans(text, changes)


def rewrite(text):
    result = Cutover(text)
    edits, extra_tests = [], []
    for item in items(text):
        if item.keyword != 'fn' or not item.body:
            continue
        fragment = text[item.start:item.end]
        if item.name == 'run_compile_pass_bundle':
            edits.append((item.start, item.end, ''))
            continue
        bindings = {match[1] for match in INITIALIZER.finditer(fragment)}
        parameters = list(PARAMETER.finditer(fragment))
        bindings.update(match[1] for match in parameters)
        if parameters:
            if len(parameters) != 1:
                raise ValueError('multiple compiler-session parameters require review')
            result.parameterized_helpers.add(item.name)
        changes = [(match.start(), match.end(), '') for match in INITIALIZER.finditer(fragment)]
        changes += [(match.start(), match.end(), '') for match in parameters]
        had_pass, had_absence = False, False
        for call in calls(fragment, {'compile_fail', 'pass', 'run_compile_pass_bundle'}):
            if call.name == 'run_compile_pass_bundle':
                start, end = call_statement(fragment, call)
                changes.append((start, end, ''))
                had_pass = True
                continue
            start, receiver = expression_start(fragment, call)
            if receiver not in bindings and receiver != '<temporary>':
                continue
            args = arguments(fragment, call)
            if len(args) != 1:
                raise ValueError('compiler fixture call must take one input path')
            argument = args[0]
            if call.name == 'pass':
                end = call.end
                semicolon = re.match(r'\s*;', fragment[end:])
                if semicolon:
                    end += semicolon.end()
                changes.append((start, end, ''))
                result.pass_calls.append({'function': item.name, 'argument': argument})
                had_pass = True
            else:
                name = 'denied'
                result.denials += 1
                if 'certification_authority_is_absent.rs' in argument:
                    had_absence = True
                    attribute = re.search(r'#\s*\[\s*cfg\s*\(\s*not\s*\(\s*all\s*\(\s*\)\s*\)\s*\)\s*\]\s*$', fragment[:start])
                    if attribute:
                        start = attribute.start()
                    extra_tests.append('#[test]\nfn certification_authority_is_absent_in_the_production_environment() {\n'
                                       f'    crate::fixture_outcomes::production_denial({argument}).expect("production compiler denial");\n}}\n')
                    end = call.end
                    semicolon = re.match(r'\s*;', fragment[end:])
                    if semicolon:
                        end += semicolon.end()
                    changes.append((start, end, ''))
                else:
                    changes.append((start, call.end,
                                    f'crate::fixture_outcomes::{name}({argument}).expect("prebuilt compiler denial")'))
        changed = replace_spans(fragment, changes)
        changed = empty_literal_loops(changed)
        if had_pass:
            # Pure fixture-list construction formerly fed only to the removed
            # compiler runner. Keep lists still observed by semantic assertions.
            unused = []
            for match in re.finditer(r'\blet\s+(\w+)\s*=\s*([^;]+);', changed):
                if 'compile_pass_bundle' in match[2] and not re.search(r'\b' + match[1] + r'\b', changed[match.end():]):
                    unused.append((match.start(), match.end(), ''))
            changed = replace_spans(changed, unused)
            current = next((value for value in items(changed) if value.keyword == 'fn'), None)
            if item.test and current and not tokens(changed[current.body[0]:current.body[1]]):
                result.moved_tests.append(item.name)
                changed = ''
        elif had_absence and item.test:
            current = next((value for value in items(changed) if value.keyword == 'fn'), None)
            if current and not tokens(changed[current.body[0]:current.body[1]]):
                result.moved_denial_tests.append(item.name)
                changed = ''
        if changed != fragment:
            edits.append((item.start, item.end, changed))
    result.text = replace_spans(text, edits)
    result.text = re.sub(r'(?:\w+::)*run_compile_pass_bundle\s*,?\s*(?=[};])', '', result.text)
    result.text = re.sub(r'(?:#\[allow\(unused_imports\)\]\s*)?(?:pub\(crate\)\s+)?use\s*\{?\s*trybuild::TestCases\s*\}?\s*;', '', result.text)
    result.text += '\n' + '\n'.join(extra_tests) if extra_tests else ''
    return result
