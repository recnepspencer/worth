"""Lexical cost sites, with source locations rather than timing-derived guesses."""
from pathlib import Path
import re
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'test_topology'))
from rust_source import tokens, string_value
from module_bindings import use_leaves

from workspace import sources


PATTERNS = {
    'cargo-command': re.compile(r'Command::new\([^;]{0,500}?(?:"cargo"|"CARGO")', re.S),
    'cargo-environment': re.compile(r'(?:env!|(?:std::)?env::var(?:_os)?)\("CARGO"\)'),
    'trybuild': re.compile(r'\btrybuild::(?:TestCases|Runner)'),
    'cargo-metadata': re.compile(r'\bcargo_metadata::|\bMetadataCommand::new\('),
    'trybuild-pass': re.compile(r'\b(?:(\w+)|trybuild::TestCases::new\(\))\.pass\('),
    'sleep': re.compile(r'\b(?:(?:std::)?thread|tokio::time|async_std::task)::sleep\('),
    'polling-loop': re.compile(r'\b(?:while\b|loop\s*\{)'),
    'fixed-timeout': re.compile(r'\b(?:Duration::from_(?:secs|millis|micros)\(\d|TIMEOUT\b)'),
}


def normalized_source(text):
    stream = tokens(text)
    normalized, offsets = '', []
    for index, token in enumerate(stream):
        value = token.value
        if token.kind == 'string':
            literal = string_value(token)
            executable = literal.replace('\\', '/').rsplit('/', 1)[-1].casefold()
            value = '"' + ('CARGO' if literal == 'CARGO' else 'cargo'
                           if executable in ('cargo', 'cargo.exe') else '<string>') + '"'
        if index and stream[index - 1].kind == token.kind == 'ident':
            normalized += ' '
            offsets.append(token.start)
        normalized += value
        offsets.extend([token.start] * len(value))
    return normalized, offsets


def imported_types(definitions):
    result = {'cargo-command': {'Command'}, 'trybuild': {'trybuild::TestCases'},
              'cargo-metadata': {'cargo_metadata::MetadataCommand'}}
    stream = tokens(definitions)
    for index, token in enumerate(stream[:-1]):
        if token.value != 'use' or stream[index + 1].value == '<':
            continue
        end = next((position for position in range(index + 1, len(stream)) if stream[position].value == ';'), None)
        if end is None:
            continue
        for path, alias in use_leaves(stream[index + 1:end]):
            if not path or path[-1] == '*':
                continue
            name = alias or path[-1]
            if path[-1] == 'Command':
                result['cargo-command'].add(name)
            if path == ['trybuild']:
                result['trybuild'].add(name + '::TestCases')
            elif path == ['trybuild', 'TestCases']:
                result['trybuild'].add(name)
            if path == ['cargo_metadata']:
                result['cargo-metadata'].add(name + '::MetadataCommand')
            elif path == ['cargo_metadata', 'MetadataCommand']:
                result['cargo-metadata'].add(name)
    result['cargo-command'].update(re.findall(r'\btype\s+(\w+)=(?:std::process::)?Command\b', definitions))
    return result


def scan_text(text, context=''):
    normalized, offsets = normalized_source(text)
    definitions = normalized + '\n' + (normalized_source(context)[0] if context else '')
    result, recorded = [], set()
    imported = imported_types(definitions)
    compiler_sessions = set()
    for constructor in imported['trybuild']:
        compiler_sessions.update(re.findall(r'\blet\s+(?:mut\s+)?(\w+)\s*=' + re.escape(constructor) + r'::new\(', definitions))
        compiler_sessions.update(re.findall(r'\b(\w+):&' + re.escape(constructor) + r'\b', definitions))

    def record(kind, match):
        position = offsets[match.start()]
        if (kind, position) in recorded:
            return
        recorded.add((kind, position))
        preceding = text[:position]
        functions = re.findall(r'\bfn\s+(\w+)\s*\(', preceding)
        result.append({'kind': kind, 'line': preceding.count('\n') + 1,
                       'function': functions[-1] if functions else None,
                       'expression': normalized[match.start():match.end()][:200]})

    for kind, pattern in PATTERNS.items():
        for match in pattern.finditer(normalized):
            if kind == 'trybuild-pass' and match[1] is not None and match[1] not in compiler_sessions:
                continue
            record(kind, match)
    executable = r'(?:"(?:cargo|CARGO)"|env!\("CARGO"\)|(?:std::)?env::var(?:_os)?\("CARGO"\))'
    bindings = set(re.findall(r'\blet\s+(\w+)\s*=\s*' + executable, definitions))
    bindings.update(re.findall(r'\b(?:const|static)\s+(\w+):[^;=]{1,200}=' + executable, definitions))
    for alias in imported['cargo-command']:
        pattern = re.compile(r'\b' + re.escape(alias) + r'::new\(([^;)]+)\)')
        for match in pattern.finditer(normalized):
            if match[1] not in ('"cargo"', '"CARGO"') and match[1] not in bindings:
                continue
            record('cargo-command', match)
    for kind in ('trybuild', 'cargo-metadata'):
        for alias in imported[kind]:
            for match in re.finditer(r'\b' + re.escape(alias) + r'::(?:new|default)\(', normalized):
                record(kind, match)
    return sorted(result, key=lambda r: (r['line'], r['kind']))


def scan_workspace(manifest, repository):
    rows = []
    for source in sources(manifest):
        for row in scan_text(source.text, source.context):
            rows.append(dict(row, package=source.package['name'],
                             path=source.path.relative_to(repository).as_posix()))
    return rows
