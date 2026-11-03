"""Behaviour proofs for the sealed court door paths and the door module text."""
from pathlib import Path
import unittest

from src_court_doors import rewritten
from src_door_items import door_files, library_paths, split_module


class CourtDoorTests(unittest.TestCase):
    def test_library_paths_skip_facade_and_doors(self):
        text = ('use worth_query_execution::domain_computation::a::Item;\n'
                'use worth_query_execution::facade::x::Y;\n'
                'use worth_query_execution::court_doors_for_test::a::Z;\n'
                'fn f() { worth_query_execution::domain_computation::a::Item::new(); }\n')
        found = [r.segments for r in library_paths(Path('f.rs'), text)]
        self.assertEqual(found, [('domain_computation', 'a', 'Item'),
                                 ('domain_computation', 'a', 'Item', 'new')])

    def test_split_module_stops_at_the_item(self):
        index = {(): 0, ('domain_computation',): 0, ('domain_computation', 'a'): 0}
        self.assertEqual(split_module(index, ('domain_computation', 'a', 'Item', 'new')),
                         (('domain_computation', 'a'), ('Item', 'new')))

    def test_rewrites_route_through_the_door_or_the_facade(self):
        text = 'use worth_query_execution::domain_computation::a::Item;\nuse worth_query_execution::domain_computation::a::Pub;\n'
        refs = library_paths(Path('f.rs'), text)
        module = ('domain_computation', 'a')
        out = rewritten(text, [(refs[0], module, 'Item', ('door', module + ('Item',))),
                               (refs[1], module, 'Pub', ('public', ('facade', 'Pub')))])
        self.assertEqual(out, 'use worth_query_execution::court_doors_for_test::domain_computation::a::Item;\n'
                              'use worth_query_execution::facade::Pub;\n')
        self.assertEqual(rewritten(out, []), out)

    def test_door_files_nest_explicit_reexports(self):
        files = door_files({('domain_computation', 'a'): {'Item': (('domain_computation', 'a', 'b'), 'Item')}},
                           'court_doors_for_test')
        self.assertIn('pub mod domain_computation;', files['court_doors_for_test/mod.rs'])
        leaf = files['court_doors_for_test/domain_computation/a/mod.rs']
        self.assertIn('pub use crate::domain_computation::a::b::Item;', leaf)
        self.assertNotIn('*', leaf)


if __name__ == '__main__':
    unittest.main()
