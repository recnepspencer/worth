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


def snapshot_warning_policy_violations(residuals, policy):
    """Live debt must match the reviewed set and its open, filed issue list."""
    expected = set(policy['pinned_snapshots'])
    observed = set(residuals)
    violations = [dict(kind='unapproved-snapshot-warning-residual', snapshot=path)
                  for path in sorted(observed - expected)]
    violations += [dict(kind='missing-approved-snapshot-warning-residual', snapshot=path)
                   for path in sorted(expected - observed)]
    for path, issue in residuals.items():
        if issue not in policy['open_filed_issues']:
            violations.append(dict(kind='unfiled-snapshot-warning-issue',
                                   snapshot=path, issue=issue))
    return violations
