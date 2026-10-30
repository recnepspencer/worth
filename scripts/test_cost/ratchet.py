"""Deterministic build and source budgets, never elapsed-time thresholds."""
from collections import Counter
import json


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
        return [{'kind': 'unmeasured-build', 'log': lane['log']}]
    counts = Counter()
    directories, modes, violations = {}, {}, []
    for artifact in lane['artifacts']:
        # lib, libtest and each executable are distinct compiler units. A
        # package's library appearing in two directories is a duplicate graph;
        # two different binaries belonging to that package are not.
        declared = budget.get('declared_cargo_profiles')
        if declared is not None and artifact.get('cargo_profile') not in declared:
            violations.append({'kind': 'undeclared-profile', 'package': artifact['package'],
                               'cargo_profile': artifact.get('cargo_profile')})
        key = (artifact['package'], artifact['target'], tuple(artifact['kind']),
               tuple(artifact['features']), json.dumps(artifact['profile'], sort_keys=True),
               artifact.get('cargo_profile'))
        counts[key] += 1
        directories.setdefault(key, set()).add(artifact['target_dir'])
        modes.setdefault(key, set()).add(artifact['check_mode'])
    for key, count in sorted(counts.items()):
        if count > budget['max_compiles_per_unit']:
            violations.append({'kind': 'duplicate-graph', 'package': key[0], 'target': key[1],
                               'features': list(key[3]), 'profile': json.loads(key[4]),
                               'check_modes': sorted(modes[key]), 'compiles': count,
                               'cargo_profile': key[5],
                               'target_dirs': sorted(str(p) for p in directories[key])})
    return violations


def source_violations(sites):
    return [s for s in sites if s['kind'] in
            ('cargo-command', 'cargo-environment', 'cargo-metadata', 'trybuild')]


def check(census, budget):
    return {'source': source_violations(census['source_sites']),
            'build': {name: build_violations(lane, budget)
                      for name, lane in census['lanes'].items()},
            'coverage': [] if census['coverage']['status'] == 'pass' else [census['coverage']['status']]}
