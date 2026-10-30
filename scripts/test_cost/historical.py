#!/usr/bin/env python3
"""Reproduce a source census from a committed tree without changing a worktree."""
import argparse
import json
from pathlib import Path
import subprocess
import tarfile

from census import collect, markdown
from filesystem_path import filesystem_path
from workspace import WORKSPACES, repository_root


def safe_member(root, member):
    destination = (Path(root) / member.name).resolve()
    if not destination.is_relative_to(Path(root).resolve()):
        raise ValueError(f'archive member escapes the source snapshot: {member.name}')
    if member.issym() or member.islnk():
        raise ValueError(f'census source archives cannot redirect through links: {member.name}')
    return member


def export(repository, revision, destination):
    destination = Path(destination).resolve()
    marker = destination / '.source-revision'
    supplemental = False
    if marker.exists():
        recorded = marker.read_text().splitlines()
        if recorded[0] != revision:
            raise ValueError('use a separate source-snapshot directory for a different revision')
        if recorded[1:] == ['schema=2']:
            return
        supplemental = True
    destination.mkdir(parents=True, exist_ok=True)
    selected = ['Cargo.toml', 'crates', 'workspaces/worth-query',
                'workspaces/worth-query-bank-world', 'workspaces/worth-store', 'scripts']
    if supplemental:
        selected = ['scripts']
    command = ['git', 'archive', '--format=tar', revision, *selected]
    child = subprocess.Popen(command, cwd=repository, stdout=subprocess.PIPE)
    try:
        with tarfile.open(fileobj=child.stdout, mode='r|') as archive:
            for member in archive:
                if member.isfile() and (member.name.endswith('.rs') or member.name.endswith('/Cargo.toml')
                                        or member.name == 'Cargo.toml'):
                    archive.extract(safe_member(destination, member), path=filesystem_path(destination), filter='data')
    finally:
        child.stdout.close()
    if child.wait():
        raise ValueError('git source archive failed')
    marker.write_text(revision + '\nschema=2\n', encoding='utf-8')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--revision', required=True)
    args = parser.parse_args()
    root = repository_root(Path.cwd())
    revision = subprocess.check_output(['git', 'rev-parse', args.revision], cwd=root, text=True).strip()
    snapshot = root / 'target/test-cost/source-snapshots' / revision
    export(root, revision, snapshot)
    baseline = root.parent / 'm3-p2/target/lane-evidence'
    logs = [('landing-' + name, baseline / f'landing-store-{name}.log') for name in (
        'scenario', 'ui', 'owner-unit', 'formal', 'workspace-tests', 'phase34')]
    for workspace, relative in WORKSPACES.items():
        observed = collect(snapshot / relative, snapshot, logs if workspace == 'store' else [])
        observed['source_revision'] = revision
        observed['source_timing_relationship'] = 'Committed court-base source; Landing 1 timings precede the court move.'
        destination = root / 'target/test-cost' / workspace
        destination.mkdir(parents=True, exist_ok=True)
        (destination / 'before.json').write_text(json.dumps(observed, indent=2) + '\n', encoding='utf-8')
        (destination / 'before.md').write_text(markdown(observed), encoding='utf-8')
        print(workspace, observed['source_counts'], flush=True)


if __name__ == '__main__':
    main()
