import unittest
from sweep_cutover import SharedCaseState, rewrite


class IndependentSweepCutoverTests(unittest.TestCase):
    def test_independent_case_loop_keeps_every_body_statement_and_duplicate_input(self):
        text = 'fn proof() { for case in [1,1,3] { run_case(case); } after(); }'
        result, rows = rewrite(text, 'run_case')
        self.assertEqual(len(rows), 1)
        self.assertIn('IntoIterator::into_iter([1,1,3]).collect()', result)
        self.assertIn('|case| { run_case(case); }', result)
        self.assertIn('after();', result)

    def test_nested_case_membership_is_preserved_inside_the_worker(self):
        text = 'fn proof() { for role in ROLES { for cut in cuts(role) { run_case(role,cut); } } }'
        result, rows = rewrite(text, 'run_case')
        self.assertEqual(len(rows), 1)
        self.assertIn('for cut in cuts(role) { run_case(role,cut); }', result)

    def test_shared_mutable_witnesses_are_refused(self):
        for body in ['answers += run_case(case);', 'run_case(case, &mut answers);']:
            with self.assertRaises(SharedCaseState):
                rewrite('fn proof() { for case in cases { ' + body + ' } }', 'run_case')

    def test_rerunning_the_cutover_does_not_pool_a_worker_again(self):
        original = 'fn proof() { for role in ROLES { for cut in cuts(role) { run_case(role,cut); } } }'
        changed, _ = rewrite(original, 'run_case')
        repeated, rows = rewrite(changed, 'run_case')
        self.assertEqual(repeated, changed)
        self.assertEqual(rows, [])


if __name__ == '__main__':
    unittest.main()
