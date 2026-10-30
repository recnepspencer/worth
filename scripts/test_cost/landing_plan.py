"""Validate landing descriptors by reading their data, never invoking Cargo metadata."""
import argparse
import ast
import hashlib
import json
from pathlib import Path
import re

STORE = 'workspaces/worth-store/Cargo.toml'
QUERY = 'workspaces/worth-query/Cargo.toml'
ROOT = 'Cargo.toml'
BANK = 'workspaces/worth-query-bank-world/Cargo.toml'
STORE_LANDING = 'store: python scripts/test_cost/run_lane.py --workspace store --cargo-profile ci-test'
QUERY_LANDING = 'query: python scripts/ci/query_gate.py query-landing'
DOC_WORKSPACES = ((ROOT, 'root'), (QUERY, 'query'), (BANK, 'bank'))


def literal(node, values):
    if isinstance(node, ast.Constant):
        return node.value
    if isinstance(node, ast.Name) and node.id in values:
        return values[node.id]
    if isinstance(node, (ast.List, ast.Tuple)):
        return [literal(value, values) for value in node.elts]
    if isinstance(node, ast.Dict):
        keys = [literal(key, values) for key in node.keys]
        if len(set(keys)) != len(keys):
            raise ValueError('landing descriptor repeats a dictionary key')
        return dict(zip(keys, (literal(value, values) for value in node.values)))
    raise ValueError('landing configuration is not literal data')


def configuration(path):
    source = Path(path).read_text(encoding='utf-8-sig')
    tree = ast.parse(source)
    values = {}
    for node in tree.body:
        if not isinstance(node, ast.Assign) or len(node.targets) != 1 or not isinstance(node.targets[0], ast.Name):
            continue
        name = node.targets[0].id
        if name in ('FIXTURE', 'WORKSPACES', 'ALWAYS'):
            if name in values:
                raise ValueError('landing configuration is assigned more than once: ' + name)
            values[name] = literal(node.value, values)
    if 'WORKSPACES' not in values:
        raise ValueError('landing configuration has no WORKSPACES data')
    return values, source, tree


def consumer_document_lanes(tree, reached):
    candidates = [node for node in tree.body if isinstance(node, ast.FunctionDef)
                  and node.name == 'consumer_document_lanes']
    if len(candidates) != 1:
        raise ValueError('landing plan needs one consumer-document routing function')
    function = candidates[0]
    if any(isinstance(node, (ast.Call, ast.Import, ast.ImportFrom, ast.Global, ast.Nonlocal))
           for node in ast.walk(function)):
        raise ValueError('document routing must be a pure literal selection, without process or metadata calls')
    # Execute this isolated pure function only. No imports, main(), metadata
    # owner discovery or other code from the external module can execute.
    namespace = {}
    exec(compile(ast.fix_missing_locations(ast.Module(body=[function], type_ignores=[])),
                 '<consumer-document-selection>', 'exec'), namespace)
    return namespace['consumer_document_lanes'](reached)


def violations(workspaces, before=None, source=None):
    result = []
    store, query = workspaces.get(STORE, []), workspaces.get(QUERY, [])
    if store.count(STORE_LANDING) != 1:
        result.append('Store must select its complete prebuilt landing runner exactly once')
    if any(re.search(r'cargo test|store-test-runner|--include-ignored|--run-ignored', lane) for lane in store):
        result.append('Store repeats serial workspace, court partitions, smoke or ignored execution')
    if not any('clippy' in lane for lane in store) or not any(all(value in lane for value in ('phase34', 'formal-toolchain', 'integrity')) for lane in store):
        result.append('Store clippy, dependency direction, TLC or integrity descriptor disappeared')
    if any('C9' in lane for lane in store):
        result.append('C9 ignored-phase script must not be a normal landing descriptor')
    if source is not None and not re.search(r'C9[^\n]*scheduled-only[^\n]*excluded', source):
        result.append('C9 scheduled-only exclusion must be explicit in the landing plan')
    if query.count(QUERY_LANDING) != 1:
        result.append('Query must select query-landing exactly once')
    if any(value in lane for lane in query for value in ('query-execution,', 'query-process,', 'query_gate.py doc', 'observable-workflow', 'query-scale', 'cargo test')):
        result.append('Query retains overlapping native, aggregate-doc, example or scheduled selections')
    if not any('lint-query' in lane for lane in query):
        result.append('Query clippy descriptor disappeared')
    if before is not None:
        if set(workspaces) != set(before):
            result.append('workspace ownership keys changed outside the authorized cutover')
        for manifest in before:
            if manifest not in (STORE, QUERY) and workspaces.get(manifest) != before[manifest]:
                result.append('unapproved workspace descriptors changed: ' + manifest)
        for manifest in (STORE, QUERY):
            previous = [lane for lane in before[manifest] if 'clippy' in lane or 'lint-query' in lane]
            if any(lane not in workspaces[manifest] for lane in previous):
                result.append('existing warning/clippy descriptor changed: ' + manifest)
    return result


def phase34_violations(source):
    result = []
    if re.search(r'(?im)^\s*(?:&\s+)?cargo\b|Invoke-Checked|\$Workspace\b', source):
        result.append('Phase34 retains duplicate Cargo execution or obsolete Workspace switch')
    if source.count('& python') != 1 or 'scripts/ci/check_worth_store_phase34_dependencies.py' not in source:
        result.append('Phase34 must invoke only its existing dependency-direction Python check')
    if '$LASTEXITCODE -ne 0' not in source or 'throw ' not in source:
        result.append('Phase34 must propagate the dependency-direction failure')
    return result


def captured_gate_command(path, workspace, expanded):
    from logs import read_lane
    headers = read_lane(path)['headers']
    if any(headers[name] is None for name in ('Command', 'HEAD', 'Start')):
        raise ValueError('gate command capture has no original command provenance')
    command = json.loads(headers['Command'])
    indices = [index for index, value in enumerate(command) if Path(value).name == 'query_gate.py']
    if len(indices) != 1 or indices[0] + 1 >= len(command):
        raise ValueError('not an explicit workspace gate command')
    selected = command[indices[0] + 1].split(',')
    if selected != list(expanded):
        raise ValueError('landing alias differs from captured explicit command: ' + workspace)
    return {'workspace': workspace, 'capture': str(path), 'expanded': list(expanded),
            'captured': selected, 'head': headers['HEAD'], 'start': headers['Start'],
            'scope': 'command argument parity only; not runtime or completion evidence'}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--external', type=Path, required=True)
    parser.add_argument('--before', type=Path)
    parser.add_argument('--phase34', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--gate-selection', action='append', default=[], metavar='WORKSPACE=PATH')
    parser.add_argument('--gate-command', action='append', default=[], metavar='WORKSPACE=LOG')
    args = parser.parse_args()
    values, source, tree = configuration(args.external)
    before = configuration(args.before)[0] if args.before else None
    errors = violations(values['WORKSPACES'], before['WORKSPACES'] if before else None, source)
    errors += phase34_violations(args.phase34.read_text(encoding='utf-8'))
    if before and values.get('ALWAYS') != before.get('ALWAYS'):
        errors.append('global structural descriptors changed')
    routes = []
    for mask in range(8):
        reached = [manifest for bit, (manifest, _) in enumerate(DOC_WORKSPACES) if mask & (1 << bit)]
        actual = consumer_document_lanes(tree, reached)
        expected = [f'docs: python scripts/ci/query_gate.py {name}-doc' for manifest, name in DOC_WORKSPACES if manifest not in reached]
        if actual != expected:
            errors.append('consumer-document selection overlaps or omits a workspace: ' + repr(reached))
        routes.append({'reached': reached, 'documentation_lanes': actual})
    gate_sets = []
    if args.gate_selection or args.gate_command:
        import sys
        sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'ci'))
        from query_gate_cost import expand_suites
        for value in args.gate_selection:
            workspace, separator, path = value.partition('=')
            if not separator or workspace not in ('query', 'bank'):
                raise ValueError('gate selection input must be query=PATH or bank=PATH')
            observed = json.loads(Path(path).read_text())
            actual = list(observed['suites']) + [workspace+'-doc']
            expanded = expand_suites([workspace+'-landing'])
            if expanded != actual:
                errors.append('landing alias differs from the actual captured suite order: ' + workspace)
            gate_sets.append({'workspace': workspace, 'capture': path, 'expanded': expanded, 'captured': actual})
    if args.gate_command:
        for value in args.gate_command:
            workspace, separator, path = value.partition('=')
            if not separator or workspace not in ('query', 'bank'):
                raise ValueError('gate command input must be query=LOG or bank=LOG')
            gate_sets.append(captured_gate_command(Path(path), workspace, expand_suites([workspace+'-landing'])))
    result = {'status': 'pass' if not errors else 'fail', 'violations': errors,
              'scope': 'Read-only literal external WORKSPACES and pure document routing; no Cargo metadata execution.',
              'external': str(args.external.resolve()), 'sha256': hashlib.sha256(args.external.read_bytes()).hexdigest(),
              'workspaces': values['WORKSPACES'], 'consumer_document_routes': routes, 'actual_gate_aliases': gate_sets}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, indent=2)+'\n', encoding='utf-8')
    print(json.dumps({'status': result['status'], 'violations': errors, 'metadata_invocations': 0,
                      'document_route_cases': len(routes), 'actual_gate_aliases': len(gate_sets)}))
    return bool(errors)


if __name__ == '__main__':
    raise SystemExit(main())
