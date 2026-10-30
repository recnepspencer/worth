#!/usr/bin/env python3
"""Format changed Rust sources and each affected Cargo package."""
from pathlib import Path
import argparse
import subprocess
import sys
import tomllib

from cargo_capture import CargoRunner
from run_lane import child_environment
from workspace import WORKSPACES, members, repository_root
from filesystem_path import filesystem_path


def package_for(source, packages):
    source = Path(source).resolve()
    owners = [package for package in packages if source.is_relative_to(package['manifest'].parent)]
    return max(owners, key=lambda package: len(package['manifest'].parts)) if owners else None


def changed_sources(root, changes):
    return [Path(root) / relative for relative in sorted(set(changes))
            if Path(relative).suffix == '.rs' and filesystem_path(Path(root) / relative).is_file()]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--workspace', choices=WORKSPACES, action='append')
    args = parser.parse_args()
    root = repository_root(Path.cwd())
    changed = subprocess.check_output(['git', 'diff', '--name-only', '-z', 'HEAD'],
                                      cwd=root, stderr=subprocess.PIPE).decode().split('\0')
    changed += subprocess.check_output(['git', 'ls-files', '--others', '--exclude-standard', '-z'], cwd=root).decode().split('\0')
    runner = CargoRunner(root, child_environment())
    for workspace, relative in WORKSPACES.items():
        if args.workspace and workspace not in args.workspace:
            continue
        manifest = root / relative
        packages = members(manifest)
        selected = set()
        for path in changed_sources(root, changed):
            owner = package_for(path, packages)
            if owner is None:
                continue
            selected.add(owner['name'])
            # Explicit fixture programs are not ordinary Cargo module roots.
            result = subprocess.run(['rustfmt', '--edition', '2024', '--config', 'skip_children=true', str(path)], cwd=root)
            if result.returncode:
                return result.returncode
        if selected:
            command = ['cargo', 'fmt', '--manifest-path', str(manifest)]
            command += [argument for package in sorted(selected) for argument in ('-p', package)]
            result = runner(command, root, root / 'target/lane-evidence' / f'format-{workspace}.log')
            if result.returncode:
                return result.returncode
    return 0


if __name__ == '__main__':
    sys.exit(main())
