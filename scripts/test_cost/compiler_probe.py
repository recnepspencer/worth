#!/usr/bin/env python3
"""Probe the batched engine against real denied and supported fixture programs."""
import json
from pathlib import Path

from cargo_capture import CargoRunner
from compile_batch import prepare_environment
from compiler_audiences import group
from logs import llvm_lines
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
    absent = fixtures / 'absent_dependency.rs'
    denied.write_text('fn main() { let _ = test_cost_sealed_probe::Sealed { field: 7 }; }\n', encoding='utf-8')
    supported.write_text('fn main() { let _ = test_cost_sealed_probe::admitted(7); }\n', encoding='utf-8')
    absent.write_text('use test_cost_sealed_probe::admitted;\nfn main() {}\n', encoding='utf-8')
    environment = child_environment()
    environment['CARGO_TARGET_DIR'] = str(repository / 'target')
    runner = CargoRunner(repository, environment)
    metadata = runner(['cargo', 'metadata', '--offline', '--format-version', '1', '--manifest-path', str(manifest)],
                      output, output / 'metadata.log')
    if metadata.returncode:
        raise ValueError('probe workspace metadata failed')
    row = dict(identity='production', profile='dev', reject_test_features=True, packages=group([{
        'dependencies': {'test-cost-sealed-probe': {'path': output.as_posix()}},
        'sources': [denied, supported], 'audience': 'real-compiler-probe'},
        {'dependencies': {}, 'sources': [absent], 'audience': 'no-direct-dependency'}]))
    observed = prepare_environment(manifest, row, output / 'batch', runner)
    fates = {Path(source).stem: value['fate'] for source, value in observed['fixtures'].items()}
    if fates != {'denied': 'denied', 'supported': 'compiled', 'absent_dependency': 'denied'}:
        raise ValueError(f'real compiler attribution failed: {fates}')
    messages = observed['fixtures'][str(denied.resolve())]['messages']
    if not any(message.get('code', {}).get('code') == 'E0451' for message in messages if message.get('code')):
        raise ValueError('real private-field boundary was not observed')
    missing = observed['fixtures'][str(absent.resolve())]['messages']
    if not any(message.get('code', {}).get('code') == 'E0432' for message in missing if message.get('code')):
        raise ValueError('a sibling fixture exposed its direct dependency')
    ir = output / 'sealed-probe.ll'
    emitted = runner(['cargo', 'rustc', '--manifest-path', str(manifest), '--lib',
                      '--', '--emit=llvm-ir=' + str(ir) + ',link'], output, output / 'ir-compiler.log')
    if emitted.returncode or not ir.is_file():
        raise ValueError('compiler did not emit the requested library IR')
    measured = runner(['cargo', 'llvm-lines', '--files', str(ir)], output, output / 'ir-lines.log')
    if measured.returncode:
        raise ValueError('LLVM file-mode measurement failed')
    total = next(row['lines'] for row in llvm_lines(measured.stdout) if row['name'] == 'TOTAL')
    if total <= 0:
        raise ValueError('real library has no measured IR')
    print(json.dumps({'fixtures': fates, 'diagnostic_messages': [
        {'level': message['level'], 'message': message['message']} for message in messages],
        'llvm_ir_lines': total}, indent=2))


if __name__ == '__main__':
    main()
