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


if __name__ == '__main__':
    unittest.main()
