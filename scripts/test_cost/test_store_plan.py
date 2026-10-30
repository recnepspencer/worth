import unittest

from store_plan import nextest_query


class StructuredStorePlanTests(unittest.TestCase):
    def test_exact_selector_and_target_kind_survive_the_prebuild_handoff(self):
        result = nextest_query(['nextest', 'run', '-p', 'court', '--tests',
                                '--filterset', 'test(=claim)'])
        self.assertEqual(result, '((test(=claim)) & (package(=court))) & (kind(test))')

    def test_scheduled_ignored_execution_is_refused(self):
        with self.assertRaises(ValueError):
            nextest_query(['nextest', 'run', '--include-ignored'])


if __name__ == '__main__':
    unittest.main()
