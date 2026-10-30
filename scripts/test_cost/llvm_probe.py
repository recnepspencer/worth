#!/usr/bin/env python3
"""Probe the single-workspace LLVM protocol without executing the test bodies."""
import json
import os
from pathlib import Path

from cargo_capture import CargoRunner
from llvm_measure import measure
from filesystem_path import filesystem_path
from run_lane import child_environment
from workspace import repository_root


def materialize(output):
    output = Path(output)
    kit, court = output / 'kit', output / 'court'
    (kit / 'src').mkdir(parents=True, exist_ok=True)
    (court / 'tests').mkdir(parents=True, exist_ok=True)
    manifest = output / 'Cargo.toml'
    manifest.write_text('[workspace]\nmembers=["kit", "court"]\nresolver="2"\n'
                        '[profile.dev]\ndebug=0\n', encoding='utf-8')
    (kit / 'Cargo.toml').write_text('[package]\nname="worth-testkit-ir-probe"\nversion="0.0.0"\n'
                                  'edition="2024"\n', encoding='utf-8')
    (kit / 'src/lib.rs').write_text(
        '#[inline(never)]\npub fn build<T: Copy>(value: T) -> T { value }\n'
        'pub fn concrete(value: u64) -> u64 { build(value) }\n'
        '#[test]\nfn measurement_must_not_execute_unit_tests() {\n'
        '    let _ = concrete(17);\n    panic!("LLVM probe unit test executed");\n}\n', encoding='utf-8')
    (court / 'Cargo.toml').write_text('[package]\nname="worth-court-ir-probe"\nversion="0.0.0"\n'
                                    'edition="2024"\nautotests=false\n'
                                    '[dependencies]\nworth-testkit-ir-probe={path="../kit"}\n'
                                    '[[test]]\nname="ir_contract"\npath="tests/ir_contract.rs"\n', encoding='utf-8')
    (court / 'tests/ir_contract.rs').write_text(
        '#[test]\nfn measurement_must_not_execute_integration_tests() {\n'
        '    let _ = worth_testkit_ir_probe::build(23u32);\n'
        '    panic!("LLVM probe integration test executed");\n}\n', encoding='utf-8')
    return manifest


def validate(measured):
    expected = {'worth-testkit-ir-probe-lib', 'worth-testkit-ir-probe-lib-test-harness',
                'worth-court-ir-probe-ir_contract'}
    if set(measured) != expected:
        raise ValueError('LLVM probe did not observe both library units and the integration test')
    helpers, totals = set(), {}
    for identity, row in measured.items():
        total = next(value['lines'] for value in row['rows'] if value['name'] == 'TOTAL')
        if total <= 0 or row['ir']['bytes'] <= 0:
            raise ValueError('LLVM probe emitted no measurable IR: ' + identity)
        totals[identity] = total
        helpers.update(helper for value in row['rows'] for helper in value['testkit_helpers'])
        executable = row['executable']
        if executable:
            contents = filesystem_path(executable).read_bytes()
            if not contents or os.name == 'nt' and contents[:2] != b'MZ':
                raise ValueError('LLVM flags did not retain a linked executable: ' + executable)
        else:
            archives = [Path(value) for value in row['filenames'] if value.endswith('.rlib')]
            if not archives or any(not filesystem_path(path).read_bytes().startswith(b'!<arch>\n') for path in archives):
                raise ValueError('LLVM flags did not retain the linked library archive')
    if not any(value.startswith('worth_testkit_ir_probe::build') for value in helpers):
        raise ValueError('LLVM probe did not attribute the actual testkit generic instantiation')
    return {'llvm_ir_lines': totals, 'testkit_helpers': sorted(helpers), 'tests_executed': 0}


def execute(repository):
    repository = Path(repository).resolve()
    output = repository / 'target/test-cost/llvm-probe'
    manifest = materialize(output)
    environment = child_environment()
    # A nested probe workspace uses the existing root target, never a new cache.
    environment['CARGO_TARGET_DIR'] = str(repository / 'target')
    runner = CargoRunner(repository, environment)
    metadata = runner(['cargo', 'metadata', '--offline', '--format-version', '1',
                       '--manifest-path', str(manifest)], output, output / 'metadata.log')
    if metadata.returncode or not (output / 'Cargo.lock').is_file():
        raise ValueError('LLVM probe failed to resolve its offline workspace lock')
    measured = measure(manifest, output / 'llvm', runner)
    summary = validate(measured)
    (output / 'result.json').write_text(json.dumps(summary, indent=2) + '\n', encoding='utf-8')
    return summary


if __name__ == '__main__':
    print(json.dumps(execute(repository_root(Path.cwd())), indent=2))
