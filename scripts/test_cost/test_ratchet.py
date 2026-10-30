import unittest

from ratchet import build_violations, lane_budget, source_violations


class StructuralBudgetTests(unittest.TestCase):
    def test_new_lane_or_profile_requires_a_committed_budget_declaration(self):
        configuration = {'store': {'declared_cargo_profiles': ['ci-test'],
                                    'lanes': {'process': {'max_compiles_per_unit': 1}}}}
        self.assertEqual(lane_budget(configuration, 'store', ['process'], 'ci-test'),
                         {'max_compiles_per_unit': 1, 'declared_cargo_profiles': ['ci-test']})
        for lanes, profile in [(['new-lane'], 'ci-test'), (['process'], 'release')]:
            with self.assertRaises(ValueError):
                lane_budget(configuration, 'store', lanes, profile)

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

    def test_check_mode_cannot_hide_a_second_compile_of_the_same_declared_unit(self):
        unit = dict(package='owner', target='owner', kind=['lib'], features=[],
                    profile={'test': False}, check_mode='build', target_dir='target', cargo_profile='dev')
        lane = dict(build_observation='json', artifacts=[unit, dict(unit, check_mode='check')])
        result = build_violations(lane, {'max_compiles_per_unit': 1, 'declared_cargo_profiles': ['dev']})
        self.assertEqual(result[0]['kind'], 'duplicate-graph')
        self.assertEqual(result[0]['check_modes'], ['build', 'check'])

    def test_nested_cargo_profiles_must_also_be_declared(self):
        unit = dict(package='owner', target='owner', kind=['lib'], features=[],
                    profile={'test': False}, check_mode='check', target_dir='target', cargo_profile='surprise')
        result = build_violations(dict(build_observation='json', artifacts=[unit]),
                                  {'max_compiles_per_unit': 1, 'declared_cargo_profiles': ['dev']})
        self.assertEqual(result[0]['kind'], 'undeclared-profile')

    def test_no_observation_cannot_pass_the_build_budget(self):
        self.assertEqual(build_violations(dict(build_observation='unmeasured', log='x'),
                                          {'max_compiles_per_unit': 1})[0]['kind'], 'unmeasured-build')

    def test_compiler_invocations_are_refused_but_sleep_telemetry_never_gates(self):
        rows = [dict(kind='cargo-command'), dict(kind='trybuild'), dict(kind='sleep')]
        self.assertEqual(source_violations(rows), rows[:2])


if __name__ == '__main__':
    unittest.main()
