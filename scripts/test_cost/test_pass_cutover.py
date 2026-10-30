from pathlib import Path
import unittest

from pass_cutover import helper_calls, ordinary_groups


class SuccessfulProgramOwnershipTests(unittest.TestCase):
    def test_helper_call_loses_only_the_removed_compiler_session_parameter(self):
        text = 'fn test() { fixtures(&cases); other(&cases); }'
        self.assertEqual(helper_calls(text, {'fixtures'}),
                         'fn test() { fixtures(); other(&cases); }')

    def test_moved_test_requires_real_inventoried_execution_inputs(self):
        with self.assertRaises(ValueError):
            ordinary_groups({'fixtures': []}, {'court/tests/claims.rs': ['supported']}, Path('.'))

    def test_group_keeps_all_programs_attached_to_the_original_claim(self):
        inventory = {'fixtures': [{'path': 'court/tests/valid.rs', 'current_execution_callers': [
            {'source': 'court/tests/claims.rs', 'function': 'supported'}]}]}
        self.assertEqual(ordinary_groups(inventory, {'court/tests/claims.rs': ['supported']}, Path('.')),
                         {'court': [('supported', [Path('court/tests/valid.rs')])]})


if __name__ == '__main__':
    unittest.main()
