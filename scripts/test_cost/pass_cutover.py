"""Apply a reviewed successful-fixture inventory without losing execution owners."""
import argparse
import json
import os
from pathlib import Path
import re
import tomllib

from rust_calls import calls, arguments
from rust_source import replace_spans
from supported_programs import emit
from trybuild_calls import rewrite
from workspace import members, repository_root, sources
from filesystem_path import filesystem_path


def helper_calls(text, names):
    changes = []
    for call in calls(text, names):
        args = arguments(text, call)
        if len(args) == 1 and re.fullmatch(r'&\s*\w+', args[0]):
            opening = text.index('(', call.start)
            changes.append((opening + 1, call.end - 1, ''))
    return replace_spans(text, changes)


def ordinary_groups(inventory, moved, repository):
    result = {}
    for source, names in moved.items():
        for name in names:
            owned = [repository / row['path'] for row in inventory['fixtures']
                     if any(c['source'] == source and c['function'] == name
                            for c in row['current_execution_callers'])]
            if not owned:
                raise ValueError(f'moved test has no inventoried successful programs: {source}:{name}')
            result.setdefault(source.split('/tests/')[0], []).append((name, owned))
    return result


def plan(repository, manifest, inventory):
    repository, manifest = Path(repository).resolve(), Path(manifest).resolve()
    writes, moved, helpers, packages = {}, {}, set(), set()
    owned_sources = sources(manifest)
    for package, path, masked in owned_sources:
        if 'trybuild' not in masked and 'run_compile_pass_bundle' not in masked:
            continue
        original = filesystem_path(path).read_text(encoding='utf-8')
        cutover = rewrite(original)
        changed = cutover.text.replace('crate::fixture_outcomes::', 'worth_testkit_compiler::')
        changed = re.sub(r'(?:#\[allow\(unused_imports\)\]\s*)?(?:pub\(crate\)\s+)?use\s*\{\s*\}\s*;', '', changed)
        if changed != original:
            writes[path] = changed
            packages.add(package['manifest'].parent)
        helpers.update(cutover.parameterized_helpers)
        if cutover.moved_tests:
            moved[path.relative_to(repository).as_posix()] = cutover.moved_tests
    # Session parameters may have callers in a different module.
    for package, path, _ in owned_sources:
        original = writes.get(path, filesystem_path(path).read_text(encoding='utf-8'))
        changed = helper_calls(original, helpers)
        if changed != original:
            writes[path] = changed
    groups = ordinary_groups(inventory, moved, repository)
    fixtures = {}
    for row in inventory['fixtures']:
        owners = {c['source'].split('/tests/')[0] for c in row['current_execution_callers']}
        if len(owners) != 1:
            raise ValueError(f'successful fixture requires a reviewed unique court owner: {row["path"]}')
        owner = owners.pop()
        fixtures.setdefault(owner, []).append(repository / row['path'])
    for owner, paths in fixtures.items():
        root = repository / owner
        data = tomllib.loads((root / 'Cargo.toml').read_text(encoding='utf-8'))
        targets = data.get('test', [])
        if len(targets) != 1:
            raise ValueError(f'successful programs require one explicit court target: {owner}')
        target = targets[0]['name']
        writes.update(emit(root, target, paths, groups.get(owner, [])))
        entry = root / targets[0]['path']
        original = writes.get(entry, filesystem_path(entry).read_text(encoding='utf-8'))
        writes[entry] = original + f'\n#[path = "{target}/supported_programs/mod.rs"]\nmod supported_programs;\n'
        packages.add(root)
    reader = repository / 'crates/worth-testkit-compiler'
    for root in packages:
        path = root / 'Cargo.toml'
        original = path.read_text(encoding='utf-8')
        changed = re.sub(r'^\s*"?trybuild"?\s*=.*\n', '', original, flags=re.M)
        if 'worth_testkit_compiler::' in ''.join(value for source, value in writes.items()
                                              if source.is_relative_to(root)):
            relative = os.path.relpath(reader, root).replace('\\', '/')
            dependency = f'"worth-testkit-compiler" = {{ "path" = "{relative}" }}\n'
            changed = changed.replace('[dependencies]\n', '[dependencies]\n' + dependency)
        if changed != original:
            writes[path] = changed
    return writes, moved


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--manifest', type=Path, required=True)
    parser.add_argument('--inventory', type=Path, required=True)
    parser.add_argument('--apply', action='store_true')
    args = parser.parse_args()
    repository = repository_root(Path.cwd())
    inventory = json.loads(args.inventory.read_text(encoding='utf-8'))
    writes, moved = plan(repository, args.manifest, inventory)
    if args.apply:
        for path, text in writes.items():
            filesystem_path(path.parent).mkdir(parents=True, exist_ok=True)
            filesystem_path(path).write_text(text, encoding='utf-8')
    print(json.dumps({'files': len(writes), 'moved_tests': moved}, indent=2))


if __name__ == '__main__':
    main()
