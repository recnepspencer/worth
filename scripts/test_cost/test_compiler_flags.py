import json
from types import SimpleNamespace
import unittest

from compiler_flags import arguments


class CompilerFlagLayeringTests(unittest.TestCase):
    def test_repository_flags_are_augmented_using_a_cargo_array(self):
        original = {}
        runner = SimpleNamespace(environment=original)
        with arguments(runner, ['--emit=llvm-ir', '-Ccodegen-units=1']) as extra:
            self.assertEqual(extra[0], '--config')
            self.assertEqual(json.loads(extra[1].split('=', 1)[1]),
                             ['--emit=llvm-ir', '-Ccodegen-units=1'])
            self.assertIs(runner.environment, original)

    def test_encoded_tokens_with_spaces_and_the_original_environment_survive(self):
        original = {'CARGO_ENCODED_RUSTFLAGS': '--cfg\x1fname="value with spaces"', 'CARGO_INCREMENTAL': '1'}
        runner = SimpleNamespace(environment=original)
        with arguments(runner, ['--emit=llvm-ir'], environment={'CARGO_INCREMENTAL': '0'}) as extra:
            self.assertEqual(extra, [])
            self.assertEqual(runner.environment['CARGO_ENCODED_RUSTFLAGS'].split('\x1f'),
                             ['--cfg', 'name="value with spaces"', '--emit=llvm-ir'])
            self.assertEqual(runner.environment['CARGO_INCREMENTAL'], '0')
        self.assertIs(runner.environment, original)

    def test_measurement_failure_does_not_change_later_executable_build_flags(self):
        original = {'RUSTFLAGS': '-C overflow-checks=on'}
        runner = SimpleNamespace(environment=original)
        with self.assertRaises(RuntimeError):
            with arguments(runner, ['--emit=llvm-ir'], environment={'CARGO_INCREMENTAL': '0'}):
                self.assertIn('--emit=llvm-ir', runner.environment['CARGO_ENCODED_RUSTFLAGS'])
                raise RuntimeError('measurement failed')
        self.assertIs(runner.environment, original)
        self.assertNotIn('CARGO_ENCODED_RUSTFLAGS', original)

    def test_incremental_override_without_caller_flags_keeps_repository_array_merging(self):
        original = {}
        runner = SimpleNamespace(environment=original)
        with arguments(runner, ['--emit=llvm-ir'], environment={'CARGO_INCREMENTAL': '0'}) as extra:
            self.assertEqual(extra[0], '--config')
            self.assertEqual(runner.environment, {'CARGO_INCREMENTAL': '0'})
        self.assertIs(runner.environment, original)

    def test_empty_encoded_flags_do_not_create_an_empty_rustc_argument(self):
        runner = SimpleNamespace(environment={'CARGO_ENCODED_RUSTFLAGS': ''})
        with arguments(runner, ['--emit=llvm-ir']):
            self.assertEqual(runner.environment['CARGO_ENCODED_RUSTFLAGS'], '--emit=llvm-ir')


if __name__ == '__main__':
    unittest.main()
