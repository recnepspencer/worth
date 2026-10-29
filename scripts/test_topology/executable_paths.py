"""Resolve moved tests' product executables from the active Cargo profile directory."""
import json
import re

from rust_source import pairs, replace_spans, string_value, tokens


class ExecutablePaths:
    def __init__(self, stem, workspace, owners):
        self.name, self.workspace = stem + '-testkit-executables', workspace
        self.directory = workspace / 'crates' / self.name
        self.binaries = {t['name']: o['cargo']['name'] for o in owners for t in o['cargo']['targets'] if 'bin' in t['kind']}
        self.used, self.readers = set(), set()

    def occurrences(self, text):
        stream = tokens(text)
        matching = pairs(stream)
        for i, token in enumerate(stream[:-4]):
            if token.value != 'env' or [t.value for t in stream[i + 1:i + 3]] != ['!', '('] or stream[i + 3].kind != 'string':
                continue
            value = string_value(stream[i + 3])
            if value.startswith('CARGO_BIN_EXE_'):
                binary = value.removeprefix('CARGO_BIN_EXE_')
                if binary not in self.binaries:
                    raise ValueError('executable macro has no inventoried binary: ' + binary)
                yield token.start, stream[matching[i + 2]].end, binary

    def scan(self, text, owner):
        for _a, _b, binary in self.occurrences(text):
            self.used.add(binary)
            self.readers.add(owner)

    def rewrite(self, text, source):
        changes, receipts = [], []
        for a, b, binary in self.occurrences(text):
            function = re.sub(r'\W', '_', binary)
            value = self.name.replace('-', '_') + '::' + function + '()'
            changes.append((a, b, value))
            receipts.append({'source': str(source), 'offset': a, 'kind': 'executable-path', 'before': text[a:b], 'after': value})
        return replace_spans(text, changes), receipts

    def emit(self, effects):
        text = '''//! Built product executables, colocated with the active test profile.
use std::sync::OnceLock;

fn executable(name: &str) -> String {
    let test = std::env::current_exe().expect("active integration executable");
    let profile = test.parent().and_then(std::path::Path::parent)
        .expect("Cargo integration executable is under profile/deps");
    let path = profile.join(format!("{name}{}", std::env::consts::EXE_SUFFIX));
    assert!(path.is_file(), "product executable must be built by the court gate: {}", path.display());
    path.to_str().expect("Cargo executable path is UTF-8").to_owned()
}
'''
        for binary in sorted(self.used):
            text += '\npub fn ' + re.sub(r'\W', '_', binary) + '() -> &\'static str {\n'
            text += '    static PATH: OnceLock<String> = OnceLock::new();\n'
            text += '    PATH.get_or_init(|| executable(' + json.dumps(binary) + ')).as_str()\n}\n'
        effects.write(self.directory / 'src/lib.rs', text)

    def prerequisites(self):
        return [{'package': self.binaries[name], 'binary': name} for name in sorted(self.used)]
