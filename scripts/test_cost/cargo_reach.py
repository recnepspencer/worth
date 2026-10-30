"""Cargo's local package reach and the configured consumer document inventory."""
import json
import os
from pathlib import Path, PurePosixPath
import subprocess
import tomllib


TEST_ONLY_DIRS = ('tests/', 'benches/', 'examples/')


def run(command, cwd):
    return subprocess.run(command, cwd=cwd, capture_output=True, text=True, check=True).stdout


def local_package_dirs(repo, manifest, *, offline=True):
    command = ['cargo', 'metadata', '--format-version', '1', '--locked']
    if offline:
        command.append('--offline')
    data = json.loads(run([*command, '--manifest-path', manifest], repo))
    # Cargo reports absolute paths; Windows drive-letter spelling varies.
    root = os.path.abspath(repo).replace('\\', '/').rstrip('/') + '/'
    members = set(data['workspace_members'])
    directories = []
    for package in data['packages']:
        if package['source'] is not None:
            continue
        path = str(PurePosixPath(package['manifest_path'].replace('\\', '/')).parent)
        if not path.casefold().startswith(root.casefold()):
            raise ValueError(f'local package outside the repository: {path}')
        directories.append((path[len(root):], package['id'] in members))
    return directories


def consumer_documents(repo):
    """Every Markdown path named in the real consumer-doc gate configuration."""
    config = Path(repo)/'tools/boundary-check/config/consumer-docs.toml'
    found = set()

    def walk(value):
        if isinstance(value, str) and value.endswith('.md'):
            found.add(value)
        elif isinstance(value, dict):
            for item in value.values():
                walk(item)
        elif isinstance(value, list):
            for item in value:
                walk(item)

    walk(tomllib.loads(config.read_text(encoding='utf-8')))
    return found


def under(path, directory):
    return directory in ('', '.') or path == directory or path.startswith(directory.rstrip('/') + '/')


def reaches(path, directory, member):
    if not under(path, directory):
        return False
    inner = path[len(directory.rstrip('/')) + 1:]
    if inner == 'AGENT_CONTEXT.md':
        return False
    # Cargo does not compile dependency tests, benches or examples.
    return member or not inner.startswith(TEST_ONLY_DIRS)


def workspace_hits(changed, manifest, directories):
    """Include inherited Cargo inputs, even when a changed input was deleted."""
    ancestors = {PurePosixPath(manifest).parent}
    for directory, _ in directories:
        ancestors.add(PurePosixPath(directory))
    ancestors |= {parent for path in tuple(ancestors) for parent in path.parents}
    inputs = {str(directory/name) for directory in ancestors
              for name in ('Cargo.toml', 'Cargo.lock', 'rust-toolchain', 'rust-toolchain.toml')}
    configs = {str(directory/'.cargo') for directory in ancestors}
    return [path for path in changed if path in inputs or any(under(path, config) for config in configs)
            or any(reaches(path, directory, member) for directory, member in directories
                   if directory not in ('.', ''))]


def changed_paths(repo, base, head, *, merge_base=False):
    if not base or not head or set(base) == {'0'}:
        raise ValueError('no available comparison base')
    for revision in (base, head):
        run(['git', 'rev-parse', '--verify', '--end-of-options', revision+'^{commit}'], repo)
    separator = '...' if merge_base else '..'
    # Disable rename collapsing: both the removed and added path can own inputs.
    return sorted(set(filter(None, run(['git', 'diff', '--no-renames', '--name-only', '-z',
                                       f'{base}{separator}{head}', '--'], repo).split('\0'))))
