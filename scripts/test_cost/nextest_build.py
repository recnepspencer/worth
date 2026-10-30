"""Build a graph once, then reuse nextest's own executable metadata."""
import json
from pathlib import Path

from coverage import inventory, prove
from standalone import discover, separate, synthetic_inventory


CATEGORIES = ('process', 'compile_fail', 'oracle', 'determinism', 'cost', 'exhaustive', 'scale')
PREFIX = '(' + ' + '.join(f'test(/^{name}::/)' for name in CATEGORIES) + ')'
LANES = {'owner-unit': 'not kind(test)', 'ordinary': f'kind(test) & not {PREFIX}',
         **{name.replace('_', '-'): f'kind(test) & test(/^{name}::/)' for name in CATEGORIES}}


def documents(text):
    decoder, offset = json.JSONDecoder(), 0
    while (start := text.find('{', offset)) >= 0:
        try:
            value, end = decoder.raw_decode(text, start)
        except ValueError:
            offset = start + 1
            continue
        if isinstance(value, dict):
            yield value
        offset = end


class NextestPrebuildFailure(ValueError):
    pass


def summary(text, key):
    found = [value for value in documents(text) if key in value]
    if len(found) != 1:
        raise NextestPrebuildFailure(f'expected one nextest {key} inventory, found {len(found)}')
    return found[0]


def build(manifest, output, cargo_runner, profile='dev'):
    manifest, output = Path(manifest).resolve(), Path(output).resolve()
    output.mkdir(parents=True, exist_ok=True)
    observed = cargo_runner(['cargo', 'metadata', '--offline', '--format-version', '1',
                             '--manifest-path', str(manifest)], manifest.parent, output / 'cargo-metadata.log')
    if observed.returncode:
        raise NextestPrebuildFailure('Cargo metadata failed')
    metadata_path = output / 'cargo-metadata.json'
    metadata_path.write_text(observed.stdout, encoding='utf-8')
    observed = cargo_runner(['cargo', 'nextest', 'list', '--workspace', '--all-targets',
                             '--cargo-profile', profile,
                             '--manifest-path', str(manifest), '--list-type', 'binaries-only',
                             '--message-format', 'json'], manifest.parent, output / 'binaries-build.log')
    if observed.returncode:
        raise NextestPrebuildFailure('nextest executable prebuild failed')
    binaries_path = output / 'binaries.json'
    binaries, programs = separate(summary(observed.stdout, 'rust-binaries'),
                                  discover(observed.stdout + '\n' + observed.stderr))
    binaries_path.write_text(json.dumps(binaries), encoding='utf-8')
    (output / 'standalone.json').write_text(json.dumps(programs, indent=2) + '\n', encoding='utf-8')
    return ['--cargo-metadata', str(metadata_path),
            '--binaries-metadata', str(binaries_path)]


def list_tests(reuse, output, cargo_runner, expression='all()'):
    metadata = json.loads(Path(reuse[1]).read_text(encoding='utf-8'))
    directory = Path(metadata.get('workspace_root', Path(reuse[1]).parent))
    observed = cargo_runner(['cargo', 'nextest', 'list', *reuse, '--message-format', 'json',
                             '--ignore-default-filter', '-E', expression],
                            directory, Path(output).with_suffix('.log'))
    if observed.returncode:
        raise NextestPrebuildFailure('nextest test inventory failed')
    value = summary(observed.stdout, 'rust-suites')
    Path(output).write_text(json.dumps(value, indent=2) + '\n', encoding='utf-8')
    return value


def coverage(reuse, output, cargo_runner):
    output = Path(output)
    discovered = list_tests(reuse, output / 'inventory.json', cargo_runner)
    selected = {lane: list_tests(reuse, output / f'selection-{lane}.json', cargo_runner, expression)
                for lane, expression in LANES.items()}
    programs = json.loads((output / 'standalone.json').read_text(encoding='utf-8'))
    for program in programs:
        suite = synthetic_inventory([program])['rust-suites']
        discovered['rust-suites'].update(suite)
        lane = 'ordinary' if program['kind'] == 'test' else 'owner-unit'
        selected[lane]['rust-suites'].update(suite)
    (output / 'inventory.json').write_text(json.dumps(discovered, indent=2) + '\n', encoding='utf-8')
    for lane, value in selected.items():
        (output / f'selection-{lane}.json').write_text(json.dumps(value, indent=2) + '\n', encoding='utf-8')
    result = prove(discovered, selected)
    (output / 'coverage.json').write_text(json.dumps(result, indent=2) + '\n', encoding='utf-8')
    if result['status'] != 'pass':
        raise NextestPrebuildFailure('landing partitions do not select every nonignored test exactly once')
    return discovered, selected, result
