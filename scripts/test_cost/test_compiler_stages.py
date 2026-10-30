from pathlib import Path
import tempfile
import unittest

from compiler_audiences import group, sources
from compiler_stages import CompilerStage, partition, stage


class CompilerStageTests(unittest.TestCase):
    def test_codegen_splits_only_its_fixture_preserving_production_audience(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            packages = group([{'dependencies': {'owner': {'path': root.as_posix()}},
                               'sources': [root / 'denied.rs', root / 'metadata.rs'],
                               'audience': 'original-producer'}])
            original = {'production': dict(identity='production', profile='dev',
                                           reject_test_features=True, packages=packages)}
            declarations = {'schema': 1, 'workspaces': {'root': [
                {'source': 'denied.rs', 'stage': 'codegen'}]}}
            result = partition(root, 'root', original, declarations)
            self.assertEqual(list(result), ['production', 'production-codegen'])
            self.assertEqual(sources(result['production']), [root / 'metadata.rs'])
            generated = result['production-codegen']
            self.assertEqual(sources(generated), [root / 'denied.rs'])
            self.assertTrue(generated['reject_test_features'])
            self.assertEqual(generated['environment_class'], 'production')
            self.assertEqual(generated['packages'][0]['identity'], packages[0]['identity'])
            self.assertEqual(generated['packages'][0]['audiences'], ['original-producer'])
            self.assertEqual(stage(generated), CompilerStage.CODEGEN)
            self.assertEqual(stage(generated).cargo_command, 'build')
            self.assertEqual(len(original['production']['packages'][0]['sources']), 2)

    def test_unknown_duplicate_and_absent_stage_declarations_refuse(self):
        for entries in ([{'source': 'x.rs', 'stage': 'execute'}],
                        [{'source': 'x.rs', 'stage': 'codegen'}] * 2,
                        [{'source': 'absent.rs', 'stage': 'codegen'}]):
            with self.subTest(entries=entries), self.assertRaises(ValueError):
                partition(Path.cwd(), 'root', {}, {'schema': 1, 'workspaces': {'root': entries}})

    def test_undeclared_fixture_is_a_check_not_codegen_inferred_from_diagnostics(self):
        result = partition(Path.cwd(), 'bank', {'production': dict(
            identity='production', profile='dev', packages=group([
                {'dependencies': {}, 'sources': ['specimen.rs']}]))}, {'schema': 1})
        self.assertEqual(stage(result['production']), CompilerStage.CHECK)
        self.assertEqual(stage(result['production']).cargo_command, 'check')


if __name__ == '__main__':
    unittest.main()
