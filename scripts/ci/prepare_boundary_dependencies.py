"""Fetch locked inputs before the existing boundary check's offline metadata."""
import argparse
from pathlib import Path
import subprocess
import sys
import tomllib


def manifests(root):
    root = Path(root).resolve()
    config = root/'tools/boundary-check/config'
    policy = tomllib.loads((config/'road1.toml').read_text(encoding='utf-8'))
    selected = {'Cargo.toml'}
    for key in ('subworkspaces', 'context_workspaces'):
        for row in policy.get(key, []):
            selected.add(str(Path(row['path'])/'Cargo.toml').replace('\\', '/'))
    for key in ('dependency_denials', 'source_dependency_allowlists', 'dependency_target_allowlists'):
        selected.update(row['workspace_manifest'] for row in policy.get(key, []))
    for line in (config/'test-workspaces.txt').read_text(encoding='utf-8').splitlines():
        if not line.strip() or line.lstrip().startswith('#'):
            continue
        directory, _ = line.split('|', 1)
        selected.add(str(Path(directory.strip())/'Cargo.toml').replace('\\', '/'))
    for manifest in selected:
        path = (root/manifest).resolve()
        if not path.is_relative_to(root) or not path.is_file() or not path.with_name('Cargo.lock').is_file():
            raise ValueError('configured boundary workspace needs its committed manifest and lock: ' + manifest)
    return sorted(selected)


def prepare(root, *, dry_run=False, invoke=subprocess.run):
    for manifest in manifests(root):
        command = ['cargo', 'fetch', '--locked', '--manifest-path', manifest]
        print(subprocess.list2cmdline(command), flush=True)
        if not dry_run:
            # Keep Cargo's original stdout, stderr and failure code visible.
            result = invoke(command, cwd=root)
            if result.returncode:
                return result.returncode
    return 0


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repository', type=Path, default=Path(__file__).resolve().parents[2])
    parser.add_argument('--dry-run', action='store_true')
    args = parser.parse_args()
    try:
        return prepare(args.repository, dry_run=args.dry_run)
    except (OSError, ValueError, KeyError, TypeError) as error:
        print('boundary dependency preparation failed: ' + str(error), file=sys.stderr)
        return 2


if __name__ == '__main__':
    raise SystemExit(main())
