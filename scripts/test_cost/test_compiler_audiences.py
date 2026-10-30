from pathlib import Path
import unittest

from compiler_audiences import group, sources, surface_identity


class CompilerAudienceTests(unittest.TestCase):
    def test_identical_declared_surfaces_share_a_package_but_keep_every_fixture(self):
        first = {'dependencies': {'owner': {'version': '1', 'features': ['b', 'a']}},
                 'sources': ['one.rs'], 'audience': 'first'}
        second = {'dependencies': {'owner': {'features': ['a', 'b'], 'version': '1'}},
                  'sources': ['two.rs'], 'audience': 'second'}
        packages = group([first, second])
        self.assertEqual(len(packages), 1)
        self.assertEqual(packages[0]['audiences'], ['first', 'second'])
        self.assertEqual(set(sources({'packages': packages})),
                         {Path('one.rs').resolve(), Path('two.rs').resolve()})

    def test_backend_presence_and_absence_are_different_packages_in_the_same_environment(self):
        packages = group([
            {'dependencies': {'owner': {'version': '1'}}, 'sources': ['forbidden.rs']},
            {'dependencies': {'owner': {'version': '1'}, 'backend': {'version': '1'}},
             'sources': ['allowed.rs']}])
        self.assertEqual(len(packages), 2)
        self.assertEqual(sum('backend' in package['dependencies'] for package in packages), 1)

    def test_surface_provenance_includes_unused_dependencies_and_feature_choices(self):
        base = {'owner': {'version': '1'}}
        self.assertNotEqual(surface_identity(base), surface_identity(dict(base, unused={'version': '1'})))
        self.assertNotEqual(surface_identity(base),
                            surface_identity({'owner': {'version': '1', 'features': ['authority']}}))

    def test_target_specific_dependencies_do_not_become_unconditional_or_lose_their_identity(self):
        conditional = {'cfg(target_family = "wasm")': {'backend': {'version': '1'}}}
        packages = group([{'dependencies': {}, 'target_dependencies': conditional, 'sources': ['one.rs']},
                          {'dependencies': {}, 'sources': ['two.rs']}])
        self.assertEqual(len(packages), 2)
        self.assertTrue(all('backend' not in package['dependencies'] for package in packages))
        self.assertEqual(sum(package['target_dependencies'] == conditional for package in packages), 1)

    def test_duplicate_source_is_refused_even_when_declared_in_different_audiences(self):
        with self.assertRaisesRegex(ValueError, 'duplicate fixture source'):
            group([{'dependencies': {}, 'sources': ['same.rs']},
                   {'dependencies': {'backend': {'version': '1'}}, 'sources': ['same.rs']}])

    def test_package_surface_identity_cannot_be_forged_or_repeated(self):
        packages = group([{'dependencies': {}, 'sources': ['one.rs']}])
        with self.assertRaisesRegex(ValueError, 'duplicate compiler audience'):
            sources({'packages': packages + packages})
        packages[0]['dependencies']['injected'] = {'version': '1'}
        with self.assertRaisesRegex(ValueError, 'identity does not match'):
            sources({'packages': packages})


if __name__ == '__main__':
    unittest.main()
