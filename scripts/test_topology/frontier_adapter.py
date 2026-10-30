"""Use A2's existing Python graph calculation for virtual court targets.

The isolated reporter module supplies the closure/maximal-products algorithm.
Its source-size callback is disabled here because this adapter measures graphs;
the codemod counts the actual files after assigning ownership.
"""
import copy
import importlib.util
from pathlib import Path


def package_roles(metadata):
    path = Path(__file__).with_name('product_roles.py')
    spec = importlib.util.spec_from_file_location('_a2_product_roles', path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    # Existing court names certify production roots after tests leave their owners.
    rows = graph_reporter().measure(metadata)['packages']
    established = set()
    for row in rows:
        if row['role'] == 'court':
            stem, label = row['package'].split('-court-', 1)
            frontier = row['product_frontier']
            if label == surface(stem, frontier) or label.startswith(surface(stem, frontier) + '-'):
                established.update(frontier)
    return module.classify(metadata, established)


def graph_reporter():
    report_path = Path(__file__).resolve().parents[1] / 'ci' / 'report_test_topology.py'
    spec = importlib.util.spec_from_file_location('_a2_topology_graph', report_path)
    reporter = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(reporter)
    reporter.rust_lines = lambda _directory: 0
    return reporter


def frontiers(metadata, selections):
    reporter = graph_reporter()
    data = copy.deepcopy(metadata)
    roles = package_roles(metadata)
    data['workspace_members'] = [identity for identity in data['workspace_members']
                                 if roles[identity] in ('product', 'court')]
    for key, package_ids in sorted(selections.items()):
        identity = 'codemod-court-' + key
        data['workspace_members'].append(identity)
        data['packages'].append({'id': identity, 'name': identity, 'manifest_path': '/virtual/Cargo.toml',
                                 'targets': []})
        data['resolve']['nodes'].append({'id': identity, 'deps': [
            {'pkg': dep, 'name': dep, 'dep_kinds': [{'kind': None, 'target': None}]}
            for dep in sorted(package_ids)]})
    rows = reporter.measure(data)['packages']
    return {row['id'].removeprefix('codemod-court-'): row['product_frontier']
            for row in rows if row['id'].startswith('codemod-court-')}


def surface(stem, frontier):
    """boundary-check graph_rules/frontier.rs::surface, with sorted-name input.

    This is the Python spelling of that naming function, not a frontier algorithm.
    The Rust function remains the authority; its examples are exercised by fixtures.
    """
    if not frontier:
        return 'source'
    prefix = stem.removeprefix('worth-query-')
    return '-'.join(name.removeprefix(prefix + '-') if name.startswith(prefix + '-')
                    else name.rsplit('-', 1)[-1] if name == prefix else name
                    for name in sorted(frontier))
