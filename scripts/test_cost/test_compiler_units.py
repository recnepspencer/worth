import copy
import unittest
from unittest.mock import patch

from compiler_units import CompilerUnitRefusal, attribute, declarations, failed_unit


def unit(name, *, mode='check', features=(), dependencies=(), kind='lib'):
    return {'pkg_id': name, 'target': {'name': name, 'kind': [kind],
            'crate_types': ['bin' if kind == 'test' else kind],
            'src_path': f'{name}.rs', 'edition': '2024'}, 'mode': mode, 'platform': None,
            'features': list(features), 'profile': {'name': 'dev', 'opt_level': '0', 'debuginfo': 0,
            'debug_assertions': True, 'overflow_checks': True},
            'dependencies': [{'index': index, 'extern_crate_name': str(index)} for index in dependencies]}


def graph(*units):
    return {'version': 1, 'roots': [0], 'units': list(units)}


def event(row, *, linked=False):
    return {'package_id': row['pkg_id'], 'target': row['target'], 'features': row['features'],
            'profile': {**{key: value for key, value in row['profile'].items() if key != 'name'},
                        'test': row['mode'] == 'test'},
            'filenames': ['target/debug/deps/owner.rlib' if linked else 'target/debug/deps/owner.rmeta']}


class CompilerUnitDeclarationTests(unittest.TestCase):
    def test_failed_target_without_artifact_uses_only_its_exact_predeclared_unit(self):
        row = unit('owner')
        denied = {'package_id': row['pkg_id'], 'target': row['target'],
                  'message': {'level': 'error', 'message': 'denied'}}
        declaration = {'units': declarations(graph(row)), 'compiler_inputs': 'flags'}
        self.assertEqual(failed_unit(denied, declaration)['identity'], declaration['units'][0]['identity'])
        denied['target'] = {**denied['target'], 'src_path': 'foreign.rs'}
        with self.assertRaises(CompilerUnitRefusal):
            failed_unit(denied, declaration)

    def test_failed_target_with_ambiguous_host_and_check_units_refuses_attribution(self):
        owner = unit('owner', dependencies=[1, 2])
        linked = unit('dependency', mode='build')
        checked = unit('dependency')
        declaration = {'units': declarations(graph(owner, linked, checked)), 'compiler_inputs': 'flags'}
        with self.assertRaises(CompilerUnitRefusal):
            failed_unit({'package_id': linked['pkg_id'], 'target': linked['target'],
                         'message': {'level': 'error'}}, declaration)

    def test_cached_warning_cannot_prove_a_failed_compiler_attempt(self):
        row = unit('owner')
        with self.assertRaises(CompilerUnitRefusal):
            failed_unit({'package_id': row['pkg_id'], 'target': row['target'],
                         'message': {'level': 'warning'}},
                        {'units': declarations(graph(row)), 'compiler_inputs': 'flags'})

    def test_required_transitive_feature_graph_distinguishes_equal_own_features(self):
        first = declarations(graph(unit('owner', dependencies=[1]), unit('proof')))
        second = declarations(graph(unit('owner', dependencies=[1]), unit('proof', features=['cert'])))
        self.assertNotEqual(first[0]['identity'], second[0]['identity'])
        self.assertEqual(first[0]['features'], second[0]['features'])

    def test_host_link_and_target_check_are_declared_distinct_required_units(self):
        rows = declarations(graph(unit('owner', dependencies=[1, 2]),
                                  unit('build-script', mode='build', kind='custom-build', dependencies=[3]),
                                  unit('dependency'), unit('dependency', mode='build')))
        self.assertEqual(rows[2]['required_by'], ['target'])
        self.assertEqual(rows[3]['required_by'], ['host'])
        self.assertNotEqual(rows[2]['identity'], rows[3]['identity'])

    def test_native_unit_used_by_host_and_product_does_not_gain_another_identity(self):
        dependency = unit('dependency', mode='build')
        alone = declarations(graph(dependency))[0]
        both = declarations(graph(unit('owner', mode='build', dependencies=[1, 2]),
            unit('build-script', mode='build', kind='custom-build', dependencies=[2]), dependency))[2]
        self.assertEqual(alone['identity'], both['identity'])
        self.assertEqual(both['required_by'], ['host', 'target'])

    def test_case_sensitive_source_paths_remain_distinct(self):
        first, second = unit('owner'), unit('owner')
        second['target']['src_path'] = 'OWNER.rs'
        # Exercise the case-sensitive filesystem contract on Windows as well.
        with patch('compiler_units.os.path.normcase', side_effect=lambda value: value):
            self.assertNotEqual(declarations(graph(first))[0]['identity'], declarations(graph(second))[0]['identity'])

    def test_reordering_indices_does_not_invent_another_required_unit(self):
        one, two = unit('owner', dependencies=[1, 2]), unit('owner', dependencies=[2, 1])
        for owner in (one, two):
            for edge, name in zip(owner['dependencies'], ('a', 'b')):
                edge['extern_crate_name'] = name
        first = declarations(graph(one, unit('a'), unit('b')))
        second = declarations(graph(two, unit('b'), unit('a')))
        self.assertEqual(first[0]['identity'], second[0]['identity'])

    def test_artifact_hashes_do_not_declare_compiler_identity(self):
        row = unit('owner')
        declaration = {'units': declarations(graph(row)), 'compiler_inputs': 'flags'}
        first = attribute(event(row), declaration)
        renamed = event(row)
        renamed['filenames'] = ['other-directory/owner-unrelated-hash.rmeta']
        self.assertEqual(first, attribute(renamed, declaration))

    def test_extensionless_host_build_script_is_still_the_declared_codegen_unit(self):
        row = unit('build-script', mode='build', kind='custom-build')
        observed = event(row, linked=True)
        observed.update(executable=None, filenames=['target/debug/build/owner/build-script-build'])
        result = attribute(observed, {'units': declarations(graph(row)), 'compiler_inputs': 'flags'})
        self.assertEqual(result['output'], 'codegen')

    def test_unplanned_feature_profile_or_output_drift_is_refused(self):
        row = unit('owner')
        declaration = {'units': declarations(graph(row)), 'compiler_inputs': 'flags'}
        changed = [event(row, linked=True), event(row), event(row)]
        changed[1]['features'] = ['unplanned']
        changed[2]['profile']['opt_level'] = '3'
        for observed in changed:
            with self.subTest(observed=observed), self.assertRaises(CompilerUnitRefusal):
                attribute(observed, declaration)

    def test_checked_integration_target_uses_its_declared_test_profile_and_metadata_output(self):
        row = unit('court', kind='test')
        observed = event(row)
        observed['profile']['test'] = True
        declaration = {'units': declarations(graph(row)), 'compiler_inputs': 'flags'}
        before = copy.deepcopy(declaration)
        result = attribute(observed, declaration)
        self.assertEqual(result['identity'], declaration['units'][0]['identity'])
        self.assertEqual((result['mode'], result['output']), ('check', 'check'))
        self.assertEqual(declaration, before)

    def test_checked_integration_target_refuses_a_missing_test_flag(self):
        row = unit('court', kind='test')
        with self.assertRaises(CompilerUnitRefusal):
            attribute(event(row), {'units': declarations(graph(row)), 'compiler_inputs': 'flags'})

    def test_checked_integration_target_refuses_drift_in_every_compiler_profile_field(self):
        row = unit('court', kind='test')
        declaration = {'units': declarations(graph(row)), 'compiler_inputs': 'flags'}
        changes = {'opt_level': '3', 'debuginfo': 2, 'debug_assertions': False, 'overflow_checks': False}
        for field, value in changes.items():
            observed = event(row)
            observed['profile'].update(test=True, **{field: value})
            with self.subTest(field=field), self.assertRaises(CompilerUnitRefusal):
                attribute(observed, declaration)

    def test_checked_library_test_build_remains_refused_without_a_declared_test_stage(self):
        row = unit('owner')
        row['target']['test'] = True  # Capability does not declare this compilation's test flag.
        declaration = {'units': declarations(graph(row)), 'compiler_inputs': 'flags'}
        observed = event(row)
        self.assertEqual(attribute(observed, declaration)['mode'], 'check')
        observed['profile']['test'] = True
        with self.assertRaises(CompilerUnitRefusal):
            attribute(observed, declaration)

    def test_test_mode_library_still_requires_its_test_flag(self):
        row = unit('owner', mode='test')
        declaration = {'units': declarations(graph(row)), 'compiler_inputs': 'flags'}
        observed = event(row, linked=True)
        self.assertEqual(attribute(observed, declaration)['mode'], 'test')
        observed['profile']['test'] = False
        with self.assertRaises(CompilerUnitRefusal):
            attribute(observed, declaration)

    def test_checked_integration_target_refuses_ambiguous_required_dependency_graphs(self):
        rows = declarations(graph(unit('root', dependencies=[1, 2]),
            unit('court', kind='test', dependencies=[3]), unit('court', kind='test', dependencies=[4]),
            unit('proof'), unit('proof', features=['cert'])))
        observed = event(unit('court', kind='test'))
        observed['profile']['test'] = True
        self.assertNotEqual(rows[1]['identity'], rows[2]['identity'])
        with self.assertRaises(CompilerUnitRefusal):
            attribute(observed, {'units': rows, 'compiler_inputs': 'flags'})

    def test_ambiguous_declared_units_are_refused_without_using_filename_hashes(self):
        row = unit('owner', dependencies=[1])
        rows = declarations(graph(row, unit('proof')))
        changed = copy.deepcopy(rows[0])
        changed['identity'] = 'another-dependency-graph'
        with self.assertRaises(CompilerUnitRefusal):
            attribute(event(row), {'units': rows + [changed], 'compiler_inputs': 'flags'})

    def test_doctest_driver_cannot_claim_its_library_build_artifact(self):
        driver = unit('owner', mode='doctest', dependencies=[1])
        library = unit('owner', mode='build')
        rows = declarations(graph(driver, library))
        before = copy.deepcopy(rows)
        self.assertEqual([row['mode'] for row in rows], ['doctest', 'build'])
        self.assertNotEqual(rows[0]['identity'], rows[1]['identity'])
        self.assertEqual(rows[0]['dependency_units'], [rows[1]['identity']])
        observed = attribute(event(library, linked=True), {'units': rows, 'compiler_inputs': 'flags'})
        self.assertEqual(observed['identity'], rows[1]['identity'])
        self.assertEqual(observed['mode'], 'build')
        self.assertEqual(rows, before)

    def test_doctest_only_declaration_does_not_authorize_a_library_artifact(self):
        driver = unit('owner', mode='doctest')
        with self.assertRaisesRegex(CompilerUnitRefusal, 'exactly one predeclared compiler unit'):
            attribute(event(driver, linked=True), {'units': declarations(graph(driver)), 'compiler_inputs': 'flags'})

    def test_failed_diagnostic_keeps_build_and_doctest_stage_ambiguity(self):
        driver = unit('owner', mode='doctest', dependencies=[1])
        library = unit('owner', mode='build')
        declaration = {'units': declarations(graph(driver, library)), 'compiler_inputs': 'flags'}
        with self.assertRaisesRegex(CompilerUnitRefusal, 'failed target does not identify exactly one'):
            failed_unit({'package_id': library['pkg_id'], 'target': library['target'],
                         'message': {'level': 'error'}}, declaration)

    def test_unknown_modes_unreachable_units_and_cycles_are_refused(self):
        for value in (graph(unit('owner', mode='surprise')),
                      graph(unit('owner'), unit('orphan')),
                      graph(unit('owner', dependencies=[0]))):
            with self.subTest(value=value), self.assertRaises(CompilerUnitRefusal):
                declarations(value)


if __name__ == '__main__':
    unittest.main()
