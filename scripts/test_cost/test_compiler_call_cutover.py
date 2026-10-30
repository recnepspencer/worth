import unittest

from compiler_call_cutover import rewrite


class CompilerCallCutoverTests(unittest.TestCase):
    def test_environment_selection_precedes_the_reader_and_drops_build_controls(self):
        text = '''fn boundary() { run_cargo_ui_fixture_suite(root, "suite", manifest(root),
                 "old-feature", "diagnostic-test", source, FIXTURES).unwrap(); }'''
        changed, suites = rewrite(text, {'suite': 'production'})
        self.assertEqual(suites, ['suite'])
        self.assertIn('read_ui_fixture_suite(root, "suite", UiCompilerEnvironment::Production, source, FIXTURES)', changed)
        self.assertNotIn('manifest(root)', changed)
        self.assertNotIn('diagnostic-test', changed)

    def test_wrong_argument_shape_is_refused_before_source_mutation(self):
        with self.assertRaises(ValueError):
            rewrite('fn a() { run_cargo_ui_fixture_suite(root); }', {})

    def test_qualified_manifest_import_is_removed_without_leaving_an_orphan_prefix(self):
        text = 'pub(crate) use {owner::cargo_dependency_manifest, owner::run_cargo_ui_fixture_suite};'
        changed, _ = rewrite(text, {})
        self.assertEqual(changed, 'pub(crate) use {owner::read_ui_fixture_suite};')


if __name__ == '__main__':
    unittest.main()
