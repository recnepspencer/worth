import unittest
from executable_cutover import rewrite


class ExecutableCutoverTests(unittest.TestCase):
    def test_only_declared_independent_executable_lookups_gain_refusal_handling(self):
        text = 'let binary = worth_store_test_support::FRESH_PROCESS_RECOVERY.path();\nother.path();'
        changed, count = rewrite(text)
        self.assertEqual(count, 1)
        self.assertIn('.path().expect(', changed)
        self.assertTrue(changed.endswith('other.path();'))

    def test_named_provider_handles_a_multiline_lookup(self):
        changed, count = rewrite('let path = executable\n .path()\n .canonicalize();', True)
        self.assertEqual(count, 1)
        self.assertIn('.path().expect(', changed)


if __name__ == '__main__':
    unittest.main()
