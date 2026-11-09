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
            if (repository / path).is_file()
            if (lines := warning_lines((repository / path).read_text(encoding='utf-8')))]


def snapshot_warning_violations(repository, residuals):
    """Filed residuals must remain present in the complete warning discovery."""
    repository = Path(repository)
    findings = committed_warnings(repository)
    discovered = {finding['snapshot'] for finding in findings}
    violations = [dict(finding, kind='unfiled-snapshot-warning')
                  for finding in findings if finding['snapshot'] not in residuals]
    for snapshot, issue in residuals.items():
        if not (repository / snapshot).is_file():
            kind = 'missing-snapshot-warning-residual'
        elif snapshot not in discovered:
            kind = 'stale-snapshot-warning-residual'
        else:
            continue
        violations.append({'kind': kind, 'snapshot': snapshot, 'issue': issue})
    return violations
