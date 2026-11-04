"""Behaviour proofs for destructuring doors."""
import unittest

from src_door_members_text import struct_shape
from src_door_parts import parts_door


class PartsDoorTests(unittest.TestCase):
    def test_parts_door_returns_every_field_in_declaration_order(self):
        shape = struct_shape('pub struct R<T> where T: Clone {\n    b: Vec<T>,\n    pub(crate) a: u8,\n}')
        door = parts_door(shape)
        self.assertIn('impl<T> R<T> where T: Clone {', door)
        self.assertIn('pub fn into_parts_for_test(self) -> (Vec<T>, u8,) {', door)
        self.assertIn('(self.b, self.a,)', door)

    def test_let_patterns_read_in_declaration_order(self):
        from src_door_patterns import pattern_entries, simple_postfix, statement_end
        from src_door_sites import Source
        source = Source('let T { b: x, mut a, .. } = run.take()?; next();')
        brace = [k for k, tok in enumerate(source.stream) if tok.value == '{'][0]
        self.assertEqual(pattern_entries(source, brace), ({'b': 'x', 'a': 'mut a'}, True))
        first = source.close[brace] + 2
        end = statement_end(source, first)
        self.assertEqual(source.value(end), ';')
        self.assertTrue(simple_postfix(source, first, end - 1))
        source = Source('let T { a } = x + y;')
        brace = [k for k, tok in enumerate(source.stream) if tok.value == '{'][0]
        first = source.close[brace] + 2
        self.assertFalse(simple_postfix(source, first, statement_end(source, first) - 1))

    def test_constructor_orders_read_function_pointer_types(self):
        import tempfile
        from pathlib import Path
        from src_door_bodies import LIBRARY, constructor_orders
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / LIBRARY / 'e').mkdir(parents=True)
            (root / LIBRARY / 'e' / 'court_doors_for_test.rs').write_text(
                'impl<S> E<S> {\n    #[doc(hidden)]\n    pub fn new_for_test(\n        a: u8,\n'
                '        _marker: PhantomData<fn() -> S>,\n        b: u8,\n    ) -> Self {\n        Self { a, _marker, b }\n    }\n}\n')
            self.assertEqual(constructor_orders(root), {'E': (True, ['a', '_marker', 'b'])})

    def test_body_check_reads_shared_door_names_by_site_and_ignores_trailing_commas(self):
        from src_door_bodies import without_doors
        renames = {'state_for_test': ('state', 'field+method')}
        field = ['(', '*', 'a', '.', 'state_for_test', '(', ')', ')', '.', 'f', '(', 'x', ',', ')']
        method = ['a', '.', 'state_for_test', '(', ')']
        self.assertEqual(without_doors(field, renames), ['a', '.', 'state', '.', 'f', '(', 'x', ')'])
        self.assertEqual(without_doors(method, renames), ['a', '.', 'state', '(', ')'])


if __name__ == '__main__':
    unittest.main()
