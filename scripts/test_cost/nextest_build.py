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


def selection_arguments(metadata, selection):
    """Validate explicit owners against Cargo, without another target registry."""
    if selection is None:
        return ['--workspace', '--all-targets']
    packages = sorted(set(selection.get('packages', [])))
    members = set(metadata['workspace_members'])
    owned = {p['name']: p for p in metadata['packages'] if p['id'] in members}
    if not packages or set(packages) - set(owned):
        raise NextestPrebuildFailure('native selection requires known workspace packages')
    kinds = selection.get('targets', {})
    if set(kinds) - {'lib', 'test', 'bin', 'example', 'bench'}:
        raise NextestPrebuildFailure('unknown native target kind')
    arguments = [part for package in packages for part in ('--package', package)]
    for kind, names in sorted(kinds.items()):
        if kind == 'lib':
            if names is not True or any(not any(set(t['kind']) & {'lib', 'proc-macro'}
                 for t in owned[p]['targets']) for p in packages):
                raise NextestPrebuildFailure('selected package has no library target')
            arguments.append('--lib')
        else:
            if not names or not isinstance(names, list):
                raise NextestPrebuildFailure('native targets need explicit names')
            for name in sorted(set(names)):
                if not any(t['name'] == name and kind in t['kind']
                           for p in packages for t in owned[p]['targets']):
                    raise NextestPrebuildFailure(f'unknown {kind} target: {name}')
                arguments += ['--' + kind, name]
    return arguments + ([] if kinds else ['--all-targets'])


def source_packages(metadata, selection):
    if selection is None:
        return None
    packages = {p['id']: p for p in metadata['packages']}
    members = set(metadata['workspace_members'])
    pending = [key for key in members if packages[key]['name'] in selection['packages']]
    nodes = {node['id']: node for node in metadata['resolve']['nodes']}
    required = set()
    while pending:
        key = pending.pop()
        if key in required:
            continue
        if key not in nodes:
            raise NextestPrebuildFailure('source selection lacks a resolved dependency: ' + key)
        required.add(key)
        pending.extend(nodes[key]['dependencies'])
    return sorted(packages[key]['name'] for key in required & members)


def build(manifest, output, cargo_runner, profile='dev', selection=None, before_compile=None):
    manifest, output = Path(manifest).resolve(), Path(output).resolve()
    output.mkdir(parents=True, exist_ok=True)
    observed = cargo_runner(['cargo', 'metadata', '--offline', '--format-version', '1',
                             '--manifest-path', str(manifest)], manifest.parent, output / 'cargo-metadata.log')
    if observed.returncode:
        raise NextestPrebuildFailure('Cargo metadata failed')
    metadata_path = output / 'cargo-metadata.json'
    metadata_path.write_text(observed.stdout, encoding='utf-8')
    metadata = json.loads(observed.stdout)
    selected = selection(metadata) if callable(selection) else selection
    arguments = selection_arguments(metadata, selected)
    (output / 'native-selection.json').write_text(json.dumps({
        'scope': 'workspace' if selected is None else 'selected-packages',
        'selection': selected, 'cargo_arguments': arguments}, indent=2) + '\n', encoding='utf-8')
    if before_compile is not None:
        before_compile(metadata, selected)
    observed = cargo_runner(['cargo', 'nextest', 'list', *arguments,
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


def list_tests(reuse, output, cargo_runner, expression='all()', partition=None):
    metadata = json.loads(Path(reuse[1]).read_text(encoding='utf-8'))
    directory = Path(metadata.get('workspace_root', Path(reuse[1]).parent))
    command = ['cargo', 'nextest', 'list', *reuse, '--message-format', 'json',
               '--ignore-default-filter', '-E', expression]
    if partition:
        command += ['--partition', partition]
    observed = cargo_runner(command,
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
