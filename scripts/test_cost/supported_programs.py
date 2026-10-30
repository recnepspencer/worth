"""Move successful fixture programs into ordinary, executing court tests."""
import os
from pathlib import Path
import re

from filesystem_path import filesystem_path
from rust_items import items


def expose_entry(text):
    entries = [item for item in items(text) if item.keyword == 'fn' and item.name == 'main']
    if len(entries) != 1:
        raise ValueError('successful fixture must have one top-level main entry')
    entry = entries[0]
    header = text[entry.start:entry.body[0]]
    if entry.visibility[0] == entry.visibility[1]:
        position = entry.visibility[0]
        text = text[:position] + 'pub(crate) ' + text[position:]
    signature = header.partition('fn')[2]
    if '->' not in signature:
        invocation = 'program::main();'
    elif 'Result' in signature:
        invocation = 'program::main().expect("supported program must succeed");'
    elif 'ExitCode' in signature:
        invocation = 'assert_eq!(program::main(), std::process::ExitCode::SUCCESS);'
    else:
        raise ValueError('custom Termination type needs a deliberate ordinary-test cutover')
    return text, invocation


def behavior_name(path):
    value = re.sub(r'[^a-zA-Z0-9_]', '_', Path(path).stem)
    value = re.sub(r'^(?:test_|it_)', '', value)
    return value + '_compiles_and_runs'


def emit(package_root, target, fixtures, moved_tests=(), apply=False):
    """Each fixture has one execution owner; pass-only grouping names move."""
    package_root = Path(package_root)
    directory = package_root / 'tests' / target / 'supported_programs'
    plans, assigned = [], set()
    for name, paths in moved_tests:
        paths = [Path(path) for path in paths]
        if any(path in assigned for path in paths):
            raise ValueError('a successful fixture has two ordinary execution owners')
        assigned.update(paths)
        plans.append((name, name, paths))
    for source in sorted(set(Path(source) for source in fixtures) - assigned):
        plans.append((source.stem, behavior_name(source), [source]))
    names = set()
    writes = {}
    modules = []
    for label, test, paths in plans:
        label = re.sub(r'[^a-zA-Z0-9_]', '_', label)
        if label in names:
            raise ValueError(f'ordinary program filename collision: {label}')
        names.add(label)
        destination = directory / f'{label}.rs'
        mounts, invocations = [], []
        for index, source in enumerate(paths):
            original = filesystem_path(source).read_text(encoding='utf-8')
            updated, invocation = expose_entry(original)
            writes[source] = updated
            module = 'program' if len(paths) == 1 else 'program_' + str(index + 1)
            relative = os.path.relpath(source, destination.parent).replace('\\', '/')
            mounts.append(f'#[path = "{relative}"]\nmod {module};\n')
            invocations.append('    ' + invocation.replace('program::', module + '::'))
        writes[destination] = '\n'.join(mounts) + f'\n#[test]\nfn {test}() {{\n' + '\n'.join(invocations) + '\n}\n'
        modules.append(f'mod {label};')
    writes[directory / 'mod.rs'] = '\n'.join(sorted(modules)) + '\n'
    if apply:
        for path, text in writes.items():
            filesystem_path(path.parent).mkdir(parents=True, exist_ok=True)
            filesystem_path(path).write_text(text, encoding='utf-8')
    return writes
