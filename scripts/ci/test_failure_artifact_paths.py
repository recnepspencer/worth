"""Configured failure uploads retain existing nested declarations and censuses."""
import glob
from pathlib import Path
import re
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[2]


def upload_patterns(name):
    source = (ROOT/'.github/workflows/ci.yml').read_text(encoding='utf-8')
    match = re.search(r'          name: '+re.escape(name)+r'\n          path: \|\n'
                      r'((?:            [^\n]+\n?)+)', source)
    if match is None:
        raise AssertionError('missing configured failure upload: '+name)
    return [line.strip() for line in match[1].splitlines()]


class FailureArtifactPathsTests(unittest.TestCase):
    def selected(self, name):
        files = {'target/query-gate/summary.json', 'target/query-gate/root-doc.json',
                 'target/query-gate/top-level.log',
                 'target/query-gate/test-cost/root/census.json',
                 'target/query-gate/test-cost/query/build-ratchet.json',
                 'target/query-gate/test-cost/query/documentation/doctests-units.json',
                 'target/query-gate/test-cost/query/documentation/doctests-unit-plan.log',
                 'target/query-gate/test-cost/bank/documentation/doctests.log',
                 'target/query-gate/fixture/nested/observations.jsonl',
                 'target/query-gate/fixture/nested/source.rs',
                 'target/unrelated/census.json'}
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            for relative in files:
                path = root/relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text('fixture', encoding='utf-8')
            selected = {Path(path).relative_to(root).as_posix()
                        for pattern in upload_patterns(name)
                        for path in glob.glob(str(root/pattern), recursive=True)}
        return selected

    def test_docs_failure_upload_keeps_original_summaries_and_nested_compiler_evidence(self):
        selected = self.selected('consumer-docs-gate-failure')
        self.assertEqual(selected, {'target/query-gate/summary.json', 'target/query-gate/root-doc.json',
                         'target/query-gate/top-level.log', 'target/query-gate/test-cost/root/census.json',
                         'target/query-gate/test-cost/query/build-ratchet.json',
                         'target/query-gate/test-cost/query/documentation/doctests-units.json',
                         'target/query-gate/test-cost/query/documentation/doctests-unit-plan.log',
                         'target/query-gate/test-cost/bank/documentation/doctests.log'})

    def test_checkpoint_failure_upload_keeps_nested_logs_json_and_jsonl(self):
        selected = self.selected('query-checkpoint-recovery-failure')
        self.assertIn('target/query-gate/fixture/nested/observations.jsonl', selected)
        self.assertIn('target/query-gate/test-cost/query/documentation/doctests-units.json', selected)
        self.assertIn('target/query-gate/test-cost/query/documentation/doctests-unit-plan.log', selected)
        self.assertIn('target/query-gate/summary.json', selected)
        self.assertNotIn('target/query-gate/fixture/nested/source.rs', selected)
        self.assertNotIn('target/unrelated/census.json', selected)


if __name__ == '__main__':
    unittest.main()
