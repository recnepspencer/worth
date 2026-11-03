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

    def test_closure_arrows_do_not_close_brackets(self):
        self.assertEqual(call_parts('(&self, f: impl FnMut(u8) -> Result<u8, E>, b: X) -> R'), ('', ('self', ['f', 'b'])))

    def test_method_attributes_do_not_hide_the_method(self):
        body = '\n    #[doc(hidden)]\n    #[expect(clippy::x, reason = "y")]\n    pub(crate) fn of<T: ?Sized + \'static>() -> Self { todo!() }\n'
        door = method_door(('', '', 'Id', '', body), 'of')
        self.assertIn("pub fn of_for_test<T: ?Sized + 'static>() -> Self", door)
        self.assertIn('Self::of::<T>()', door)

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

    def test_keywords_end_the_postfix_expression(self):
        text = b'std::mem::take(&mut state.records.get_mut(&k).unwrap().prerequisites)'
        at = text.index(b'prerequisites')
        self.assertEqual(text[expression_start(text, at):at], b'state.records.get_mut(&k).unwrap().')

    def test_line_comments_do_not_join_the_expression(self):
        text = b'// before retirement.\n    state\n        .records'
        at = text.index(b'records')
        self.assertEqual(text[expression_start(text, at):at], b'state\n        .')

    def test_assignment_needs_the_mutable_door(self):
        import tempfile
        from pathlib import Path
        with tempfile.TemporaryDirectory() as d:
            p = Path(d) / 'f.rs'
            p.write_text('a.f = 1; a.g == 2; a.h += 3;')
            self.assertTrue(writes(p, 3))
            self.assertFalse(writes(p, 12))
            self.assertTrue(writes(p, 22))



class DoorBodyTests(unittest.TestCase):
    def test_only_generated_door_rewrites_are_reversed(self):
        from src_door_bodies import without_doors
        renames = {'get_for_test': ('get', 'method'), 'f_for_test': ('f', 'field')}
        ours = ['let', 'x', '=', '(', '*', 'a', '.', 'f_for_test', '(', ')', ')', ';', 'a', '.', 'get_for_test', '(', ')']
        self.assertEqual(without_doors(ours, renames),
                         ['let', 'x', '=', 'a', '.', 'f', ';', 'a', '.', 'get', '(', ')'])
        orders = {'P': (True, ['x', 'y'])}
        ours = ['P', ':', ':', 'new_for_test', '(', 'a', ',', 'f', '(', '1', ')', ')']
        original = ['P', '{', 'x', ':', 'a', ',', 'y', ':', 'f', '(', '1', ')', ',', '}']
        self.assertEqual(without_doors(ours, renames, orders), without_doors(original, renames, orders))
        swapped = ['P', '{', 'y', ':', 'f', '(', '1', ')', ',', 'x', ':', 'a', '}']
        self.assertNotEqual(without_doors(ours, renames, orders), without_doors(swapped, renames, orders))
        other = ['a', '.', 'install_for_test', '(', ')']
        self.assertEqual(without_doors(other, renames), other)


if __name__ == '__main__':
    unittest.main()
