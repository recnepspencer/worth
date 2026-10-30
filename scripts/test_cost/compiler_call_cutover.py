"""Cut declared Store compiler suites over to the typed prebuilt reader."""
import argparse
import json
from pathlib import Path
import re

from rust_calls import arguments, calls
from workspace import members
from filesystem_path import filesystem_path
from rust_items import items
from rust_source import replace_spans, string_value, tokens


def literal(expression):
    values = [string_value(token) for token in tokens(expression) if token.kind == 'string']
    if len(values) != 1:
        raise ValueError(f'expected one suite identity: {expression}')
    return values[0]


def rewrite(text, environments):
    changes, changed = [], []
    for call in calls(text, {'run_cargo_ui_fixture_suite'}):
        args = arguments(text, call)
        if len(args) != 7:
            raise ValueError('compiler suite must have exactly seven old arguments')
        suite = literal(args[1])
        environment = environments[suite]
        replacement = 'read_ui_fixture_suite(' + ', '.join(
            [args[0], args[1], 'UiCompilerEnvironment::' + environment.title(), args[5], args[6]]) + ')'
        changes.append((call.start, call.end, replacement))
        changed.append(suite)
    # The one explicit declaration caller constructs fixtures itself.
    for call in calls(text, {'cargo'}):
        preceding = text[max(0, call.start - 32):call.start]
        if preceding.endswith('UiProofEnvironment::'):
            if not environments:
                raise ValueError('explicit compiler declaration needs an environment')
            environment = environments.get('recovery-read-authority')
            if environment is None:
                continue
            end = call.end
            suffix = re.match(r'\s*\.\s*(?:unwrap|expect)\s*\([^)]*\)', text[end:])
            if suffix:
                end += suffix.end()
            changes.append((call.start - len('UiProofEnvironment::'), end,
                            'UiCompilerEnvironment::' + environment.title()))
    result = replace_spans(text, changes)
    result = result.replace('run_cargo_ui_fixture_suite', 'read_ui_fixture_suite')
    result = result.replace('run_ui_proof_suite', 'read_ui_proof_suite')
    result = result.replace('UiProofEnvironment', 'UiCompilerEnvironment')
    # Construction belongs to the runner; remove displaced dependency builders.
    removals = []
    for item in items(result):
        fragment = result[item.start:item.end]
        if item.keyword == 'fn' and ('cargo_dependency_manifest(' in fragment or
                                     item.name in ('dependency_manifest', 'dependency_declarations',
                                                   'physical_dependency_manifest', 'production_dependency_manifest')):
            removals.append((item.start, item.end, ''))
    result = replace_spans(result, removals)
    result = re.sub(r'(?:\w+::)*cargo_dependency_manifest\b\s*,?\s*', '', result)
    # Typed environment is a direct import, independent of old support glob use.
    if re.search(r'(?<![:\w])UiCompilerEnvironment::', result) and not re.search(r'\buse\b[^;]*\bUiCompilerEnvironment\b', result):
        result = 'use worth_store_test_support::compiler_boundary::UiCompilerEnvironment;\n' + result
    result = re.sub(r'Path::new\(concat!\(\s*env!\("CARGO_MANIFEST_DIR"\),\s*"/\.\./[^"\n]+"\s*\)\)(?=\s*\.ancestors\(\))',
                    'Path::new(env!("CARGO_MANIFEST_DIR"))', result)
    return result, changed


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--inventory', required=True, type=Path)
    parser.add_argument('--apply', action='store_true')
    args = parser.parse_args()
    root = Path.cwd().resolve()
    inventory = json.loads(args.inventory.read_text(encoding='utf-8'))
    environments = {row['suite_identity']: row['candidate_batch'] for row in inventory['environments']}
    legacy = {}
    for row in inventory['environments']:
        parent = Path(row['fixtures'][0]['relocated_source_path']).parent
        legacy[row['legacy_source_root']] = parent.relative_to('workspaces/worth-store').as_posix()
    paths = {root / row['call_location']['path'] for row in inventory['environments']}
    paths.update(root / fixture['declaration_location']['path']
                 for row in inventory['environments'] for fixture in row['fixtures'])
    # Root and dependency builders live beside the fixture declarations.
    for package in members(root / 'workspaces/worth-store/Cargo.toml'):
        if '-court-' not in package['name']:
            continue
        directory = filesystem_path(package['manifest'].parent / 'tests')
        for io_path in directory.rglob('*.rs'):
            if 'sources' in io_path.parts:
                continue
            text = io_path.read_text(encoding='utf-8')
            if any(name in text for name in ('cargo_dependency_manifest', 'UiProofEnvironment', 'run_cargo_ui_fixture_suite')):
                paths.add(package['manifest'].parent / 'tests' / io_path.relative_to(directory))
    for path in sorted(paths):
        if any(marker in path.as_posix() for marker in
               ('/support/compile_fail_support/', '/blob_harness_public_boundary_compile_fail/',
                '/layout_indexes/layout_compile_fail/')):
            continue  # Reviewed dynamic residue; typed callers migrate separately.
        text = filesystem_path(path).read_text(encoding='utf-8')
        changed, suites = rewrite(text, environments)
        for previous, current in legacy.items():
            changed = changed.replace('"' + previous + '"', '"' + current + '"')
        if changed != text:
            print(f'{path.relative_to(root)}: {suites}')
            if args.apply:
                filesystem_path(path).write_text(changed, encoding='utf-8')


if __name__ == '__main__':
    main()
