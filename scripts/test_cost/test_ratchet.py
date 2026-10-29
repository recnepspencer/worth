import unittest

from ratchet import build_violations, source_violations


class StructuralBudgetTests(unittest.TestCase):
    def test_same_unit_recompiled_in_another_directory_exceeds_budget(self):
        unit = dict(package='owner', target='owner', kind=['lib'], features=[],
                    profile={'test': False}, check_mode='build', target_dir='target')
        lane = dict(build_observation='json', artifacts=[unit, dict(unit, target_dir='extra')])
        violations = build_violations(lane, {'max_compiles_per_unit': 1})
        self.assertEqual(violations[0]['kind'], 'duplicate-graph')

    def test_distinct_executable_targets_are_not_duplicate_dependency_graphs(self):
        unit = dict(package='owner', target='a', kind=['bin'], features=[],
                    profile={'test': False}, check_mode='build', target_dir='target')
        lane = dict(build_observation='json', artifacts=[unit, dict(unit, target='b')])
        self.assertEqual(build_violations(lane, {'max_compiles_per_unit': 1}), [])

    def test_no_observation_cannot_pass_the_build_budget(self):
        self.assertEqual(build_violations(dict(build_observation='unmeasured', log='x'),
                                          {'max_compiles_per_unit': 1})[0]['kind'], 'unmeasured-build')

    def test_compiler_invocations_are_refused_but_sleep_telemetry_never_gates(self):
        rows = [dict(kind='cargo-command'), dict(kind='trybuild'), dict(kind='sleep')]
        self.assertEqual(source_violations(rows), rows[:2])


if __name__ == '__main__':
    unittest.main()
