import unittest
from trybuild_calls import rewrite


class CompilerSessionCutoverTests(unittest.TestCase):
    def test_denied_fixture_keeps_its_exact_input_and_a_fallible_observation(self):
        source = '#[test] fn boundary() { let cases = trybuild::TestCases::new(); cases.compile_fail("bad.rs"); }'
        result = rewrite(source)
        self.assertNotIn('trybuild', result.text)
        self.assertIn('fixture_outcomes::denied("bad.rs").expect', result.text)
        self.assertEqual(result.denials, 1)

    def test_other_pass_methods_are_untouched(self):
        source = '#[test] fn verdict() { let answer = domain(); answer.pass(value); }'
        self.assertEqual(rewrite(source).text, source)

    def test_retired_compiler_session_import_is_removed(self):
        self.assertNotIn('trybuild', rewrite('pub(crate) use {trybuild::TestCases};').text)

    def test_removing_the_only_pass_runner_import_leaves_no_empty_use_item(self):
        for declaration in ('use owner::run_compile_pass_bundle;',
                            'pub(crate) use {owner::run_compile_pass_bundle};',
                            '#[allow(unused_imports)]\npub use owner::run_compile_pass_bundle;'):
            with self.subTest(declaration=declaration):
                self.assertEqual(rewrite(declaration).text.strip(), '')

    def test_pass_only_test_is_moved_instead_of_becoming_an_empty_green(self):
        result = rewrite('#[test] fn supported() { let c = trybuild::TestCases::new(); c.pass("yes.rs"); }')
        self.assertEqual(result.moved_tests, ['supported'])
        self.assertNotIn('fn supported', result.text)
        self.assertEqual(result.pass_calls[0]['argument'], '"yes.rs"')

    def test_temporary_compiler_session_is_removed(self):
        result = rewrite('#[test] fn sealed() { trybuild::TestCases::new().compile_fail("bad.rs"); }')
        self.assertNotIn('trybuild', result.text)
        self.assertEqual(result.denials, 1)

    def test_absence_claim_becomes_an_unconditional_named_production_test(self):
        result = rewrite('#[test] fn boundaries() { let c=trybuild::TestCases::new(); '
                         '#[cfg(not(all()))] c.compile_fail("certification_authority_is_absent.rs"); }')
        self.assertIn('fn certification_authority_is_absent_in_the_production_environment', result.text)
        self.assertNotIn('cfg(not(all()))', result.text)
        self.assertIn('production_denial', result.text)
        self.assertNotIn('fn boundaries()', result.text)


if __name__ == '__main__':
    unittest.main()
