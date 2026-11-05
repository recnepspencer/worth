"""Behaviour proofs for module coupling, the layering ratchet, crate partitions and churn scoring."""
from pathlib import Path
import shutil
import tempfile
import unittest

from coupling_graph import components, layering, upward_edges
from crate_partition import Partition, path_module
from module_coupling import coupling, freeze, ratchet, report
from split_candidates import balanced

FIXTURE = Path(__file__).parent / 'fixtures/coupling'
PARENT = ('graph',)


class CouplingTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary.name)
        shutil.copytree(FIXTURE, self.root / 'fixture')
        self.lib = self.root / 'fixture/src/lib.rs'

    def tearDown(self):
        self.temporary.cleanup()

    def test_root_reexports_are_attributed_to_the_defining_module(self):
        children, edges, kinds = coupling(self.lib, PARENT, self.root)
        self.assertEqual(children, ['alpha', 'beta', 'gamma'])
        self.assertIn(('beta', 'alpha'), edges)
        self.assertEqual({item for _source, item in edges[('beta', 'alpha')]}, {'graph::alpha::Alpha'})
        self.assertEqual(kinds['graph::alpha::Alpha'], 'struct')

    def test_test_gated_items_and_families_never_couple(self):
        _children, edges, _kinds = coupling(self.lib, PARENT, self.root)
        self.assertNotIn(('gamma', 'beta'), edges)
        self.assertEqual(sorted(edges), [('alpha', 'beta'), ('alpha', 'gamma'), ('beta', 'alpha')])

    def test_report_finds_the_cycle_and_its_upward_edge(self):
        result = report(self.lib, PARENT, self.root)
        self.assertEqual(result['components'], [['alpha', 'beta']])
        self.assertEqual(result['acyclic_modules'], 1)
        self.assertEqual(result['layering_top_to_bottom'][-1], 'gamma')
        self.assertEqual(result['upward_edges'], 1)

    def test_ratchet_accepts_the_frozen_tree_and_refuses_new_upward_edges(self):
        layering_file = freeze(report(self.lib, PARENT, self.root))
        children, edges, _kinds = coupling(self.lib, PARENT, self.root)
        self.assertEqual(ratchet(layering_file, children, edges), [])
        gamma = self.root / 'fixture/src/graph/gamma.rs'
        gamma.write_text(gamma.read_text() + '\npub(crate) fn upward() -> super::alpha::Alpha {\n'
                         '    super::alpha::Alpha(Gamma)\n}\n')
        children, edges, _kinds = coupling(self.lib, PARENT, self.root)
        refusals = ratchet(layering_file, children, edges)
        self.assertEqual(len(refusals), 1)
        self.assertIn('new upward edge gamma -> alpha', refusals[0])

    def test_ratchet_requires_the_baseline_to_shrink_with_the_cycle(self):
        layering_file = freeze(report(self.lib, PARENT, self.root))
        (low, high), = [tuple(e) for e in layering_file['upward']]
        children, edges, _kinds = coupling(self.lib, PARENT, self.root)
        edges.pop((low, high))
        refusals = ratchet(layering_file, children, edges)
        self.assertEqual(refusals, [f'baseline edge {low} -> {high} no longer exists; remove it from "upward"'])

    def test_ratchet_requires_new_modules_to_be_placed(self):
        layering_file = freeze(report(self.lib, PARENT, self.root))
        children, edges, _kinds = coupling(self.lib, PARENT, self.root)
        self.assertIn('module delta is not placed', ratchet(layering_file, children + ['delta'], edges)[0])


class GraphTests(unittest.TestCase):
    def test_components_and_layering_minimize_feedback(self):
        weights = {('a', 'b'): 5, ('b', 'a'): 1, ('b', 'c'): 2, ('d', 'a'): 1}
        self.assertEqual(components(['a', 'b', 'c', 'd'], weights), [['a', 'b'], ['c'], ['d']])
        order = layering(['a', 'b', 'c', 'd'], weights)
        self.assertEqual(order, ['d', 'a', 'b', 'c'])
        self.assertEqual(upward_edges(order, weights), [('b', 'a')])

    def test_balanced_groups_keep_order_and_count(self):
        groups = balanced(['a', 'b', 'c', 'd'], {'a': 10, 'b': 10, 'c': 10, 'd': 10}, 2)
        self.assertEqual(groups, [['a', 'b'], ['c', 'd']])


class PartitionTests(unittest.TestCase):
    NODES = [(), ('low',), ('mid',), ('top',)]
    EDGES = {(('top',), ('mid',)): {('f', 'x')}, (('mid',), ('low',)): {('f', 'y')},
             (('low',), ('mid',)): {('f', 'z')}}
    LINES = {(): 1, ('low',): 100, ('mid',): 50, ('top',): 10}

    def test_cycles_make_a_partition_invalid(self):
        candidate = {'name': 'c', 'default': 'top', 'crates': {'low': ['low'], 'mid': ['mid']}}
        partition = Partition(candidate, self.NODES, self.EDGES, self.LINES)
        self.assertEqual(partition.cycles, [['low', 'mid']])

    def test_hypothetical_order_drops_upward_edges_and_counts_them(self):
        candidate = {'name': 'c', 'default': 'top', 'crates': {'low': ['low'], 'mid': ['mid']},
                     'hypothetical_order': ['top', 'mid', 'low']}
        partition = Partition(candidate, self.NODES, self.EDGES, self.LINES)
        self.assertEqual(partition.cycles, [])
        self.assertEqual(partition.broken_references, 1)
        self.assertEqual(partition.reverse_closure({'mid'}), {'mid', 'top'})
        self.assertEqual(partition.lines, {'low': 100, 'mid': 50, 'top': 11})
        self.assertEqual(partition.parallel_width(), 1)

    def test_files_map_by_tree_then_by_location_and_manifests_touch_everything(self):
        candidate = {'name': 'c', 'default': 'top', 'crates': {'low': ['low']}}
        partition = Partition(candidate, self.NODES, {}, self.LINES)
        root = 'crate'
        self.assertEqual(partition.crates_for_file('crate/src/low/gone.rs', {}, root), {'low'})
        self.assertEqual(partition.crates_for_file('crate/src/x.rs', {'crate/src/x.rs': ('low',)}, root), {'low'})
        self.assertEqual(partition.crates_for_file('crate/Cargo.toml', {}, root), {'low', 'top'})
        self.assertEqual(path_module('crate/src/low/mod.rs', root), ('low',))
        self.assertEqual(path_module('crate/src/lib.rs', root), ())


if __name__ == '__main__':
    unittest.main()
