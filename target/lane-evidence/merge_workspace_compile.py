"""Compile a workspace from its own committed configuration and build identity."""
from pathlib import Path
import subprocess
import sys

root = Path.cwd().resolve()
workspace = sys.argv[1]
directory = {'root': root, 'query': root / 'workspaces/worth-query',
             'store': root / 'workspaces/worth-store'}[workspace]
command = [sys.executable, str(root / 'scripts/application_build.py'), 'cargo',
           *(['check', '--workspace', '--tests'] if workspace == 'query'
             else ['clippy', '--workspace', '--all-targets'])]
print('Working directory:', directory, flush=True)
print('Cargo command:', subprocess.list2cmdline(command), flush=True)
sys.exit(subprocess.run(command, cwd=directory).returncode)
