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
    'trybuild-pass': re.compile(r'\b\w+\.pass\('),
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
            value = '"' + (literal if literal in ('cargo', 'CARGO') else '<string>') + '"'
        if index and stream[index - 1].kind == token.kind == 'ident':
            normalized += ' '
            offsets.append(token.start)
        normalized += value
        offsets.extend([token.start] * len(value))
    result = []
    for kind, pattern in PATTERNS.items():
        for match in pattern.finditer(normalized):
            position = offsets[match.start()]
            preceding = text[:position]
            functions = re.findall(r'\bfn\s+(\w+)\s*\(', preceding)
            result.append({'kind': kind, 'line': preceding.count('\n') + 1,
                           'function': functions[-1] if functions else None,
                           'expression': normalized[match.start():match.end()][:200]})
    return sorted(result, key=lambda r: (r['line'], r['kind']))


def scan_workspace(manifest, repository):
    rows = []
    for package, path, text in sources(manifest):
        for row in scan_text(text):
            rows.append(dict(row, package=package['name'],
                             path=path.relative_to(repository).as_posix()))
    return rows
