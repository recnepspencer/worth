"""Behaviour proofs for module coupling, the layering ratchet, crate partitions and churn scoring."""
from pathlib import Path
import shutil
import tempfile
import unittest

from coupling_graph import components, layering, upward_edges
from crate_partition import Partition, path_module
import contextlib
import io
import json

from module_coupling import check_all, coupling, freeze, ratchet, report
from split_candidates import balanced
from module_graph import LibraryModel
from ownership_units import Classifier, header_parts, units

FIXTURE = Path(__file__).parent / 'fixtures/coupling'
OWNERSHIP = Path(__file__).parent / 'fixtures/ownership'
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


    def test_strict_layering_with_no_upward_edges_refuses_any_upward_edge(self):
        strict = freeze(report(self.lib, PARENT, self.root))
        (low, high), = [tuple(e) for e in strict['upward']]
        strict['upward'] = []
        children, edges, _kinds = coupling(self.lib, PARENT, self.root)
        refusals = ratchet(strict, children, edges)
        self.assertEqual(len(refusals), 1)
        self.assertIn(f'new upward edge {low} -> {high}', refusals[0])
        edges.pop((low, high))
        self.assertEqual(ratchet(strict, children, edges), [])

    def write_layering(self, name, layering_file):
        path = self.root / name
        path.write_text(json.dumps(layering_file), encoding='utf-8')
        return path

    def run_check(self, *paths):
        out, err = io.StringIO(), io.StringIO()
        with contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
            code = check_all(self.lib, list(paths), self.root)
        return code, out.getvalue().splitlines(), err.getvalue().splitlines()

    def test_check_verifies_several_layering_files_in_one_run(self):
        frozen = freeze(report(self.lib, PARENT, self.root))
        strict = dict(frozen, upward=[])
        good = self.write_layering('good.json', frozen)
        bad = self.write_layering('strict.json', strict)
        code, out, err = self.run_check(good)
        self.assertEqual((code, out, err), (0, ['{"refusals": 0, "upward_edges": 1}'], []))
        code, out, err = self.run_check(good, good)
        self.assertEqual((code, len(out)), (0, 2))
        self.assertEqual(json.loads(out[0])['parent'], 'graph')
        code, out, err = self.run_check(good, bad)
        self.assertEqual(code, 1)
        self.assertEqual([json.loads(line)['refusals'] for line in out], [0, 1])
        self.assertTrue(err[0].startswith('[graph] new upward edge '))

    def test_check_refuses_a_missing_layering_file(self):
        code, out, err = self.run_check(self.root / 'absent.json')
        self.assertEqual((code, out), (2, []))
        self.assertIn('does not exist', err[0])


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


class OwnershipTests(unittest.TestCase):
    """A type moves with every inherent impl and foreign-trait impl, wherever they live."""

    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary.name)
        shutil.copytree(OWNERSHIP, self.root / 'fixture')
        self.model = LibraryModel(self.root / 'fixture/src/lib.rs', self.root)
        self.units, _orphans = units(self.model)

    def tearDown(self):
        self.temporary.cleanup()

    def test_impl_headers_name_trait_and_self_type(self):
        generics, trait, self_type, _header = header_parts('impl<T: Clone> std::fmt::Debug for Wrapper<T> {}')
        self.assertEqual((generics, trait, self_type), ({'T'}, ('std', 'fmt', 'Debug'), ('Wrapper',)))
        self.assertEqual(header_parts('impl Plain {}')[1:3], (None, ('Plain',)))

    def test_units_gather_impls_from_every_module(self):
        thing = self.units[(('graph', 'high'), 'Thing')]
        self.assertEqual(sorted(i.module for i in thing.impls), [('graph', 'side'), ('graph', 'side')])

    def test_a_unit_moves_only_when_every_impl_needs_nothing_above(self):
        classifier = Classifier(self.model, ('graph',), ['low', 'side', 'high'], self.units)
        self.assertEqual(classifier.judge((('graph', 'high'), 'Plain'), 'low')[0], 'movable')
        verdict, detail = classifier.judge((('graph', 'high'), 'Thing'), 'low')
        self.assertEqual(verdict, 'behaviour-coupled')
        self.assertIn('graph::high::measure', {item for _label, _node, item, _kind in detail})
