"""Behaviour proofs for the sealed member doors and their call-site rewrites."""
import unittest

from src_door_sites import Source, assigned_after, borrowed_mut, field_sites
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
        cases = [
            ('let x = &world.inner(1)[0].field;', 'world.inner(1)[0].'),
            ('f(a, crate::m::X::get().field)', 'crate::m::X::get().'),
            ('std::mem::take(&mut state.records.get_mut(&k).unwrap().prerequisites)', 'state.records.get_mut(&k).unwrap().'),
            ('// before retirement.\n    state\n        .records', 'state\n        .'),
            ('if x { y(1) }\n    state.records.get_mut(&k).unwrap().state', 'state.records.get_mut(&k).unwrap().'),
            ('let s = "a.b"; (a + b).field', '(a + b).'),
        ]
        for text, prefix in cases:
            at = text.rindex(prefix.split('.')[0]) if False else None
            field_at = len(text) - len(text.split('.')[-1])
            source = Source(text)
            located = field_sites(source, field_at)
            self.assertIsNotNone(located, text)
            self.assertEqual(text[located[0]:field_at], prefix, text)

    def test_tuple_indexes_stay_in_the_postfix_expression(self):
        from src_door_sites import postfix_start
        source = Source('let s = result.0.state.lock();')
        dot = [k for k, tok in enumerate(source.stream) if tok.value == 'state'][0] - 1
        self.assertEqual(source.value(postfix_start(source, dot)), 'result')
        source = Source('let p = any.downcast_ref::<Set<u8, Box<dyn Fn() -> u8>>>().unwrap().partitions;')
        dot = [k for k, tok in enumerate(source.stream) if tok.value == 'partitions'][0] - 1
        self.assertEqual(source.value(postfix_start(source, dot)), 'any')

    def test_assignment_and_mutable_borrow(self):
        for text, expected in (('a.f = 1;', True), ('a.g == 2;', False), ('a.h += 3;', True),
                               ('a.k[0] ^= 1;', True), ('a.m <<= 1;', True)):
            source = Source(text)
            field = source.index(2)
            self.assertEqual(assigned_after(source, field), expected, text)
        source = Source('take(&mut a.b)')
        self.assertTrue(borrowed_mut(source, source.index(10)))


class LiteralSiteTests(unittest.TestCase):
    def test_literals_patterns_and_functional_updates(self):
        from src_door_sites import enclosing_brace, is_pattern, literal_parts, path_before
        text = 'let r = m::T { a: f(1, 2), b, ..base(x) };'
        source = Source(text)
        brace = enclosing_brace(source, source.index(text.index('a:')))
        first, path = path_before(source, brace)
        self.assertEqual(path, 'm::T')
        self.assertEqual(literal_parts(source, brace), ([('a', 'f(1, 2)'), ('b', None)], 'base(x)'))
        self.assertFalse(is_pattern(source, first, brace))
        pattern = Source('let T { a, b } = v;')
        brace = enclosing_brace(pattern, pattern.index(8))
        self.assertTrue(is_pattern(pattern, path_before(pattern, brace)[0], brace))


class DoorBodyTests(unittest.TestCase):
    def test_impl_headers_drop_generic_defaults(self):
        from src_door_members_text import impl_header, without_defaults
        self.assertEqual(without_defaults('<T, U = Vec<u8>, const N: usize = 3>'), '<T, U, const N: usize>')
        self.assertEqual(without_defaults('<F: Fn(u8) -> u8, S = ()>'), '<F: Fn(u8) -> u8, S>')
        self.assertEqual(impl_header('<R = Ready>', 'Set<R>', ''), 'impl<R> Set<R> {\n')

    def test_doors_are_keyed_by_exact_header_and_never_repeated(self):
        import tempfile
        from pathlib import Path
        from src_door_writes import door_keys, normalized_doors
        one = 'impl<R = A> Set<R> {\n    pub fn get_for_test(&self) {}\n}\n'
        two = 'impl Set<B> {\n    pub fn get_for_test(&self) {}\n}\n'
        self.assertEqual(door_keys(one), {(None, '< R > Set < R >', 'get_for_test')})
        # rustfmt drops a trailing comma and reflows: the key does not change.
        self.assertEqual(door_keys('impl<R,> Set<R> {\n    pub fn get_for_test(&self) {}\n}\n'),
                         door_keys('impl<R>\n    Set<R>\n{\n    pub fn get_for_test(&self) {}\n}\n'))
        self.assertNotEqual(door_keys(one), door_keys(two))
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / 'a').mkdir()
            (root / 'set.rs').write_text('pub struct Set<R>(R);\n')
            (root / 'a' / 'court_doors_for_test.rs').write_text(one)
            (root / 'b_court_doors_for_test.rs').write_text(one + '\n' + two)
            # A different `Set` in its own module keeps its door.
            (root / 'other').mkdir()
            (root / 'other' / 'set.rs').write_text('pub struct Set<R>(R);\n')
            (root / 'other' / 'court_doors_for_test.rs').write_text(one.replace('<R = A>', '<R>'))
            changed = normalized_doors(root)
            self.assertEqual(changed[root / 'a' / 'court_doors_for_test.rs'], one.replace('<R = A>', '<R>'))
            self.assertEqual(changed[root / 'b_court_doors_for_test.rs'], '\n' + two)
            self.assertNotIn(root / 'other' / 'court_doors_for_test.rs', changed)
            for path, text in changed.items():
                path.write_text(text)
            self.assertEqual(normalized_doors(root), {})

    def test_body_check_reads_renames_from_the_door_files(self):
        import tempfile
        from pathlib import Path
        from src_door_bodies import LIBRARY, door_renames
        doors = ('impl A {\n    #[doc(hidden)]\n    pub fn get_for_test(&self) -> u8 {\n        Self::get(self)\n    }\n'
                 '    pub fn f_mut_for_test(&mut self) -> &mut u8 {\n        &mut self.f\n    }\n'
                 '    pub fn s_into_for_test(self) -> u8 {\n        self.s\n    }\n'
                 '    pub fn into_parts_for_test(self) -> (u8, u8) {\n        (self.f, self.s)\n    }\n}\n')
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / LIBRARY / 'm').mkdir(parents=True)
            (root / LIBRARY / 'm' / 'x_court_doors_for_test.rs').write_text(doors)
            self.assertEqual(door_renames(root), {'get_for_test': ('get', 'method'), 'f_mut_for_test': ('f', 'field'),
                                                  's_into_for_test': ('s', 'into')})

    def test_missing_door_messages_name_the_door_and_type(self):
        from src_door_missing import MISSING
        cases = {"no method named `work_for_test` found for mutable reference `&mut Read<'_, S>` in the current scope":
                 ('work', 'Read'),
                 'no function or associated item named `mint_for_test` found for struct `a::b::Authority`':
                 ('mint', 'Authority'),
                 'no method named `kind` found for type `!` in the current scope': None}
        for message, expected in cases.items():
            match = MISSING.search(message)
            self.assertEqual(match.groups() if match else None, expected)

    def test_missing_doors_read_as_fields_only_behind_the_field_wrapper(self):
        from src_door_missing import field_site
        source = Source('let a = (*reader.work_for_test()).reads(); let b = reader.work_for_test();')
        sites = [tok.start for tok in source.stream if tok.value == 'work_for_test']
        self.assertEqual([field_site(source, (s, s + 13)) for s in sites], [True, False])

    def test_mutable_reads_switch_the_outermost_door(self):
        from src_door_members import read_door_in
        source = Source('(*(*a.x_for_test()).y_for_test()).push(1);')
        found = read_door_in(source, (0, len('(*(*a.x_for_test()).y_for_test())')))
        self.assertEqual(found[1], 'y')

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
