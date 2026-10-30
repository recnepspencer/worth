import os
import unittest
from unittest.mock import patch

from lane_environment import LaneEnvironmentRefusal, inherited, validate


class QualificationEnvironmentTests(unittest.TestCase):
    def test_external_compiler_profile_and_target_flag_overrides_are_refused(self):
        for name in ('RUSTFLAGS', 'CARGO_ENCODED_RUSTFLAGS', 'RUSTDOCFLAGS',
                     'CARGO_ENCODED_RUSTDOCFLAGS', 'RUSTUP_TOOLCHAIN', 'RUSTC',
                     'RUSTDOC', 'RUSTC_BOOTSTRAP', 'CARGO_INCREMENTAL',
                     'CARGO_BUILD_RUSTC', 'CARGO_BUILD_RUSTDOC', 'CARGO_BUILD_RUSTFLAGS',
                     'CARGO_BUILD_RUSTDOCFLAGS', 'CARGO_BUILD_INCREMENTAL',
                     'CARGO_BUILD_TARGET', 'CARGO_PROFILE_DEV_DEBUG',
                     'CARGO_PROFILE_DIAGNOSTIC_TEST_OPT_LEVEL',
                     'CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS',
                     'cargo_profile_test_debug'):
            for value in ('', 'uncommitted'):
                with self.subTest(name=name, value=value):
                    with self.assertRaisesRegex(LaneEnvironmentRefusal, name):
                        validate({name: value})

    def test_refusal_names_inputs_without_disclosing_their_values(self):
        with self.assertRaises(LaneEnvironmentRefusal) as failure:
            validate({'RUSTFLAGS': 'private-value', 'CARGO_PROFILE_TEST_DEBUG': 'private-profile'})
        self.assertIn('CARGO_PROFILE_TEST_DEBUG, RUSTFLAGS', str(failure.exception))
        self.assertNotIn('private-', str(failure.exception))

    def test_inheritance_keeps_cache_parallelism_and_build_identity_without_injecting_profiles(self):
        source = {'PATH': 'tools', 'RUSTC_WRAPPER': 'sccache', 'CARGO_BUILD_JOBS': '24',
                  'WORTH_IMPLEMENTATION_BUILD_ID': 'existing-build'}
        with patch.dict(os.environ, source, clear=True):
            observed = inherited()
        self.assertEqual(observed, source)
        observed['PATH'] = 'child-tools'
        self.assertEqual(source['PATH'], 'tools')

    def test_inherited_overrides_are_refused_before_returning_a_child_environment(self):
        with patch.dict(os.environ, {'CARGO_PROFILE_TEST_DEBUG': '0'}, clear=True):
            with self.assertRaises(LaneEnvironmentRefusal):
                inherited()


if __name__ == '__main__':
    unittest.main()
