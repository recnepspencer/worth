#!/usr/bin/env python3
"""Audit the consumer facade: public inherent methods of a runtime type and what each delegates to.

Usage:
    python scripts/query_split/facade_audit.py <lib.rs> [--type WorthQueryPrimaryGraphApplicationRuntime]
        [--module domain_computation::primary_graph::application_runtime] [--root <repo>] [--out <dir>]

Every public inherent method of the type defined anywhere under the module (test-gated families,
impls and methods excluded) is listed with the calls in its body (extension-trait calls, method
calls with their receiver, path calls) and a verdict. `thin` means the body only calls, reads
fields and returns; `logic` lists the reasons (branches, loops, closures, operators, casts, macros).
Without `--out` the Markdown goes to stdout. Output is sorted and holds no absolute paths.
"""
import argparse
import json
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parent.parent / 'test_topology'))

from rust_source import pairs, tokens  # noqa: E402
from src_families import families, test_gated  # noqa: E402
from src_scope import scope_items  # noqa: E402

TYPE = 'WorthQueryPrimaryGraphApplicationRuntime'
MODULE = 'domain_computation::primary_graph::application_runtime'
BRANCHING = {'if', 'match', 'loop', 'while', 'for', 'break', 'continue', 'unsafe'}
PLUMBING = {'Ok', 'Some', 'Err', 'clone', 'into', 'from', 'as_ref', 'as_deref', 'map_err', 'map'}
KEYWORDS = {'let', 'match', 'if', 'while', 'for', 'in', 'return', 'else', 'mut', 'move', 'ref'}
OPERAND_END = {')', ']'}
SINGLE_OPERATORS = {'+', '-', '/', '%', '^'}


def impl_self_type(stream, matching, i):
    """(self type name, is_trait_impl, index of the body brace) for the impl keyword at `i`."""
    j = i + 1
    if stream[j].value == '<':
        depth = 0
        while True:
            depth += {'<': 1, '>': -1}.get(stream[j].value, 0) if stream[j - 1].value != '-' else 0
            j += 1
            if depth == 0:
                break
    name, trait, angle = None, False, 0
    while stream[j].value != '{':
        value = stream[j].value
        if value in ('(', '['):
            j = matching[j]
        elif value == '<':
            angle += 1
        elif value == '>' and stream[j - 1].value != '-':
            angle -= 1
        elif value == 'where' and angle == 0:
            break
        elif angle == 0 and value == 'for' and stream[j + 1].value != '<':
            trait, name = True, None
        elif angle == 0 and stream[j].kind == 'ident':
            name = value
        j += 1
    while stream[j].value != '{':
        j = matching[j] + 1 if stream[j].value in ('(', '[') else j + 1
    return name, trait, j


def inherent_impls(text):
    """(self type name, body text) for each inherent impl at this scope."""
    result = []
    for attributes, _visibility, keyword, _name, start, end in scope_items(text):
        if keyword != 'impl' or test_gated(attributes):
            continue
        stream = tokens(text[start:end])
        matching = pairs(stream)
        index = next(k for k, t in enumerate(stream) if t.value == 'impl')
        name, trait, brace = impl_self_type(stream, matching, index)
        if not trait and name:
            body = text[start:end]
            result.append((name, body[stream[brace].end:stream[matching[brace]].start]))
    return result


def body_range(stream, matching):
    """Token range of a function body, or None for a declaration without one."""
    i = 0
    while i < len(stream) and stream[i].value not in ('{', ';'):
        i = matching[i] + 1 if stream[i].value in ('(', '[') else i + 1
    if i >= len(stream) or stream[i].value == ';':
        return None
    return i + 1, matching[i]


def qualified_path_end(stream, i):
    """Index after the `>` closing a `<T as Trait>` qualified path that starts at `i`."""
    depth, j = 0, i
    while j < len(stream):
        depth += {'<': 1, '>': -1}.get(stream[j].value, 0) if stream[j - 1].value != '-' else 0
        j += 1
        if depth == 0:
            return j
    return j


def receiver_text(stream, i, start):
    """The receiver chain before the `.` at token index `i`, with call and index arguments elided."""
    parts, j = [], i - 1
    while j >= start:
        value = stream[j].value
        if value in (')', ']'):
            depth, k = 0, j
            while k >= start:
                depth += 1 if stream[k].value in (')', ']') else -1 if stream[k].value in ('(', '[') else 0
                if depth == 0:
                    break
                k -= 1
            parts.append('()' if value == ')' else '[]')
            j = k - 1
        elif (stream[j].kind == 'ident' and value not in KEYWORDS) or value in ('.', '?') or (
                value == ':' and ':' in (stream[j - 1].value, stream[j + 1].value if j + 1 < i else '')):
            parts.append(value)
            j -= 1
        else:
            break
    return ''.join(reversed(parts))


def analyse(text, stream, start, end):
    """(calls, logic reasons) for the body tokens in [start, end)."""
    calls, reasons, i = [], set(), start
    while i < end:
        token, value = stream[i], stream[i].value
        previous = stream[i - 1].value if i > start else ''
        following = stream[i + 1].value if i + 1 < end else ''
        adjacent = i + 1 < end and stream[i + 1].start == token.end
        if token.kind == 'ident' and value in BRANCHING:
            reasons.add(value)
        elif token.kind == 'ident' and value == 'as':
            reasons.add('cast')
        elif token.kind == 'ident' and following == '!' and adjacent and \
                i + 2 < end and stream[i + 2].value in '([{':
            reasons.add(f'macro {value}!')
            i += 1
        elif value == '<' and (i == start or previous in ('(', ',', '=', '{', ';', '&', '!')
                               or previous == 'return') and following != '=':
            stop = qualified_path_end(stream, i)
            if stop < end and stream[stop].value == ':' and stream[stop + 1].value == ':':
                path = [t.value for t in stream[i + 1:stop - 1]]
                trait = path[path.index('as') + 1:] if 'as' in path else path
                trait = trait[:trait.index('<')] if '<' in trait else trait
                trait_path = ''.join(trait)
                names = [v for v in trait if v.isidentifier()]
                if stop + 2 < end and stream[stop + 2].kind == 'ident':
                    calls.append({'kind': 'trait', 'name': stream[stop + 2].value,
                                  'target': trait_path, 'trait': names[-1] if names else ''})
                i = stop + 3
                continue
        elif token.kind == 'ident' and following == '(' and value not in KEYWORDS:
            if previous == '.':
                calls.append({'kind': 'method', 'name': value,
                              'target': receiver_text(stream, i - 1, start)})
            elif previous not in (':',) or stream[i - 2].value != ':':
                calls.append({'kind': 'path', 'name': value, 'target': ''})
        elif token.kind == 'ident' and following == ':' and i + 2 < end and stream[i + 2].value == ':':
            j = i
            while j + 3 < end and stream[j + 1].value == ':' and stream[j + 2].value == ':' \
                    and stream[j + 3].kind == 'ident':
                j += 3
            if j + 1 < end and stream[j + 1].value == '(' and j > i:
                path = ''.join(stream[k].value for k in range(i, j - 2))
                calls.append({'kind': 'path', 'name': stream[j].value, 'target': path})
                i = j + 1
                continue
        elif value == '|':
            reasons.add('closure')
        elif value == '=' and adjacent and following == '=':
            reasons.add('operator ==')
        elif value in ('!', '<', '>') and adjacent and following == '=':
            reasons.add(f'operator {value}=')
        elif value == '!':
            reasons.add('operator !')
        elif value == '&' and adjacent and following == '&' and i > start and (
                stream[i - 1].kind == 'ident' or previous in OPERAND_END):
            reasons.add('operator &&')
        elif value in SINGLE_OPERATORS and not (value == '-' and following == '>' and adjacent):
            reasons.add(f'operator {value}')
        elif value == '*' and i > start and (stream[i - 1].kind == 'ident' or previous in OPERAND_END):
            reasons.add('operator *')
        i += 1
    return calls, sorted(reasons)


def delegates(calls):
    return [c for c in calls if c['name'] not in PLUMBING]


def audit_method(source, owner, visibility, name, text):
    stream = tokens(text)
    matching = pairs(stream)
    found = body_range(stream, matching)
    calls, reasons = analyse(text, stream, *found) if found else ([], ['no body'])
    unique = sorted({(c['kind'], c['target'], c['name'], c.get('trait', '')) for c in calls})
    rows = [{'kind': k, 'target': t, 'name': n, **({'trait': tr} if tr else {})}
            for k, t, n, tr in unique]
    return {'method': name, 'visibility': visibility, 'file': source, 'module': owner,
            'calls': rows, 'delegates': [r for r in rows if r['name'] not in PLUMBING],
            'verdict': 'logic' if reasons else 'thin', 'logic': reasons}


def audit(lib, type_name, module, root):
    tree, family_list = families(Path(lib))
    root = Path(root).resolve()
    skipped = [f.names for f in family_list]
    start = tuple(module.split('::'))
    methods = []
    for node in tree.walk():
        if node.names[:len(start)] != start or any(node.names[:len(s)] == s for s in skipped):
            continue
        text = node.own_text()
        for name, body in inherent_impls(text):
            if name != type_name:
                continue
            for attributes, visibility, keyword, fn_name, begin, finish in scope_items(body):
                if keyword == 'fn' and visibility.startswith('pub') and not test_gated(attributes):
                    methods.append(audit_method(node.source.relative_to(root).as_posix(),
                                                '::'.join(node.names), visibility, fn_name,
                                                body[begin:finish]))
    methods.sort(key=lambda m: (m['method'], m['file']))
    logic = [m['method'] for m in methods if m['verdict'] == 'logic']
    return {'version': 1, 'type': type_name, 'module': module, 'methods': methods,
            'public_methods': len(methods), 'thin': len(methods) - len(logic),
            'logic_methods': sorted(logic)}


def markdown(result):
    lines = [f"# Facade audit: `{result['type']}`", '',
             f"- module: `{result['module']}`", f"- public inherent methods: {result['public_methods']}",
             f"- thin composition: {result['thin']}", f"- containing logic: {len(result['logic_methods'])}",
             '', '| method | verdict | delegates | logic |', '| --- | --- | --- | --- |']
    for m in result['methods']:
        targets = ', '.join(sorted({(f"{d['trait']}::" if d['kind'] == 'trait' else
                                     f"{d['target']}." if d['kind'] == 'method' and d['target'] else
                                     f"{d['target']}::" if d['target'] else '') + d['name']
                                    for d in m['delegates']})) or '(field reads only)'
        lines.append(f"| `{m['method']}` | {m['verdict']} | {targets} | {', '.join(m['logic'])} |")
    return '\n'.join(lines) + '\n'


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('lib', type=Path)
    parser.add_argument('--type', default=TYPE)
    parser.add_argument('--module', default=MODULE)
    parser.add_argument('--root', type=Path, default=Path.cwd())
    parser.add_argument('--out', type=Path)
    args = parser.parse_args(argv)
    result = audit(args.lib, args.type, args.module, args.root)
    if args.out is None:
        sys.stdout.write(markdown(result))
    else:
        args.out.mkdir(parents=True, exist_ok=True)
        (args.out / 'facade_audit.json').write_text(json.dumps(result, indent=1, sort_keys=True) + '\n',
                                                    encoding='utf-8', newline='\n')
        (args.out / 'facade_audit.md').write_text(markdown(result), encoding='utf-8', newline='\n')
    print(json.dumps({k: result[k] for k in ('public_methods', 'thin')} |
                     {'logic': len(result['logic_methods'])}, sort_keys=True),
          file=sys.stderr if args.out is None else sys.stdout)
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
