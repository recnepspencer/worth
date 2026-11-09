"""Compiler snapshots pin protected refusals, never incidental lint diagnostics."""
from pathlib import Path
import subprocess


def warning_lines(text):
    return [number for number, line in enumerate(text.splitlines(), 1)
            if line.startswith(('warning:', 'warning['))]


def committed_warnings(repository):
    repository = Path(repository)
    paths = subprocess.check_output(
        ['git', 'ls-files', '-z', '--', '*.stderr'], cwd=repository).decode().split('\0')
    return [{'snapshot': path, 'lines': lines} for path in paths if path
            if (lines := warning_lines((repository / path).read_text(encoding='utf-8')))]
