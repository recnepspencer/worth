"""Local selections and workspace qualification use the existing suite authority."""
from pathlib import Path
import tomllib

from query_gate_cost import expand_suites, governed, landing_suites, workspace_of
from query_gate_courts import CATEGORY_SUITES, TEST_SUITES
from workspace import WORKSPACES


def local_workspace(repository, current):
    repository, current = Path(repository).resolve(), Path(current).resolve()
    known = {(repository/manifest).parent.resolve(): name for name, manifest in WORKSPACES.items()}
    if current.is_relative_to(repository):
        for directory in (current, *current.parents):
            manifest = directory/'Cargo.toml'
            if manifest.is_file() and 'workspace' in tomllib.loads(manifest.read_text(encoding='utf-8')):
                owner = known.get(directory)
                if owner in ('root', 'query', 'bank'):
                    return owner
                break
            if directory == repository:
                break
    raise ValueError('choose an explicit suite here; local defaults cover Root, Query and Bank; '
                     'Store uses run_lane.py with explicit local lanes or its full landing command')


def select_suites(requested, full, defaults, full_order, repository, current):
    if full and requested is not None:
        raise ValueError('choose explicit suites or --full, not both')
    if requested is not None:
        return expand_suites(requested.split(','))
    if defaults is not None:
        return list(defaults)
    if full:
        return list(full_order)
    workspace = local_workspace(repository, current)
    return [name for name, row in TEST_SUITES.items()
            if row[0] == workspace and name not in CATEGORY_SUITES]


def workspace_coverage(suites, results):
    """A request is not qualification; require the existing observed coverage."""
    observed = {row['suite']: row for row in results}
    summary = {}
    for workspace in sorted({workspace_of(suite) for suite in suites if governed(suite)}):
        required = {*landing_suites(workspace), workspace+'-doc'}
        complete = required <= observed.keys() and all(
            observed[name].get('ok') is True
            and observed[name].get('coverage', {}).get('status') == 'pass'
            and observed[name].get('coverage', {}).get('documentation') == 'observed'
            for name in required)
        summary[workspace] = 'full' if complete else 'partial'
    return summary
