"""Relocate fixture path attributes to the same real producer source they used."""
import argparse
import json
import os
from pathlib import Path

from workspace import WORKSPACES, members, repository_root
from filesystem_path import filesystem_path
from rust_source import modules, replace_spans


def rewrite(text, source, original_source):
    changes, observations = [], []
    for module in modules(text):
        if not module.path or module.body:
            continue
        current = source.parent / module.path
        if filesystem_path(current).is_file():
            continue
        original = (original_source.parent / module.path).resolve()
        if not filesystem_path(original).is_file():
            observations.append({'module': module.name, 'status': 'unresolved', 'original': str(original)})
            continue
        relative = os.path.relpath(original, source.parent).replace('\\', '/')
        changes.append((*module.path_span, json.dumps(relative)))
        observations.append({'module': module.name, 'status': 'relocated', 'original': str(original),
                             'before': module.path, 'after': relative})
    return replace_spans(text, changes), observations


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--workspace', choices=WORKSPACES, required=True)
    parser.add_argument('--apply', action='store_true')
    args = parser.parse_args()
    root = repository_root(Path.cwd())
    packages = members(root / WORKSPACES[args.workspace])
    owners = {p['name'].replace('-', '_'): p['manifest'].parent for p in packages}
    for package in packages:
        if '-court-' not in package['name']:
            continue
        directory = filesystem_path(package['manifest'].parent / 'tests')
        for io_path in sorted(directory.rglob('*.rs')):
            relative = io_path.relative_to(directory)
            if 'sources' not in relative.parts:
                continue
            index = relative.parts.index('sources')
            owner = owners.get(relative.parts[index + 1])
            if owner is None:
                continue
            source = package['manifest'].parent / 'tests' / relative
            original = owner / 'tests' / Path(*relative.parts[index + 2:])
            text = io_path.read_text(encoding='utf-8')
            changed, observations = rewrite(text, source, original)
            if observations:
                print(json.dumps({'source': source.relative_to(root).as_posix(), 'paths': observations}))
            if changed != text and args.apply:
                io_path.write_text(changed, encoding='utf-8')


if __name__ == '__main__':
    main()
