#!/usr/bin/env python3
"""Relocate split court snapshots beside their existing, unique fixture sources."""
import argparse
import json
from pathlib import Path
import tomllib

from workspace import members, repository_root
from filesystem_path import filesystem_path


def plan(manifest, repository):
    manifest, repository = Path(manifest).resolve(), Path(repository).resolve()
    manifest.relative_to(repository)
    packages = members(manifest)
    courts = []
    for package in packages:
        if '-court-' not in package['name']:
            continue
        data = tomllib.loads(package['manifest'].read_text(encoding='utf-8'))
        for target in data.get('test', []):
            entry = package['manifest'].parent / target.get('path', 'tests/' + target['name'] + '.rs')
            courts.append(entry.with_suffix('') / 'sources')
    moves = []
    for package in packages:
        if '-testkit-' not in package['name']:
            continue
        namespace = package['name'].replace('-testkit-', '-').replace('-', '_')
        assets = package['manifest'].parent / 'assets'
        for snapshot in sorted(filesystem_path(assets).rglob('*.stderr')):
            if snapshot.with_suffix('.rs').is_file():
                continue
            relative = snapshot.relative_to(filesystem_path(assets))
            candidates = [root / namespace / relative.with_suffix('.rs') for root in courts
                          if filesystem_path(root / namespace / relative.with_suffix('.rs')).is_file()]
            if len(candidates) != 1:
                raise ValueError(f'orphan snapshot has {len(candidates)} exact source owners: {snapshot}')
            source = assets / relative
            destination = candidates[0].with_suffix('.stderr')
            source.resolve().relative_to(repository)
            destination.resolve().relative_to(repository)
            if filesystem_path(destination).exists():
                raise ValueError(f'fixture already owns a snapshot: {destination}')
            moves.append((source, destination))
    return moves


def apply(moves, repository):
    repository = Path(repository).resolve()
    for source, destination in moves:
        source.resolve().relative_to(repository)
        destination.resolve().relative_to(repository)
        filesystem_path(source).replace(filesystem_path(destination))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--manifest', type=Path, required=True)
    parser.add_argument('--apply', action='store_true')
    args = parser.parse_args()
    repository = repository_root(Path.cwd())
    moves = plan(args.manifest, repository)
    print(json.dumps([{'from': str(source.relative_to(repository)), 'to': str(destination.relative_to(repository))}
                      for source, destination in moves], indent=2))
    if args.apply:
        apply(moves, repository)


if __name__ == '__main__':
    main()
