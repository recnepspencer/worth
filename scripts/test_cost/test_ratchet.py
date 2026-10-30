import unittest
import json
from pathlib import Path
import tempfile

from ratchet import build_violations, compiler_observations, lane_budget, source_violations
from compiler_units import declarations
from logs import read_lane
from test_compiler_units import event, graph, unit


def artifact(**changes):
    value = dict(package='owner', target='owner', kind=['lib'], features=[],
                 profile={'test': False}, check_mode='build', target_dir='target',
                 unit={'identity': 'required-owner-codegen', 'roles': ['target'], 'output': 'codegen',
                       'declared_profile': {'name': 'dev'}, 'compiler_inputs': 'declared-flags'})
    value.update(changes)
    return value


class StructuralBudgetTests(unittest.TestCase):
    def failed_lane(self, invocations, errors=1, level='error'):
        denied, cached = unit('denied'), unit('cached')
        required = graph(denied, cached)
        required['roots'] = [0, 1]
        marker = {'type': 'cargo-invocation', 'mode': 'check', 'cargo_profile': 'dev',
                  'unit_plan': {'units': declarations(required), 'compiler_inputs': 'flags'}}
        diagnostic = {'reason': 'compiler-message', 'package_id': denied['pkg_id'], 'target': denied['target'],
                      'message': {'level': level, 'message': 'diagnostic'}}
        cached = dict(event(cached), reason='compiler-artifact', fresh=True)
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary)/'lane.log'
            path.write_text('\n'.join(map(json.dumps, ([marker] + [diagnostic]*errors + [cached])*invocations)))
            return read_lane(path)

    def test_identical_failed_unit_in_two_cargo_invocations_exceeds_unchanged_budget(self):
        lane = self.failed_lane(2)
        self.assertEqual(lane['artifacts'], [])
        result = build_violations(lane, {'max_compiles_per_unit': 1})
        self.assertEqual(result[0]['kind'], 'duplicate-graph')
        self.assertEqual(result[0]['compiles'], 2)

    def test_multiple_error_messages_in_one_invocation_are_one_budgeted_attempt(self):
        lane = self.failed_lane(1, errors=3)
        self.assertEqual(len(lane['failed_compiler_attempts']), 1)
        self.assertEqual(build_violations(lane, {'max_compiles_per_unit': 1}), [])

    def test_replayed_cached_warnings_are_not_compilation_attempts(self):
        lane = self.failed_lane(2, errors=2, level='warning')
        self.assertEqual(lane['artifacts'], [])
        self.assertEqual(lane['failed_compiler_attempts'], [])
        self.assertEqual(build_violations(lane, {'max_compiles_per_unit': 1}), [])

    def test_unattributed_failed_unit_cannot_pass_as_zero_work(self):
        lane = {'build_observation': 'json', 'artifacts': [], 'compiler_attempt_refusals': [
            {'kind': 'unattributed-failed-compiler-unit', 'reason': 'ambiguous required role'}]}
        self.assertEqual(build_violations(lane, {'max_compiles_per_unit': 1})[0]['kind'],
                         'unattributed-failed-compiler-unit')

    def test_aggregating_command_captures_preserves_failed_attempts_and_refusals(self):
        failed = self.failed_lane(2)
        refused = {'build_observation': 'json', 'artifacts': [],
                   'compiler_attempt_refusals': [{'kind': 'unattributed-failed-compiler-unit'}]}
        combined = compiler_observations([failed, refused], 'lane')
        self.assertEqual(len(combined['failed_compiler_attempts']), 2)
        self.assertEqual(len(combined['compiler_attempt_refusals']), 1)
        self.assertEqual({row['kind'] for row in build_violations(combined, {'max_compiles_per_unit': 1})},
                         {'duplicate-graph', 'unattributed-failed-compiler-unit'})

    def test_new_lane_or_profile_requires_a_committed_budget_declaration(self):
        configuration = {'store': {'declared_cargo_profiles': ['ci-test'],
                                    'lanes': {'process': {'max_compiles_per_unit': 1}}}}
        self.assertEqual(lane_budget(configuration, 'store', ['process'], 'ci-test'),
                         {'max_compiles_per_unit': 1, 'declared_cargo_profiles': ['ci-test']})
        for lanes, profile in [(['new-lane'], 'ci-test'), (['process'], 'release')]:
            with self.assertRaises(ValueError):
                lane_budget(configuration, 'store', lanes, profile)

    def test_same_unit_recompiled_in_another_directory_exceeds_budget(self):
        unit = artifact()
        lane = dict(build_observation='json', artifacts=[unit, dict(unit, target_dir='extra')])
        violations = build_violations(lane, {'max_compiles_per_unit': 1})
        self.assertEqual(violations[0]['kind'], 'duplicate-graph')

    def test_distinct_executable_targets_are_not_duplicate_dependency_graphs(self):
        unit = artifact(target='a', kind=['bin'])
        other = dict(unit, target='b', unit={**unit['unit'], 'identity': 'required-bin-b'})
        lane = dict(build_observation='json', artifacts=[unit, other])
        self.assertEqual(build_violations(lane, {'max_compiles_per_unit': 1}), [])

    def test_required_host_link_and_target_check_units_are_not_false_duplicates(self):
        unit = artifact(cargo_profile='dev')
        checked = dict(unit, check_mode='check', unit={**unit['unit'], 'identity': 'required-owner-check',
                       'roles': ['target'], 'output': 'check'})
        lane = dict(build_observation='json', artifacts=[unit, checked])
        result = build_violations(lane, {'max_compiles_per_unit': 1, 'declared_cargo_profiles': ['dev']})
        self.assertEqual(result, [])

    def test_identical_unit_recompilation_still_fails_when_filenames_change(self):
        unit = artifact(filenames=['target/owner-one.rlib'])
        repeated = dict(unit, filenames=['target/owner-another.rlib'])
        result = build_violations(dict(build_observation='json', artifacts=[unit, repeated]),
                                  {'max_compiles_per_unit': 1})
        self.assertEqual(result[0]['kind'], 'duplicate-graph')

    def test_undeclared_unit_or_changed_compiler_flags_cannot_mint_a_budget_identity(self):
        unit = artifact()
        changed = dict(unit, unit={**unit['unit'], 'compiler_inputs': 'unplanned-flags'})
        result = build_violations(dict(build_observation='json', artifacts=[unit, changed]),
                                  {'max_compiles_per_unit': 1})
        self.assertEqual({row['kind'] for row in result}, {'unplanned-compiler-input-drift', 'duplicate-graph'})
        result = build_violations(dict(build_observation='json', artifacts=[dict(unit, unit=None)]),
                                  {'max_compiles_per_unit': 1})
        self.assertEqual(result[0]['kind'], 'undeclared-compiler-unit')

    def test_nested_cargo_profiles_must_also_be_declared(self):
        unit = artifact(cargo_profile='surprise')
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
