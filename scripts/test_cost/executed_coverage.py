"""Prove actual executions, including mains and rustdoc, against selected tests."""
from collections import Counter
import json
from pathlib import Path

from coverage import inventory
from logs import read_lane


class ExecutedCoverageFailure(ValueError):
    def __init__(self, result):
        self.result = result
        super().__init__('actual executions do not prove the selected coverage: ' +
                         json.dumps(result, sort_keys=True))


def aliases(discovered):
    """Native libtest JSON uses package::target, not nextest's binary ID."""
    result = {}
    for binary, suite in discovered.get('rust-suites', {}).items():
        if suite.get('status', 'listed') != 'listed':
            raise ValueError(f'test inventory was not listed: {binary}')
        if not suite.get('harness', True):
            continue
        alias = f'{suite["package-name"]}::{suite["binary-name"]}'
        if alias in result and result[alias] != binary:
            raise ValueError(f'ambiguous native test binary alias: {alias}')
        result[alias] = binary
    return result


def execution_kind(command):
    if command[:3] == ['cargo', 'nextest', 'run']:
        return 'nextest'
    if len(command) > 1 and Path(command[1]).name == 'account.py':
        return 'standalone'
    if command[:2] == ['cargo', 'test'] and '--doc' in command:
        return 'documentation'
    return None


def identity(event):
    name = event.get('name', '')
    if '$' not in name:
        raise ValueError(f'test result has no binary identity: {name!r}')
    binary, test = name.split('$', 1)
    if not binary or not test:
        raise ValueError(f'test result has an incomplete identity: {name!r}')
    return binary, test


def observations(paths, discovered):
    """Read each retained execution log once; do not pass its working mirror.

    Build/list/telemetry logs contribute no executions. Rustdoc events already
    carry the exact owner and name established by doc_contracts' framed parser.
    """
    native_aliases = aliases(discovered)
    events, walls, seen = [], {}, set()
    for value in paths:
        path = Path(value)
        key = path.resolve()
        if key in seen:
            raise ValueError(f'execution evidence log repeats: {path}')
        seen.add(key)
        with path.open(encoding='utf-8-sig', errors='replace') as stream:
            first = stream.readline()
        if not first.startswith('Command: '):
            continue
        command = json.loads(first.removeprefix('Command: '))
        kind = execution_kind(command)
        if kind is None:
            continue
        lane = read_lane(path)
        walls[str(path)] = lane['telemetry'].get('wall_seconds')
        for event in lane['tests']:
            if event['event'] == 'ignored':
                continue
            binary, test = identity(event)
            if kind == 'nextest':
                if binary not in native_aliases:
                    raise ValueError(f'{path} reports unknown native binary alias: {binary}')
                binary = native_aliases[binary]
            events.append(dict(event, reported_name=event['name'], name=f'{binary}${test}',
                               binary=binary, test=test, log=str(path), execution=kind))
    return events, walls


def prove(discovered, selections, events, *, require_full_workspace=False):
    """Partial requests prove their exact selected subset, never a full workspace.

    A failed test is an observed execution but cannot produce a passing result.
    Scheduled-only inventory entries remain visible and must never execute.
    """
    if not isinstance(discovered, list):
        for binary, suite in discovered.get('rust-suites', {}).items():
            if suite.get('status', 'listed') != 'listed':
                raise ValueError(f'test inventory was not listed: {binary}')
    known = {(row['binary'], row['test']): row for row in inventory(discovered)}
    selected = Counter()
    for value in selections.values():
        for row in inventory(value):
            if row.get('selected', True) and not row['ignored']:
                selected[row['binary'], row['test']] += 1
    actual, failed = Counter(), []
    for event in events:
        if event['event'] == 'ignored':
            continue
        if event['event'] not in ('ok', 'failed'):
            raise ValueError(f'not a terminal test observation: {event["event"]}')
        key = identity(event)
        if ('binary' in event or 'test' in event) and key != (event.get('binary'), event.get('test')):
            raise ValueError(f'canonical event identity contradicts its name: {event}')
        actual[key] += 1
        if event['event'] == 'failed':
            failed.append(key)
    expected = set(selected)
    landing = {key for key, row in known.items() if not row['ignored']}
    unselected = sorted(landing - expected)
    missing = sorted(expected - set(actual))
    unexpected = sorted(set(actual) - expected)
    multiple = sorted(key for key, count in actual.items() if count > 1)
    unknown = sorted((set(actual) | expected) - set(known))
    ignored_executed = sorted(key for key in actual if key in known and known[key]['ignored'])
    ignored_selected = sorted(key for key in expected if key in known and known[key]['ignored'])
    selection_multiple = sorted(key for key, count in selected.items() if count > 1)
    violations = any((missing, unexpected, multiple, unknown, ignored_executed,
                      ignored_selected, selection_multiple, failed))
    if require_full_workspace and unselected:
        violations = True
    return {
        'status': 'fail' if violations else 'partial' if unselected else 'pass',
        'scope': 'selected-subset' if unselected else 'full-workspace',
        'require_full_workspace': require_full_workspace,
        'selected': len(expected), 'executed': sum(actual.values()),
        'passed': sum(actual.values()) - len(failed), 'failed': sorted(failed),
        'scheduled_only': sum(row['ignored'] for row in known.values()),
        'missing': missing, 'unexpected': unexpected, 'multiple': multiple,
        'unknown': unknown, 'ignored_executed': ignored_executed,
        'ignored_selected': ignored_selected, 'selection_multiple': selection_multiple,
        'unselected': unselected,
        'tests': [dict(binary=key[0], test=key[1], executions=actual[key],
                       selections=selected[key], ignored=row['ignored'])
                  for key, row in sorted(known.items())],
    }


def verify(discovered, selections, logs, *, document_inventory=None,
           require_full_workspace=False):
    """Convenience entry point for native inventories plus optional doc rows.

    For already merged structural coverage, call observations with the native
    discovered object, then prove with complete_coverage.merge's returned rows.
    """
    events, walls = observations(logs, discovered)
    if document_inventory is not None:
        documents = inventory(document_inventory)
        discovered = inventory(discovered) + documents
        selections = dict(selections, documentation=documents)
    result = prove(discovered, selections, events,
                   require_full_workspace=require_full_workspace)
    result['logs'] = walls
    if result['status'] == 'fail':
        raise ExecutedCoverageFailure(result)
    return result
