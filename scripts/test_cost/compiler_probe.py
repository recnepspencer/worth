#!/usr/bin/env python3
"""Probe the batched engine against real denied and supported fixture programs."""
import json
from pathlib import Path

from cargo_capture import CargoRunner
from compile_batch import prepare_environment
from compiler_audiences import group
from run_lane import child_environment
from workspace import repository_root


def main():
    repository = repository_root(Path.cwd())
    output = repository / 'target/test-cost/compiler-probe'
    (output / 'src').mkdir(parents=True, exist_ok=True)
    manifest = output / 'Cargo.toml'
    manifest.write_text('[package]\nname="test-cost-sealed-probe"\nversion="0.0.0"\nedition="2024"\n'
                        '[workspace]\n[profile.dev]\ndebug=0\n', encoding='utf-8')
    (output / 'src/lib.rs').write_text('pub struct Sealed { field: u8 }\npub fn admitted(field:u8)->Sealed { Sealed {field} }\n', encoding='utf-8')
    fixtures = output / 'fixtures'
    fixtures.mkdir(exist_ok=True)
    denied, supported = fixtures / 'denied.rs', fixtures / 'supported.rs'
    denied.write_text('fn main() { let _ = test_cost_sealed_probe::Sealed { field: 7 }; }\n', encoding='utf-8')
    supported.write_text('fn main() { let _ = test_cost_sealed_probe::admitted(7); }\n', encoding='utf-8')
    environment = child_environment()
    environment['CARGO_TARGET_DIR'] = str(repository / 'target')
    runner = CargoRunner(repository, environment)
    metadata = runner(['cargo', 'metadata', '--offline', '--format-version', '1', '--manifest-path', str(manifest)],
                      output, output / 'metadata.log')
    if metadata.returncode:
        raise ValueError('probe workspace metadata failed')
    row = dict(identity='production', profile='dev', reject_test_features=True, packages=group([{
        'dependencies': {'test-cost-sealed-probe': {'path': output.as_posix()}},
        'sources': [denied, supported], 'audience': 'real-compiler-probe'}]))
    observed = prepare_environment(manifest, row, output / 'batch', runner)
    fates = {Path(source).stem: value['fate'] for source, value in observed['fixtures'].items()}
    if fates != {'denied': 'denied', 'supported': 'compiled'}:
        raise ValueError(f'real compiler attribution failed: {fates}')
    messages = observed['fixtures'][str(denied.resolve())]['messages']
    if not any(message.get('code', {}).get('code') == 'E0451' for message in messages if message.get('code')):
        raise ValueError('real private-field boundary was not observed')
    print(json.dumps({'fixtures': fates, 'diagnostic_messages': [
        {'level': message['level'], 'message': message['message']} for message in messages]}, indent=2))


if __name__ == '__main__':
    main()
