from types import SimpleNamespace
import unittest

from diagnostic_flags import FLAGS, arguments


class CompilerDiagnosticFlagTests(unittest.TestCase):
    def test_repository_flags_are_augmented_through_cargo_array_merging(self):
        runner = SimpleNamespace(environment={})
        with arguments(runner) as extra:
            self.assertIn('build.rustflags=', extra[1])
            self.assertIn('--verbose', extra[1])
            self.assertNotIn('trybuild', extra[1])

    def test_encoded_caller_flags_and_the_owner_environment_are_preserved(self):
        original = {'CARGO_ENCODED_RUSTFLAGS': '--cfg\x1fname="value with spaces"'}
        runner = SimpleNamespace(environment=original)
        with arguments(runner) as extra:
            self.assertEqual(extra, [])
            self.assertEqual(runner.environment['CARGO_ENCODED_RUSTFLAGS'].split('\x1f'),
                             ['--cfg', 'name="value with spaces"', *FLAGS])
        self.assertIs(runner.environment, original)

    def test_failure_cannot_leak_diagnostic_flags_into_executable_prebuilding(self):
        original = {'RUSTFLAGS': '-C overflow-checks=on'}
        runner = SimpleNamespace(environment=original)
        with self.assertRaises(RuntimeError):
            with arguments(runner):
                raise RuntimeError('expected compiler refusal')
        self.assertIs(runner.environment, original)


if __name__ == '__main__':
    unittest.main()
