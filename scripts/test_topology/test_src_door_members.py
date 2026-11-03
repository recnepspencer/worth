"""Behaviour proofs for the sealed member doors and their call-site rewrites."""
import unittest

from src_door_members import expression_start, writes
from src_door_members_text import call_parts, constructor_door, field_door, method_door, struct_shape


class MemberDoorTests(unittest.TestCase):
    def test_method_door_repeats_the_signature_and_delegates(self):
        body = '\n    pub(crate) fn get<T: Copy>(&self, a: T, (b, c): (u8, u8)) -> T { a }\n'
        door = method_door(('', '<S: X>', 'Foo<S>', 'where S: Y', body), 'get')
        self.assertIn('impl<S: X> Foo<S> where S: Y {', door)
        self.assertIn('pub fn get_for_test<T: Copy>(&self, a: T, _arg2: (u8, u8)) -> T {', door)
        self.assertIn('Self::get::<T>(self, a, _arg2)', door)
        self.assertIsNone(method_door(('', '', 'Foo', '', body), 'missing'))

    def test_impl_trait_arguments_infer_their_generics(self):
        self.assertEqual(call_parts('<F>(f: impl Fn(u8)) -> u8'), ('', (None, ['f'])))

    def test_field_and_constructor_doors(self):
        shape = struct_shape('pub struct P<T> where T: Clone {\n    a: u8,\n    pub(crate) b: Vec<T>,\n}')
        self.assertEqual(shape['fields'], {'a': 'u8', 'b': 'Vec<T>'})
        self.assertIn('pub fn b_for_test(&self) -> &Vec<T> {\n        &self.b', field_door(shape, 'b', False))
        self.assertIn('pub fn a_mut_for_test(&mut self) -> &mut u8', field_door(shape, 'a', True))
        self.assertIn('Self { a, b }', constructor_door(shape))
        tuple = constructor_door(struct_shape('pub struct R(pub(super) u64, String);'))
        self.assertIn('pub fn new_for_test(field0: u64, field1: String) -> Self', tuple)

    def test_field_reads_wrap_the_whole_postfix_expression(self):
        text = b'let x = &world.inner(1)[0].field;'
        self.assertEqual(text[expression_start(text, text.index(b'field')):text.index(b'field')],
                         b'world.inner(1)[0].')
        text = b'f(a, crate::m::X::get().field)'
        self.assertEqual(text[expression_start(text, text.index(b'field')):], b'crate::m::X::get().field)')

    def test_assignment_needs_the_mutable_door(self):
        import tempfile
        from pathlib import Path
        with tempfile.TemporaryDirectory() as d:
            p = Path(d) / 'f.rs'
            p.write_text('a.f = 1; a.g == 2; a.h += 3;')
            self.assertTrue(writes(p, 3))
            self.assertFalse(writes(p, 12))
            self.assertTrue(writes(p, 22))


if __name__ == '__main__':
    unittest.main()
