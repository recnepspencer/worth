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
        found = [(r.segments, r.through_door) for r in library_paths(Path('f.rs'), text)]
        self.assertEqual(found, [(('domain_computation', 'a', 'Item'), False), (('a', 'Z'), True),
                                 (('domain_computation', 'a', 'Item', 'new'), False)])

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

    def test_callers_register_door_readers_and_move_only_this_family(self):
        import tempfile
        from pathlib import Path
        from src_door_callers import CRATES, OWNER, updated
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            lib = root / OWNER / 'src'
            lib.mkdir(parents=True)
            (lib / 'a.rs').write_text('pub fn get_for_test() {}\n')
            court = root / CRATES / 'worth-query-court-execution/tests/c/domain_computation/x'
            court.mkdir(parents=True)
            (court / 'mod.rs').write_text('fn t() { get_for_test(); }\n')
            (court / 'plain.rs').write_text('fn t() {}\n')
            moved = f'{CRATES}/worth-query-court-execution/tests/c/domain_computation/x/mod.rs'
            stale = f'{OWNER} {OWNER}src/domain_computation/x/mod.rs'
            foreign = 'crates/other/ crates/other-court/tests/gone.rs'
            gone = f'{OWNER} {OWNER}src/domain_computation/y.rs'
            self.assertEqual(updated(root, [stale, foreign, gone]), [f'{OWNER} {moved}', foreign])
            kit = root / CRATES / 'worth-query-testkit-execution-k/src'
            kit.mkdir(parents=True)
            (kit / 'lib.rs').write_text('pub struct GuardForTest;\n')
            (court / 'guard.rs').write_text('fn t(_: GuardForTest) {}\n')
            guard = f'{CRATES}/worth-query-court-execution/tests/c/domain_computation/x/guard.rs'
            self.assertIn(f'{CRATES}/worth-query-testkit-execution-k/ {guard}', updated(root, []))
            (court / 'guard.rs').unlink()
            self.assertEqual(updated(root, []), [
                '# Moved worth-query-execution courts and testkits reach the library only through its sealed doors, and testkit or root-crate controls only as listed.',
                f'{OWNER} {moved}'])

    def test_inventory_classifies_doors_and_lists_authority(self):
        import tempfile
        from pathlib import Path
        from src_door_inventory import LIBRARY, inventory
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / LIBRARY / 'court_doors_for_test' / 'a').mkdir(parents=True)
            (root / LIBRARY / 'court_doors_for_test' / 'a' / 'mod.rs').write_text(
                'pub use crate::a::RunAuthority;\npub use crate::a::Plain;\n')
            (root / LIBRARY / 'a').mkdir()
            (root / LIBRARY / 'a' / 'court_doors_for_test.rs').write_text(
                'impl RunAuthority {\n    pub fn mint_for_test() -> Self {\n        Self::mint()\n    }\n}\n\n'
                'impl Plain {\n    pub fn f_for_test(&self) -> &u8 {\n        &self.f\n    }\n'
                '    pub fn new_for_test(f: u8) -> Self {\n        Self { f }\n    }\n}\n')
            data = inventory(root)
            self.assertEqual((data['item_doors'], data['member_doors']), (2, 3))
            self.assertEqual(data['member_kinds'], {'constructor': 1, 'field': 1, 'method': 1})
            self.assertEqual(data['authority_member_doors'], ['a/court_doors_for_test.rs: impl RunAuthority :: mint_for_test (method)'])
            self.assertEqual(data['authority_item_doors'], ['crate::a::RunAuthority'])

    def test_repeated_file_scope_imports_are_dropped_once(self):
        from src_door_uses import without_repeats
        text = ('use a::X;\nuse b::Y;\nuse a::X;\n#[cfg(test)]\nuse b::Y;\nmod m {\n    use a::X;\n}\n'
                'pub(crate) use c::Z;\npub(crate) use c::Z;\nuse d::*;\nuse d::*;\n')
        self.assertEqual(without_repeats(text),
                         'use a::X;\nuse b::Y;\n#[cfg(test)]\nuse b::Y;\nmod m {\n    use a::X;\n}\npub(crate) use c::Z;\n'
                         'use d::*;\nuse d::*;\n')

    def test_relocated_certification_callers_follow_their_moved_file(self):
        import tempfile
        from pathlib import Path
        from src_door_callers import CRATES, OWNER, relocated_caller
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            base = f'{CRATES}/worth-query-court-execution/tests/c/domain_computation/'
            for rel, text in [('a/b/tests/mod.rs', 'fn t() { certification_query(); }'),
                              ('x/lifecycle/lease_tests/tests/mod.rs', 'fn t() { certification_query(); }'),
                              ('x/other/lease_tests/tests/mod.rs', 'fn t() {}')]:
                (root / base / rel).parent.mkdir(parents=True, exist_ok=True)
                (root / base / rel).write_text(text)
            files = [base + r for r in ('a/b/tests/mod.rs', 'x/lifecycle/lease_tests/tests/mod.rs',
                                        'x/other/lease_tests/tests/mod.rs')]
            src = f'{OWNER}src/domain_computation/'
            self.assertEqual(relocated_caller(root, files, 'installation', src + 'a/b/tests.rs'), files[0])
            self.assertEqual(relocated_caller(root, files, 'installation', src + 'x/lease_tests/tests.rs'), files[1])
            self.assertIsNone(relocated_caller(root, files, 'installation', src + 'gone.rs'))

    def test_denial_exclusions_follow_the_moved_tests(self):
        import tempfile
        from pathlib import Path
        from src_door_callers import CRATES, MOVED_NOTE, OWNER, ROAD, relocate_denial_exclusions
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            court = root / CRATES / 'worth-query-court-execution/tests/c/domain_computation'
            (court / 'a/tests').mkdir(parents=True)
            (court / 'a/tests/mod.rs').write_text('fn t() { boundary(); access(); }')
            (root / OWNER / 'src').mkdir(parents=True)
            (root / ROAD).parent.mkdir(parents=True)
            moved = f'{CRATES}/worth-query-court-execution/tests/c/domain_computation/a/tests/mod.rs'
            (root / ROAD).write_text('\n'.join([
                '[[source_identifier_denials]]', 'exclude_paths = [', f'  "{OWNER}src/domain_computation/a/tests.rs",',
                ']', 'forbidden_identifiers = ["access"]', '[[source_identifier_denials]]', 'exclude_paths = [',
                f'  "{OWNER}src",', ']', 'forbidden_identifiers = ["boundary"]']))
            self.assertEqual(relocate_denial_exclusions(root), 2)
            text = (root / ROAD).read_text()
            self.assertIn(f'  "{moved}",\n]\nforbidden_identifiers = ["access"]', text)
            self.assertIn(f'  "{OWNER}src",\n{MOVED_NOTE}\n  "{moved}",', text)
            self.assertEqual(relocate_denial_exclusions(root), 0)

    def test_doc_links_follow_the_moved_court_file(self):
        import tempfile
        from pathlib import Path
        from src_door_callers import CRATES, DOCS, relocate_doc_links
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for crate, tail in [('worth-query-court-execution', 'tests/c'), ('worth-query-testkit-execution-k', 'src')]:
                moved = root / CRATES / crate / tail / 'domain_computation/a/walks/mod.rs'
                moved.parent.mkdir(parents=True)
                moved.write_text('')
            (root / 'docs').mkdir()
            (root / DOCS[0]).write_text('[w](../workspaces/worth-query/crates/worth-query-execution/src/domain_computation/a/walks.rs#L3)')
            self.assertEqual(relocate_doc_links(root), 1)
            self.assertEqual((root / DOCS[0]).read_text(),
                             f'[w](../{CRATES}/worth-query-court-execution/tests/c/domain_computation/a/walks/mod.rs#L3)')


if __name__ == '__main__':
    unittest.main()
