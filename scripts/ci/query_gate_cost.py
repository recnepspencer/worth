"""Batch governed gate selections through the shared census and prebuilt runner."""
from collections import Counter, defaultdict
import json
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'test_cost'))
from cargo_capture import CargoRunner
from doc_lane import execute as execute_documentation_lane
from logs import read_lane
from run_lane import LaneExecutionFailure, execute as execute_workspace

from query_gate_courts import CATEGORIES, CATEGORY_SUITES, TEST_SUITES, landing_suites, selected


DOC_SUITES = {workspace + '-doc': workspace for workspace in ('root', 'query', 'bank')}
DOC_ALIASES = {'doc': tuple(DOC_SUITES)}


def expand_suites(suites):
    return [name for suite in suites for name in DOC_ALIASES.get(suite, (suite,))]


def workspace_of(suite):
    return DOC_SUITES[suite] if suite in DOC_SUITES else TEST_SUITES[suite][0]


class GateSelectionFailure(ValueError):
    pass


def governed(suite):
    return suite in DOC_SUITES or (suite in TEST_SUITES and
           TEST_SUITES[suite][0] in ('root', 'query', 'bank') and not suite.endswith('-scale'))


def descriptor(binary, suite):
    try:
        kind = suite['kind']
        return {'binary': binary, 'package': suite['package-name'], 'target': suite['binary-name'],
                'lib': kind in ('lib', 'proc-macro'), 'harness': suite.get('harness', True)}
    except KeyError as error:
        raise GateSelectionFailure(f'incomplete nextest binary {binary}: {error}') from error


def exact_binary(binary):
    if any(value in binary for value in ('(', ')', '\n')):
        raise GateSelectionFailure(f'unsupported exact nextest binary identity: {binary}')
    return f'binary_id(={binary})'


def native_expression(suite, binary):
    result = exact_binary(binary['binary'])
    if '-court-' not in binary['package']:
        return result
    if suite in CATEGORY_SUITES:
        return f'({result} & test(/^{CATEGORY_SUITES[suite]}::/))'
    categories = ' + '.join(f'test(/^{name}::/)' for name in CATEGORIES)
    return f'({result} & not ({categories}))'


class SuiteQueries:
    """Resolve the gate's existing predicates against nextest's real inventory."""
    def __init__(self, suites, output):
        self.suites = tuple(suites)
        self.output = Path(output)
        self.expected = {suite: set() for suite in suites}
        self.ignored = {suite: set() for suite in suites}
        self.binaries = {suite: {} for suite in suites}
        self.coverage = {'status': 'unmeasured'}
        self.aliases = {}

    def __call__(self, discovered):
        jobs, owners, all_tests = [], defaultdict(list), set()
        for binary_id, metadata in sorted(discovered['rust-suites'].items()):
            binary = descriptor(binary_id, metadata)
            if binary['harness']:
                alias = f'{binary["package"]}::{binary["target"]}'
                if alias in self.aliases and self.aliases[alias] != binary_id:
                    raise GateSelectionFailure(f'ambiguous libtest binary alias {alias}')
                self.aliases[alias] = binary_id
        for suite in self.suites:
            _, want, _, _ = TEST_SUITES[suite]
            terms, mains = [], []
            for binary_id, metadata in sorted(discovered['rust-suites'].items()):
                binary = descriptor(binary_id, metadata)
                cases = metadata['testcases']
                if metadata.get('status', 'listed') != 'listed':
                    raise GateSelectionFailure(f'{binary_id} was not listed; exact coverage is unknown')
                all_tests.update((binary_id, name) for name, case in cases.items() if not case['ignored'])
                if not want(binary):
                    continue
                chosen = [(name, case) for name, case in cases.items() if selected(suite, binary, name)]
                self.binaries[suite][binary_id] = binary
                executable = False
                for name, case in chosen:
                    identity = (binary_id, name)
                    if case['ignored']:
                        self.ignored[suite].add(identity)
                    else:
                        self.expected[suite].add(identity)
                        owners[identity].append(suite)
                        executable = True
                if not executable:
                    continue
                if binary['harness']:
                    terms.append(native_expression(suite, binary))
                else:
                    if set(cases) != {'<main>'}:
                        raise GateSelectionFailure(f'{binary_id} is not a single standalone main')
                    mains.append(binary_id)
            if terms or mains:
                jobs.append((suite, ' + '.join(terms) if terms else None, None, 'ci', mains))
        duplicate = {str(key): value for key, value in owners.items() if len(value) != 1}
        if duplicate:
            raise GateSelectionFailure('requested suites repeat tests: ' + json.dumps(duplicate))
        unselected = sorted(all_tests - set(owners))
        self.coverage = {'status': 'partial' if unselected else 'pass',
                         'unselected': [f'{binary}${name}' for binary, name in unselected],
                         'selected': len(owners), 'ignored': len(set().union(*self.ignored.values()))}
        self.output.mkdir(parents=True, exist_ok=True)
        (self.output / 'gate-selection.json').write_text(json.dumps({
            'coverage': self.coverage,
            'suites': {suite: {'tests': sorted(self.expected[suite]), 'ignored': sorted(self.ignored[suite])}
                       for suite in self.suites}}, indent=2) + '\n', encoding='utf-8')
        return jobs


def executed_observations(logs, aliases):
    events, walls = [], {}
    for path in logs:
        path = Path(path)
        with path.open(encoding='utf-8', errors='replace') as stream:
            first = stream.readline()
        if not first.startswith('Command: '):
            continue
        command = json.loads(first.removeprefix('Command: '))
        native = command[:3] == ['cargo', 'nextest', 'run']
        documentation = command[:2] == ['cargo', 'test'] and '--doc' in command
        main = len(command) > 1 and Path(command[1]).name == 'account.py'
        if not (native or main or documentation):
            continue
        observed = read_lane(path)
        walls[str(path)] = observed['telemetry'].get('wall_seconds')
        for event in observed['tests']:
            if '$' not in event['name']:
                raise GateSelectionFailure(f'{path} omits the binary identity of {event["name"]}')
            binary, test = event['name'].split('$', 1)
            if native:
                if binary not in aliases:
                    raise GateSelectionFailure(f'{path} reports an unknown libtest binary alias {binary}')
                binary = aliases[binary]
            events.append(dict(event, reported_name=event['name'], name=f'{binary}${test}',
                               binary=binary, test=test, log=str(path)))
    return events, walls


class CostGate:
    def __init__(self, repository, output, environment, execute=None, runner_factory=None):
        self.repository, self.output = Path(repository), Path(output)
        self.environment = environment
        self.execute = execute or execute_workspace
        self.runner_factory = runner_factory or CargoRunner
        self.groups, self.results, self.finished = {}, {}, set()
        self.configured = False

    def configure(self, suites):
        names = [suite for suite in suites if governed(suite)]
        if len(set(names)) != len(names):
            raise GateSelectionFailure('a governed suite repeats in the execution plan')
        groups = defaultdict(list)
        for suite in names:
            groups[workspace_of(suite)].append(suite)
        if self.finished and dict(groups) != self.groups:
            raise GateSelectionFailure('configure every selected suite before workspace execution')
        self.groups, self.configured = dict(groups), True

    def run(self, suite):
        if not governed(suite):
            raise GateSelectionFailure(f'{suite} is not a governed landing suite')
        if not self.configured:
            self.configure([suite])
        workspace = workspace_of(suite)
        if suite not in self.groups.get(workspace, []):
            raise GateSelectionFailure('configure every selected suite before workspace execution')
        if workspace not in self.finished:
            self.run_workspace(workspace)
        return self.results[suite]

    def run_workspace(self, workspace):
        output = self.output / workspace
        names = self.groups[workspace]
        native = [name for name in names if name not in DOC_SUITES]
        doc_suite = next((name for name in names if name in DOC_SUITES), None)
        queries = SuiteQueries(native, output)
        runner = self.runner_factory(self.repository, self.environment)
        failure, result, documentation = None, None, None
        try:
            try:
                if native:
                    result = self.execute(self.repository, workspace, [], output, cargo_runner=runner,
                                          queries=queries, include_docs=bool(doc_suite))
                else:
                    result = execute_documentation_lane(self.repository, workspace, output, runner)
                    queries.coverage = {'status': 'partial', 'reason': 'native inventory not requested'}
            except LaneExecutionFailure as error:
                result, failure = error.result, str(error)
                if not native:
                    queries.coverage = {'status': 'partial', 'reason': 'native inventory not requested'}
            documentation = result.get('documentation') if result else None
            if doc_suite:
                if documentation is None:
                    raise GateSelectionFailure('requested documentation has no execution observation')
                queries.expected[doc_suite] = {(r['binary'], r['test']) for r in documentation['selections']}
                queries.ignored[doc_suite] = {(r['binary'], r['test']) for r in documentation['inventory'] if r['ignored']}
                if documentation['status'] != 'pass':
                    message = f'documentation failed: exit {documentation["exit_code"]}'
                    failure = f'{failure}; {message}' if failure else message
            if set(landing_suites(workspace)) <= set(native) and queries.coverage['status'] != 'pass':
                raise GateSelectionFailure('complete landing gate leaves native tests unselected')
        except (ValueError, OSError) as error:
            failure = f'{failure}; {error}' if failure else str(error)
        if doc_suite:
            queries.expected.setdefault(doc_suite, set())
            queries.ignored.setdefault(doc_suite, set())
        queries.coverage['documentation'] = (
            'observed' if documentation is not None else 'unmeasured' if doc_suite else 'not-requested')
        try:
            events, walls = executed_observations(runner.logs, queries.aliases)
            self.record(workspace, queries, events, walls, failure)
        except (ValueError, OSError) as error:
            self.record(workspace, queries, [], {}, f'{failure or ""} {error}'.strip())
        self.finished.add(workspace)

    def record(self, workspace, queries, events, walls, failure):
        selected = set().union(*queries.expected.values())
        observed = [event for event in events if event['event'] != 'ignored']
        counts = Counter((event['binary'], event['test']) for event in observed)
        unexpected = sorted(set(counts) - selected)
        repeated = sorted(identity for identity, count in counts.items() if count != 1)
        if unexpected or repeated:
            failure = f'{failure or ""} unexpected executions={unexpected}; repeated executions={repeated}'.strip()
        for suite in self.groups[workspace]:
            expected = queries.expected[suite]
            cases = [event for event in observed if (event['binary'], event['test']) in expected]
            missing = sorted(expected - set(counts))
            logs = sorted({event['log'] for event in cases})
            timings = [walls[path] for path in logs]
            eligible = bool(expected or queries.ignored[suite]) or suite in CATEGORY_SUITES or suite in DOC_SUITES
            run_seconds = (sum(timings) if logs and all(t is not None for t in timings)
                           else 0 if not expected and not failure else None)
            self.results[suite] = {
                'suite': suite, 'workspace': workspace, 'tests': len(expected),
                'passed': sum(event['event'] == 'ok' for event in cases),
                'failed': [event['name'] for event in cases if event['event'] == 'failed'],
                'ignored': len(queries.ignored[suite]),
                'missing': [f'{binary}${name}' for binary, name in missing],
                'names': [f'{binary}${name}' for binary, name in sorted(expected)],
                'coverage': queries.coverage, 'failure': failure,
                'logs': logs, 'run_seconds': run_seconds, 'no_tests': not eligible,
                'build_seconds': None, 'chunks': cases,
                'ok': not failure and not missing and eligible and all(event['event'] == 'ok' for event in cases),
            }
