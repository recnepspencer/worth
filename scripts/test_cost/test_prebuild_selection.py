import unittest

from compiler_prebuild import CompilerScope
from prebuild_selection import required


class SelectedPrerequisiteGraphTests(unittest.TestCase):
    def row(self, binary, test, **changes):
        return dict(binary=binary, test=test, ignored=False, **changes)

    def test_process_partition_does_not_repeat_the_compile_fail_graphs(self):
        scope, proof = required('store', [self.row('court', 'process::record::reopens')])
        self.assertIs(scope, CompilerScope.NONE)
        self.assertFalse(proof)

    def test_fixture_and_compiler_owner_claims_share_one_combined_prebuild(self):
        ui = self.row('court', 'compile_fail::authority::sealed')
        unit = self.row('worth-store-test-support', 'compiler_boundary::tests::checks_real_denial')
        self.assertIs(required('store', [ui])[0], CompilerScope.FIXTURES)
        self.assertIs(required('store', [unit])[0], CompilerScope.UNITS)
        self.assertIs(required('store', [ui, unit])[0], CompilerScope.ALL)

    def test_unselected_and_ignored_cases_cannot_request_cold_prerequisites(self):
        row = self.row('court', 'compile_fail::authority::sealed', selected=False)
        self.assertIs(required('store', [row])[0], CompilerScope.NONE)

    def test_proof_specimens_are_built_only_for_their_actual_process_contract(self):
        self.assertFalse(required('root', [self.row('worth-court-proof::proof', 'supported_programs::works')])[1])
        self.assertTrue(required('root', [self.row('worth-court-proof::proof',
                                                 'process::proof::band_guard_cargo_contract::legal_package')])[1])


if __name__ == '__main__':
    unittest.main()
