import unittest
from unittest.mock import patch
from pathlib import Path
import tempfile

from store_plan import execute_plan, nextest_query


class StructuredStorePlanTests(unittest.TestCase):
    def test_workspace_documentation_joins_the_prebuilt_execution_exactly_once(self):
        with tempfile.TemporaryDirectory() as temporary, patch('store_plan.execute') as execute:
            root = Path(temporary)
            execute_plan(root, [('native', root, 'cargo', ['nextest', 'run', '--cargo-profile', 'ci-test']),
                                ('docs', root, 'cargo', ['test', '--workspace', '--doc', '--profile', 'ci-test'])], root)
            self.assertEqual(execute.call_count, 1)
            self.assertTrue(execute.call_args.kwargs['include_docs'])

    def test_exact_selector_and_target_kind_survive_the_prebuild_handoff(self):
        result = nextest_query(['nextest', 'run', '-p', 'court', '--tests',
                                '--filterset', 'test(=claim)'])
        self.assertEqual(result, '((test(=claim)) & (package(=court))) & (kind(test))')

    def test_scheduled_ignored_execution_is_refused(self):
        with self.assertRaises(ValueError):
            nextest_query(['nextest', 'run', '--include-ignored'])


if __name__ == '__main__':
    unittest.main()
