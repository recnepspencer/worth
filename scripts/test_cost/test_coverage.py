import unittest
from coverage import inventory, prove


class LaneCoverageTests(unittest.TestCase):
    def test_zero_and_multiple_selections_cannot_prove_exact_coverage(self):
        rows = [dict(binary='court', test=name, ignored=False) for name in ['one', 'two']]
        result = prove(rows, {'first': rows[:1], 'second': rows[:1]})
        self.assertEqual(result['status'], 'fail')
        self.assertEqual(result['zero'][0]['test'], 'two')
        self.assertEqual(result['multiple'][0]['test'], 'one')

    def test_ignored_tests_remain_visible_without_entering_landing_selection(self):
        rows = [dict(binary='court', test='ordinary', ignored=False),
                dict(binary='court', test='scale', ignored=True)]
        result = prove(rows, {'first': rows})
        self.assertEqual(result['status'], 'pass')
        self.assertEqual(result['scheduled_only'], 1)
        self.assertEqual(result['tests'][1]['selections'], 0)

    def test_nextest_filter_misses_do_not_count_as_selection(self):
        def suite(status):
            return {'rust-suites': {'court': {'testcases': {
                'a': {'ignored': False, 'filter-match': {'status': status}}}}}}
        self.assertEqual(prove(suite('matches'), {'lane': suite('mismatch')})['status'], 'fail')

    def test_unknown_or_duplicate_test_identities_are_refused(self):
        row = dict(binary='court', test='a', ignored=False)
        with self.assertRaises(ValueError):
            prove([], {'lane': [row]})
        with self.assertRaises(ValueError):
            inventory([row, row])


if __name__ == '__main__':
    unittest.main()
