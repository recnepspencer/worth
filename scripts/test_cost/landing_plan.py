"""Validate landing descriptors by reading their data, never invoking Cargo metadata."""
import argparse
import ast
import hashlib
import json
from pathlib import Path
import re
import sys

STORE = 'workspaces/worth-store/Cargo.toml'
QUERY = 'workspaces/worth-query/Cargo.toml'
ROOT = 'Cargo.toml'
BANK = 'workspaces/worth-query-bank-world/Cargo.toml'
STORE_LANDING = 'store: python scripts/test_cost/run_lane.py --workspace store --cargo-profile ci-test'
QUERY_LANDING = 'query: python scripts/ci/query_gate.py query-landing'
BANK_LANDING = 'bank: python scripts/ci/query_gate.py bank-landing'
ROOT_LANDING = 'root: python scripts/ci/root_gate.py'
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


def gate_contract(repository):
    """Read the gate's actual alias and category declarations without running it."""
    sys.path.insert(0, str(Path(repository).resolve() / 'scripts/ci'))
    from query_gate_cost import LANDING_ALIASES, landing_suites, workspace_of
    import root_gate
    from workspace import WORKSPACES
    contracts = {}
    for alias, expansion in LANDING_ALIASES.items():
        owners = {workspace_of(suite) for suite in expansion}
        if len(owners) != 1:
            raise ValueError('landing alias crosses workspaces: ' + alias)
        owner = owners.pop()
        expected = (*landing_suites(owner), owner + '-doc')
        if tuple(expansion) != expected or len(set(expansion)) != len(expansion) or any(
                suite.endswith('-scale') for suite in expansion):
            raise ValueError('landing alias omits, repeats or schedules a normal category: ' + alias)
        if owner in contracts:
            raise ValueError('workspace has multiple landing aliases: ' + owner)
        contracts[owner] = {'manifest': WORKSPACES[owner], 'alias': alias, 'expanded': list(expansion),
                            'descriptor': f'{owner}: python scripts/ci/query_gate.py {alias}'}
    if not {'query', 'bank'} <= contracts.keys():
        raise ValueError('Query and Bank require complete landing aliases')
    root_suites = tuple(root_gate.ROOT_SUITES)
    if root_suites != (*landing_suites('root'), 'root-doc') or len(set(root_suites)) != len(root_suites):
        raise ValueError('Root default must select every normal category and its own documentation exactly once')
    entrypoint = Path(root_gate.__file__).resolve().relative_to(Path(repository).resolve()).as_posix()
    contracts['root'] = {'manifest': WORKSPACES['root'], 'entrypoint': entrypoint,
                         'expanded': list(root_suites), 'descriptor': f'root: python {entrypoint}'}
    return contracts


def violations(workspaces, before=None, source=None, contracts=None):
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
    contracts = contracts or {'query': {'descriptor': QUERY_LANDING}, 'bank': {'descriptor': BANK_LANDING},
                              'root': {'descriptor': ROOT_LANDING}}
    if query.count(contracts['query']['descriptor']) != 1:
        result.append('Query must select query-landing exactly once')
    if any(value in lane for lane in query for value in ('query-execution,', 'query-process,', 'query_gate.py doc', 'observable-workflow', 'query-scale', 'cargo test')):
        result.append('Query retains overlapping native, aggregate-doc, example or scheduled selections')
    if not any('lint-query' in lane for lane in query):
        result.append('Query clippy descriptor disappeared')
    bank = workspaces.get(BANK, [])
    if bank.count(contracts['bank']['descriptor']) != 1:
        result.append('Bank must select its complete landing alias exactly once')
    if any(re.search(r'cargo test|query_gate.py (?:doc|bank(?:-|,|$))', lane)
           and lane != contracts['bank']['descriptor'] for lane in bank):
        result.append('Bank retains serial, overlapping or scheduled selections')
    if not any('clippy-bank' in lane or 'lint-bank' in lane for lane in bank):
        result.append('Bank clippy descriptor disappeared')
    root = workspaces.get(ROOT, [])
    if root.count(contracts['root']['descriptor']) != 1:
        result.append('Root must select its complete default gate exactly once')
    if any('cargo test' in lane or 'query_gate.py root' in lane or 'root_gate.py ' in lane for lane in root):
        result.append('Root retains serial, overlapping or scheduled selections')
    if not any('clippy' in lane for lane in root):
        result.append('Root clippy descriptor disappeared')
    if before is not None:
        if set(workspaces) != set(before):
            result.append('workspace ownership keys changed outside the authorized cutover')
        for manifest in before:
            if manifest not in (STORE, QUERY, BANK, ROOT) and workspaces.get(manifest) != before[manifest]:
                result.append('unapproved workspace descriptors changed: ' + manifest)
        for manifest in (STORE, QUERY, BANK, ROOT):
            previous = [lane for lane in before[manifest] if 'clippy' in lane or 'lint-' in lane]
            if any(lane not in workspaces[manifest] for lane in previous):
                result.append('existing warning/clippy descriptor changed: ' + manifest)
        previous_root = [lane for lane in before[ROOT]
                         if lane not in ('root: cargo test --workspace', contracts['root']['descriptor'])]
        if [lane for lane in root if lane != contracts['root']['descriptor']] != previous_root:
            result.append('existing Root structural or product-check descriptor changed')
    return result


def document_routes(tree):
    routes, errors = [], []
    for mask in range(8):
        reached = [manifest for bit, (manifest, _) in enumerate(DOC_WORKSPACES) if mask & (1 << bit)]
        actual = consumer_document_lanes(tree, reached)
        expected = [f'docs: python scripts/ci/query_gate.py {name}-doc'
                    for manifest, name in DOC_WORKSPACES if manifest not in reached]
        if actual != expected:
            errors.append('consumer-document selection overlaps or omits a workspace: ' + repr(reached))
        routes.append({'reached': reached, 'documentation_lanes': actual})
    return routes, errors


def planner_hook_violations(tree):
    """Require the planner's actual early, fail-closed validation handoff."""
    functions = {node.name: node for node in tree.body if isinstance(node, ast.FunctionDef)}
    main, hook = functions.get('main'), functions.get('validate_landing_plan')
    if main is None or hook is None:
        return ['planner must define main and its landing validation handoff']
    guards = [index for index, node in enumerate(main.body) if isinstance(node, ast.Expr)
              and ast.dump(node.value, include_attributes=False) == ast.dump(
                  ast.parse('validate_landing_plan(repo)', mode='eval').body, include_attributes=False)]
    if len(guards) != 1:
        return ['planner must invoke its validation handoff exactly once']
    if any(not isinstance(node, ast.Assign) or any(
            isinstance(call, ast.Call) and not (isinstance(call.func, ast.Name) and call.func.id == 'len')
            for call in ast.walk(node)) for node in main.body[:guards[0]]):
        return ['planner validation must precede git, Cargo metadata and plan emission']
    calls = [node for node in ast.walk(hook) if isinstance(node, ast.Call)
             and isinstance(node.func, ast.Attribute) and isinstance(node.func.value, ast.Name)
             and node.func.value.id == 'subprocess']
    expected = ast.parse('''[sys.executable, "-I", str(checker),
                            "--repository", str(Path(repo).resolve()),
                            "--external", str(Path(__file__).resolve()),
                            "--check-only"]''', mode='eval').body
    if len(calls) != 1 or calls[0].func.attr != 'run' or not calls[0].args or ast.dump(
            calls[0].args[0], include_attributes=False) != ast.dump(expected, include_attributes=False):
        return ['planner handoff must run its repository checker silently, without metadata']
    assignments = [node for node in hook.body if isinstance(node, ast.Assign)
                   and any(isinstance(target, ast.Name) and target.id == 'checker' for target in node.targets)]
    expected_checker = ast.parse('Path(repo).resolve() / "scripts/test_cost/landing_plan.py"', mode='eval').body
    if len(assignments) != 1 or ast.dump(assignments[0].value, include_attributes=False) != ast.dump(
            expected_checker, include_attributes=False):
        return ['planner checker must come from the selected repository root']
    failures = [node for node in hook.body if isinstance(node, ast.If)
                and ast.unparse(node.test) == 'observed.returncode']
    if len(failures) != 1 or not any(isinstance(node, ast.Raise) and isinstance(node.exc, ast.Call)
            and isinstance(node.exc.func, ast.Name) and node.exc.func.id == 'SystemExit'
            for node in ast.walk(failures[0])):
        return ['planner must refuse a failed landing validation before emitting a plan']
    return []


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
    script = 'root_gate.py' if workspace == 'root' else 'query_gate.py'
    indices = [index for index, value in enumerate(command) if Path(value).name == script]
    if len(indices) != 1:
        raise ValueError('not an explicit workspace gate command')
    arguments = command[indices[0]+1:]
    if workspace == 'root' and (not arguments or arguments[0].startswith('--')):
        selected = list(expanded)
    elif arguments:
        selected = arguments[0].split(',')
    else:
        raise ValueError('workspace gate command omits its selection')
    if selected != list(expanded):
        raise ValueError('landing alias differs from captured explicit command: ' + workspace)
    return {'workspace': workspace, 'capture': str(path), 'expanded': list(expanded),
            'captured': selected, 'head': headers['HEAD'], 'start': headers['Start'],
            'scope': 'command argument parity only; not runtime or completion evidence'}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--external', type=Path, required=True)
    parser.add_argument('--before', type=Path)
    parser.add_argument('--phase34', type=Path)
    parser.add_argument('--repository', type=Path, default=Path(__file__).resolve().parents[2])
    parser.add_argument('--output', type=Path)
    parser.add_argument('--check-only', action='store_true', help='silent successful validation for the external planner')
    parser.add_argument('--gate-selection', action='append', default=[], metavar='WORKSPACE=PATH')
    parser.add_argument('--gate-command', action='append', default=[], metavar='WORKSPACE=LOG')
    args = parser.parse_args()
    values, source, tree = configuration(args.external)
    before = configuration(args.before)[0] if args.before else None
    contracts = gate_contract(args.repository)
    errors = violations(values['WORKSPACES'], before['WORKSPACES'] if before else None, source,
                        contracts)
    errors += planner_hook_violations(tree)
    phase34 = args.phase34 or args.repository / 'scripts/ci/verify_worth_store_phase34.ps1'
    errors += phase34_violations(phase34.read_text(encoding='utf-8'))
    if before and values.get('ALWAYS') != before.get('ALWAYS'):
        errors.append('global structural descriptors changed')
    routes, route_errors = document_routes(tree)
    errors += route_errors
    gate_sets = []
    if args.gate_selection or args.gate_command:
        for value in args.gate_selection:
            workspace, separator, path = value.partition('=')
            if not separator or workspace not in contracts:
                raise ValueError('gate selection input must name a governed gate workspace')
            observed = json.loads(Path(path).read_text())
            actual = list(observed['suites']) + [workspace+'-doc']
            expanded = contracts[workspace]['expanded']
            if expanded != actual:
                errors.append('landing alias differs from the actual captured suite order: ' + workspace)
            gate_sets.append({'workspace': workspace, 'capture': path, 'expanded': expanded, 'captured': actual})
    if args.gate_command:
        for value in args.gate_command:
            workspace, separator, path = value.partition('=')
            if not separator or workspace not in contracts:
                raise ValueError('gate command input must name a governed gate workspace')
            gate_sets.append(captured_gate_command(Path(path), workspace, contracts[workspace]['expanded']))
    result = {'status': 'pass' if not errors else 'fail', 'violations': errors,
              'scope': 'Read-only literal external WORKSPACES and pure document routing; no Cargo metadata execution.',
              'external': str(args.external.resolve()), 'sha256': hashlib.sha256(args.external.read_bytes()).hexdigest(),
              'workspaces': values['WORKSPACES'], 'consumer_document_routes': routes,
              'source_gate_contracts': contracts, 'actual_gate_aliases': gate_sets}
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(result, indent=2)+'\n', encoding='utf-8')
    if not args.check_only or errors:
        print(json.dumps({'status': result['status'], 'violations': errors, 'metadata_invocations': 0,
                          'document_route_cases': len(routes), 'actual_gate_aliases': len(gate_sets)}))
    return bool(errors)


if __name__ == '__main__':
    raise SystemExit(main())
