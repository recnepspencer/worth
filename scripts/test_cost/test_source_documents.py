from pathlib import Path
import tempfile
import unittest

from source_documents import documents


class TestSourceOwnershipTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name).resolve()
        self.package = {'name': 'product', 'manifest': self.root / 'Cargo.toml'}

    def observe(self, files, testing=False):
        for relative, text in files.items():
            path = self.root / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(text)
        return documents(self.package, [(self.root / 'lib.rs', testing)])

    def test_enclosing_context_survives_without_copying_production_bodies_into_tests(self):
        rows = self.observe({'lib.rs': 'use std::process::Command as Process;\n'
                             'const EXECUTABLE:&str="cargo";\nfn ordinary() { loop {} }\n'
                             '#[cfg(test)] mod tests { #[test] fn proof() { Process::new(EXECUTABLE); } }'})
        self.assertEqual(len(rows), 1)
        self.assertIn('Command as Process', rows[0].context)
        self.assertIn('"cargo"', rows[0].context)
        self.assertNotIn('loop {}', rows[0].text)

    def test_external_test_modules_retain_their_ancestor_context(self):
        rows = self.observe({'lib.rs': 'use std::process::Command as Process;\n#[cfg(test)] mod tests;',
                             'tests.rs': '#[test] fn proof() { Process::new("cargo"); }'})
        child = next(row for row in rows if row.path.name == 'tests.rs')
        self.assertIn('Command as Process', child.context)
        self.assertIn('Process::new', child.text)

    def test_literal_included_tests_and_nested_includes_keep_real_paths(self):
        rows = self.observe({'lib.rs': '#[cfg(test)] mod tests { include!("included.rs"); }',
                             'included.rs': '#[test] fn first() {}\ninclude!("nested.rs");',
                             'nested.rs': '#[test] fn second() { thread::sleep(delay); }'})
        self.assertEqual({row.path.name for row in rows}, {'lib.rs', 'included.rs', 'nested.rs'})
        self.assertIn('thread::sleep', next(row.text for row in rows if row.path.name == 'nested.rs'))

    def test_named_ordinary_helpers_referenced_by_tests_are_selected(self):
        rows = self.observe({'lib.rs': 'fn unrelated() { loop {} }\n'
                             'fn prepare() { compile(); }\nfn compile() { Command::new("cargo"); }\n'
                             '#[cfg(test)] mod tests { #[test] fn proof() { prepare(); } }'})
        self.assertIn('Command::new', rows[0].text)
        self.assertNotIn('loop {}', rows[0].text)

    def test_selected_helpers_promote_their_included_expressions(self):
        rows = self.observe({'lib.rs': 'fn prepare() { include!("expression.rs"); }\n'
                             '#[cfg(test)] mod tests { #[test] fn proof() { prepare(); } }',
                             'expression.rs': 'Command::new("cargo")'})
        self.assertIn('Command::new', next(row.text for row in rows if row.path.name == 'expression.rs'))

    def test_referenced_associated_methods_enter_the_same_test_closure(self):
        rows = self.observe({'lib.rs': 'struct Compiler; impl Compiler {\n'
                             'fn launch() { Command::new("cargo"); }\n'
                             'fn unrelated() { loop {} } }\n'
                             '#[cfg(test)] mod tests { #[test] fn proof() { Compiler::launch(); } }'})
        self.assertIn('Command::new', rows[0].text)
        self.assertNotIn('loop {}', rows[0].text)

    def test_referenced_trait_default_methods_enter_the_test_closure(self):
        rows = self.observe({'lib.rs': 'trait Compiler { fn launch() { Command::new("cargo"); } }\n'
                             'struct Driver; impl Compiler for Driver {}\n'
                             '#[cfg(test)] mod tests { #[test] fn proof() { Driver::launch(); } }'})
        self.assertIn('Command::new', rows[0].text)

    def test_unresolved_test_include_refuses_instead_of_claiming_no_sites(self):
        with self.assertRaisesRegex(ValueError, 'unresolved include'):
            self.observe({'lib.rs': '#[cfg(test)] mod tests { include!(generated_path!()); }'})

    def test_missing_and_cyclic_includes_refuse(self):
        with self.assertRaises(FileNotFoundError):
            self.observe({'lib.rs': '#[cfg(test)] mod tests { include!("missing.rs"); }'})
        with self.assertRaisesRegex(ValueError, 'include cycle'):
            self.observe({'lib.rs': '#[cfg(test)] mod tests { include!("lib.rs"); }'})

    def test_unselected_expression_includes_do_not_become_test_claims(self):
        rows = self.observe({'lib.rs': 'fn ordinary() { include!("expression.rs"); }\n'
                             '#[cfg(test)] mod tests { #[test] fn proof() {} }',
                             'expression.rs': '[1, 2, 3]'})
        self.assertNotIn('expression.rs', {row.path.name for row in rows})


if __name__ == '__main__':
    unittest.main()
