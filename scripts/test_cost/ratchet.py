"""Deterministic build and source budgets, never elapsed-time thresholds."""
from collections import Counter


def lane_budget(configuration, workspace, lanes, profile):
    governed = configuration[workspace]
    if profile not in governed['declared_cargo_profiles']:
        raise ValueError(f'undeclared {workspace} build profile: {profile}')
    names = list(lanes) or ['selection']
    missing = sorted(set(names) - set(governed['lanes']))
    if missing:
        raise ValueError(f'undeclared {workspace} landing lanes: {missing}')
    return {'max_compiles_per_unit': min(governed['lanes'][name]['max_compiles_per_unit'] for name in names),
            'declared_cargo_profiles': governed['declared_cargo_profiles']}


def build_violations(lane, budget):
    if lane['build_observation'] != 'json':
        return list(lane.get('compiler_attempt_refusals', [])) or [{'kind': 'unmeasured-build', 'log': lane['log']}]
    counts = Counter()
    directories, units, compiler_inputs = {}, {}, {}
    violations = list(lane.get('compiler_attempt_refusals', []))
    for artifact in lane['artifacts'] + lane.get('failed_compiler_attempts', []):
        # Identity is declared from Cargo inputs before rustc executes. Output
        # hashes and target directories never mint an additional budget unit.
        declared = budget.get('declared_cargo_profiles')
        if declared is not None and artifact.get('cargo_profile') not in declared:
            violations.append({'kind': 'undeclared-profile', 'package': artifact['package'],
                               'cargo_profile': artifact.get('cargo_profile')})
        unit = artifact.get('unit')
        if unit is None or artifact.get('unit_refusal'):
            violations.append({'kind': 'undeclared-compiler-unit', 'package': artifact['package'],
                               'target': artifact['target'], 'reason': artifact.get('unit_refusal')})
            continue
        key = unit['identity']
        counts[key] += 1
        units[key] = artifact
        directories.setdefault(key, set()).add(artifact['target_dir'])
        compiler_inputs.setdefault(key, set()).add(unit['compiler_inputs'])
    for key, count in sorted(counts.items()):
        artifact = units[key]
        if len(compiler_inputs[key]) > 1:
            violations.append({'kind': 'unplanned-compiler-input-drift', 'unit': key,
                               'package': artifact['package'], 'target': artifact['target']})
        if count > budget['max_compiles_per_unit']:
            violations.append({'kind': 'duplicate-graph', 'unit': key,
                               'package': artifact['package'], 'target': artifact['target'],
                               'features': artifact['features'], 'profile': artifact['unit']['declared_profile'],
                               'roles': artifact['unit']['roles'],
                               'required_output': artifact['unit']['output'], 'compiles': count,
                               'cargo_profile': artifact.get('cargo_profile'),
                               'target_dirs': sorted(str(p) for p in directories[key])})
    return violations


def compiler_observations(lanes, log):
    lanes = list(lanes)
    return {'build_observation': 'json' if any(lane['build_observation'] == 'json' for lane in lanes) else 'unmeasured',
            'log': str(log), **{field: [row for lane in lanes for row in lane.get(field, [])]
                for field in ('artifacts', 'failed_compiler_attempts', 'compiler_attempt_refusals')}}


def source_violations(sites):
    return [s for s in sites if s['kind'] in
            ('cargo-command', 'cargo-environment', 'cargo-metadata', 'trybuild')]


def check(census, budget):
    return {'source': source_violations(census['source_sites']),
            'build': {name: build_violations(lane, budget)
                      for name, lane in census['lanes'].items()},
            'coverage': [] if census['coverage']['status'] == 'pass' else [census['coverage']['status']]}
