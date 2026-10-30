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
    return {'max_compiles_per_unit': min(governed['lanes'][name]['max_compiles_per_unit'] for name in names)}


def build_violations(lane, budget):
    if lane['build_observation'] != 'json':
        return [{'kind': 'unmeasured-build', 'log': lane['log']}]
    counts = Counter()
    directories = {}
    for artifact in lane['artifacts']:
        # lib, libtest and each executable are distinct compiler units. A
        # package's library appearing in two directories is a duplicate graph;
        # two different binaries belonging to that package are not.
        key = (artifact['package'], artifact['target'], tuple(artifact['kind']),
               tuple(artifact['features']), json.dumps(artifact['profile'], sort_keys=True),
               artifact['check_mode'])
        counts[key] += 1
        directories.setdefault(key, set()).add(artifact['target_dir'])
    violations = []
    for key, count in sorted(counts.items()):
        if count > budget['max_compiles_per_unit']:
            violations.append({'kind': 'duplicate-graph', 'package': key[0], 'target': key[1],
                               'features': list(key[3]), 'profile': json.loads(key[4]),
                               'check_mode': key[5], 'compiles': count,
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
