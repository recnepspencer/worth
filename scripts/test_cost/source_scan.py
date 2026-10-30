"""Lexical cost sites, with source locations rather than timing-derived guesses."""
from pathlib import Path
import re
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'test_topology'))
from rust_source import tokens, string_value

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


def scan_text(text):
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
    result = []
    compiler_sessions = set(re.findall(r'\blet\s+(?:mut\s+)?(\w+)\s*=trybuild::TestCases::new\(', normalized))
    compiler_sessions.update(re.findall(r'\b(\w+):&trybuild::TestCases\b', normalized))
    if 'trybuild::TestCases' in normalized:
        compiler_sessions.update(re.findall(r'\blet\s+(\w+)\s*=TestCases::new\(', normalized))
    for kind, pattern in PATTERNS.items():
        for match in pattern.finditer(normalized):
            if kind == 'trybuild-pass' and match[1] is not None and match[1] not in compiler_sessions:
                continue
            position = offsets[match.start()]
            preceding = text[:position]
            functions = re.findall(r'\bfn\s+(\w+)\s*\(', preceding)
            result.append({'kind': kind, 'line': preceding.count('\n') + 1,
                           'function': functions[-1] if functions else None,
                           'expression': normalized[match.start():match.end()][:200]})
    bindings = set(re.findall(r'\blet\s+(\w+)\s*=\s*"(?:cargo|CARGO)"', normalized))
    bindings.update(re.findall(r'\b(?:const|static)\s+(\w+):[^;=]{1,200}="(?:cargo|CARGO)"', normalized))
    aliases = {'Command'} | set(re.findall(r'\bCommand\s+as\s+(\w+)', normalized))
    for alias in aliases:
        pattern = re.compile(r'\b' + re.escape(alias) + r'::new\(([^;)]+)\)')
        for match in pattern.finditer(normalized):
            if match[1] not in ('"cargo"', '"CARGO"') and match[1] not in bindings:
                continue
            position = offsets[match.start()]
            line = text[:position].count('\n') + 1
            if any(row['kind'] == 'cargo-command' and row['line'] == line for row in result):
                continue
            functions = re.findall(r'\bfn\s+(\w+)\s*\(', text[:position])
            result.append({'kind': 'cargo-command', 'line': line,
                           'function': functions[-1] if functions else None,
                           'expression': match[0]})
    return sorted(result, key=lambda r: (r['line'], r['kind']))


def scan_workspace(manifest, repository):
    rows = []
    for package, path, text in sources(manifest):
        for row in scan_text(text):
            rows.append(dict(row, package=package['name'],
                             path=path.relative_to(repository).as_posix()))
    return rows
