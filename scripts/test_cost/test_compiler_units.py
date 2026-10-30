import copy
import unittest
from unittest.mock import patch

from compiler_units import CompilerUnitRefusal, attribute, declarations


def unit(name, *, mode='check', features=(), dependencies=(), kind='lib'):
    return {'pkg_id': name, 'target': {'name': name, 'kind': [kind], 'crate_types': [kind],
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

    def test_unplanned_feature_profile_or_output_drift_is_refused(self):
        row = unit('owner')
        declaration = {'units': declarations(graph(row)), 'compiler_inputs': 'flags'}
        changed = [event(row, linked=True), event(row), event(row)]
        changed[1]['features'] = ['unplanned']
        changed[2]['profile']['opt_level'] = '3'
        for observed in changed:
            with self.subTest(observed=observed), self.assertRaises(CompilerUnitRefusal):
                attribute(observed, declaration)

    def test_ambiguous_declared_units_are_refused_without_using_filename_hashes(self):
        row = unit('owner', dependencies=[1])
        rows = declarations(graph(row, unit('proof')))
        changed = copy.deepcopy(rows[0])
        changed['identity'] = 'another-dependency-graph'
        with self.assertRaises(CompilerUnitRefusal):
            attribute(event(row), {'units': rows + [changed], 'compiler_inputs': 'flags'})

    def test_unknown_modes_unreachable_units_and_cycles_are_refused(self):
        for value in (graph(unit('owner', mode='surprise')),
                      graph(unit('owner'), unit('orphan')),
                      graph(unit('owner', dependencies=[0]))):
            with self.subTest(value=value), self.assertRaises(CompilerUnitRefusal):
                declarations(value)


if __name__ == '__main__':
    unittest.main()
