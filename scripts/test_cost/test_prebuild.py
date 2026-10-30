import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from prebuild import prepare


class PrerequisiteOwnershipTests(unittest.TestCase):
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


if __name__ == '__main__':
    unittest.main()
