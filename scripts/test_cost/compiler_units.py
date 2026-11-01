"""Required compiler units come from Cargo's preexecution dependency graph."""
import hashlib
import json
import os
from pathlib import Path


class CompilerUnitRefusal(ValueError):
    pass


def encoded(value):
    return json.dumps(value, sort_keys=True, separators=(',', ':'))


def digest(value):
    return hashlib.sha256(encoded(value).encode()).hexdigest()


def normalized_path(value):
    return os.path.normcase(str(Path(value).resolve())).replace('\\', '/')


def target_identity(target):
    return {'name': target['name'], 'kind': sorted(target['kind']),
            'crate_types': sorted(target['crate_types']),
            'source': normalized_path(target['src_path']), 'edition': target['edition']}


def declarations(graph):
    if graph.get('version') != 1 or not isinstance(graph.get('units'), list):
        raise CompilerUnitRefusal('unsupported Cargo unit graph')
    units = graph['units']
    roles = [set() for _ in units]

    def visit(index, role):
        if not isinstance(index, int) or not 0 <= index < len(units):
            raise CompilerUnitRefusal('Cargo unit dependency index is invalid')
        kinds = units[index]['target']['kind']
        role = 'host' if 'custom-build' in kinds or 'proc-macro' in kinds else role
        if role in roles[index]:
            return
        roles[index].add(role)
        for dependency in units[index]['dependencies']:
            visit(dependency['index'], role)

    for index in graph['roots']:
        visit(index, 'target')
    if any(not role for role in roles):
        raise CompilerUnitRefusal('Cargo unit graph contains unreachable units')
    identities, active = {}, set()

    def identity(index):
        if index in identities:
            return identities[index]
        if index in active:
            raise CompilerUnitRefusal('Cargo unit graph contains a dependency cycle')
        active.add(index)
        row = units[index]
        mode = row['mode']
        if mode not in ('build', 'check', 'test', 'doc', 'doctest', 'run-custom-build'):
            raise CompilerUnitRefusal(f'undeclared Cargo compiler mode: {mode}')
        dependencies = []
        for edge in row['dependencies']:
            dependencies.append({'unit': identity(edge['index']),
                                 **{key: value for key, value in edge.items() if key != 'index'}})
        # Cargo's null platform means the physical host/native compile kind.
        # A native linked library can serve both a build script and a product;
        # those use paths must not mint separate compilation allowances.
        role = 'host' if row['platform'] is None else 'target'
        meaning = {'package': row['pkg_id'], 'target': target_identity(row['target']),
                   'role': role, 'platform': row['platform'],
                   'output': 'check' if mode == 'check' else 'codegen', 'mode': mode,
                   'profile': row['profile'], 'features': sorted(row['features']),
                   'dependencies': sorted(dependencies, key=encoded)}
        value = digest(meaning)
        identities[index] = value
        active.remove(index)
        return value

    for index in range(len(units)):
        identity(index)
    rows = []
    for index, row in enumerate(units):
        if row['mode'] == 'run-custom-build':
            continue  # Execution of an already compiled build script is not rustc.
        rows.append({'identity': identities[index], 'package': row['pkg_id'],
                     'target': target_identity(row['target']),
                     'roles': ['host' if row['platform'] is None else 'target'],
                     'required_by': sorted(roles[index]),
                     'platform': row['platform'], 'mode': row['mode'],
                     'output': 'check' if row['mode'] == 'check' else 'codegen',
                     'profile': row['profile'], 'features': sorted(row['features']),
                     'dependency_units': sorted(identities[edge['index']] for edge in row['dependencies'])})
    return rows


def output(event):
    files = event.get('filenames', [])
    linked = ('.rlib', '.dll', '.so', '.dylib', '.a', '.lib', '.exe', '.wasm')
    if ('custom-build' in event['target']['kind'] or event.get('executable')
            or any(path.endswith(linked) for path in files)):
        return 'codegen'
    if any(path.endswith('.rmeta') for path in files):
        return 'check'
    raise CompilerUnitRefusal('compiler artifact has no attributable required output')


def matches_profile(declared, observed, mode, target_kind):
    fields = ('opt_level', 'debuginfo', 'debug_assertions', 'overflow_checks')
    test = mode == 'test' or (mode == 'check' and target_kind == ['test'])
    return (all(declared[field] == observed[field] for field in fields)
            and observed['test'] == test)


def attribute(event, declaration):
    target = event['target']
    candidates = [row for row in declaration['units']
                  if row['mode'] != 'doctest'  # The rustdoc driver consumes, rather than emits, the library artifact.
                  and row['package'] == event['package_id']
                  and row['target']['name'] == target['name']
                  and row['target']['kind'] == sorted(target['kind'])
                  and row['target']['source'] == normalized_path(target['src_path'])
                  and row['features'] == sorted(event['features'])
                  and row['output'] == output(event)
                  and matches_profile(row['profile'], event['profile'], row['mode'], row['target']['kind'])]
    unique = {row['identity']: row for row in candidates}
    if len(unique) != 1:
        raise CompilerUnitRefusal('artifact does not identify exactly one predeclared compiler unit: '
                                  + encoded({'package': event['package_id'], 'target': target,
                                             'matches': sorted(unique)}))
    row = next(iter(unique.values()))
    return observed_unit(row, declaration)


def observed_unit(row, declaration):
    return {'identity': row['identity'], 'roles': row['roles'], 'output': row['output'],
            'mode': row['mode'], 'features': row['features'],
            'platform': row['platform'], 'declared_target': row['target'],
            'dependency_units': row['dependency_units'], 'declared_profile': row['profile'],
            'compiler_inputs': declaration['compiler_inputs']}


def failed_unit(event, declaration):
    """An exact Cargo target error proves an attempt, never an aggregate exit."""
    if event.get('message', {}).get('level') != 'error':
        raise CompilerUnitRefusal('non-error diagnostic does not prove a failed compiler attempt')
    target = event['target']
    candidates = {row['identity']: row for row in declaration['units']
                  if row['package'] == event['package_id']
                  and row['target']['name'] == target['name']
                  and row['target']['kind'] == sorted(target['kind'])
                  and row['target']['source'] == normalized_path(target['src_path'])}
    if len(candidates) != 1:
        raise CompilerUnitRefusal('failed target does not identify exactly one predeclared compiler unit: '
                                  + encoded({'package': event['package_id'], 'target': target,
                                             'matches': sorted(candidates)}))
    return observed_unit(next(iter(candidates.values())), declaration)
