import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import Mock, patch

from prebuild import main, prepare
from lane_environment import LaneEnvironmentRefusal
from native_features import NativeFeatureRefusal


class PrerequisiteOwnershipTests(unittest.TestCase):
    def test_standalone_prebuild_refuses_external_flags_before_preparing_any_graph(self):
        with patch('sys.argv', ['prebuild.py', '--workspace', 'query']), \
             patch.dict(os.environ, {'RUSTFLAGS': '-C opt-level=3'}), \
             patch('prebuild.CargoRunner') as runner, patch('prebuild.prepare') as build:
            with self.assertRaises(LaneEnvironmentRefusal):
                main()
        runner.assert_not_called()
        build.assert_not_called()

    def test_workspace_prebuild_exports_only_its_owned_environment(self):
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary)
            with patch('bank_prebuild.prepare', return_value={'WORTH_PREBUILT_BANK_METADATA': 'raw.json'}) as bank:
                env = prepare(Path.cwd(), 'bank', output, object())
            self.assertEqual(bank.call_count, 1)
            self.assertEqual(env, {'WORTH_PREBUILT_BANK_METADATA': 'raw.json'})
            self.assertEqual(json.loads((output / 'environment.json').read_text()), env)

    def test_query_without_nested_compilation_has_no_fabricated_prerequisite(self):
        with tempfile.TemporaryDirectory() as temporary:
            self.assertEqual(prepare(Path.cwd(), 'query', Path(temporary), object()), {})

    def test_missing_native_graph_refuses_store_preparation_before_compiler_work(self):
        with tempfile.TemporaryDirectory() as temporary:
            runner = Mock()
            with self.assertRaisesRegex(NativeFeatureRefusal, 'native build directory'):
                prepare(Path.cwd(), 'store', Path(temporary), runner)
            runner.assert_not_called()


if __name__ == '__main__':
    unittest.main()
