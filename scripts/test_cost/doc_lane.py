"""Apply the governed census and structural ratchets to a doc-only selection."""
import json
from pathlib import Path

from census import collect, markdown
from complete_coverage import merge
from doc_contracts import execute as execute_documentation
from executed_coverage import observations, prove
from lane_environment import inherited, validate as validate_environment
from ratchet import build_violations, compiler_observations, lane_budget, source_violations
from run_lane import LaneExecutionFailure
from workspace import WORKSPACES


def retain(result, repository, workspace, output):
    for destination in {output, repository / 'target/test-cost' / workspace}:
        destination.mkdir(parents=True, exist_ok=True)
        (destination / 'census.json').write_text(json.dumps(result, indent=2) + '\n', encoding='utf-8')
        (destination / 'census.md').write_text(markdown(result), encoding='utf-8')


def execute(repository, workspace, output, runner, profile='dev', engine=None):
    original_environment = inherited()
    environment = getattr(runner, 'environment', None)
    validate_environment(original_environment if environment is None else environment)
    repository, output = Path(repository).resolve(), Path(output).resolve()
    manifest = repository / WORKSPACES[workspace]
    configuration = json.loads(Path(__file__).with_name('budgets.json').read_text())
    # A doc-only request is the existing committed generic selection lane.
    budget = lane_budget(configuration, workspace, ['selection'], profile)
    output.mkdir(parents=True, exist_ok=True)
    before = collect(manifest, repository)
    forbidden = source_violations(before['source_sites'])
    (output / 'source-ratchet.json').write_text(json.dumps(forbidden, indent=2) + '\n', encoding='utf-8')
    if forbidden:
        retain(before, repository, workspace, output)
        raise ValueError('tests invoke compilers: ' + json.dumps(forbidden))
    start = len(runner.logs)
    documentation, failure = None, None
    try:
        documentation = (engine or execute_documentation)(manifest, output / 'documentation', runner, profile)
    except (ValueError, OSError) as error:
        failure = error
    logs = runner.logs[start:]
    complete, chosen, structural = merge([], {}, documentation)
    result = collect(manifest, repository, [(f'documentation-{i}', path) for i, path in enumerate(logs)],
                     complete if documentation is not None else None, chosen)
    result['documentation'] = documentation
    result['documentation_coverage'] = structural if documentation is not None else {'status': 'unmeasured'}
    result['coverage'] = {'status': 'partial', 'scope': 'documentation-only',
                          'reason': 'native inventory was not requested'}
    if documentation is not None:
        try:
            events, walls = observations(logs, {'rust-suites': {}})
            executed = prove(complete, chosen, events)
            executed.update(scope='documentation-only', logs=walls)
        except (ValueError, OSError) as error:
            failure = failure or error
            executed = {'status': 'fail', 'scope': 'documentation-only', 'failure': str(error)}
        result['executed_coverage'] = executed
        (output / 'executed-coverage.json').write_text(json.dumps(executed, indent=2) + '\n', encoding='utf-8')
        if executed['status'] == 'fail':
            failure = failure or ValueError('documentation executions do not match selections')
        if documentation['status'] != 'pass':
            failure = failure or ValueError(f'documentation failed: exit {documentation["exit_code"]}')
    violations = build_violations(compiler_observations(result['lanes'].values(), output), budget)
    result['build_ratchet'] = {'budget_lane': 'selection', 'violations': violations}
    result['failure'] = str(failure) if failure is not None else None
    (output / 'build-ratchet.json').write_text(json.dumps(violations, indent=2) + '\n', encoding='utf-8')
    retain(result, repository, workspace, output)
    if violations:
        message = 'build budget exceeded: ' + json.dumps(violations)
        if documentation is not None:
            raise LaneExecutionFailure(message, result)
        raise ValueError(message)
    if failure is not None:
        if documentation is not None:
            raise LaneExecutionFailure(str(failure), result) from failure
        raise failure
    return result
