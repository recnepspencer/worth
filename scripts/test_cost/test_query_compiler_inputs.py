"""Consumer receipts require exact native compiler attribution for both twins."""
from pathlib import Path
import unittest
import tempfile

from compile_batch import BatchCompilationFailure
from query_compiler_inputs import attributed, resolve_locations


class ConsumerCompilerReceipts(unittest.TestCase):
    def test_relative_span_and_expansion_paths_keep_the_actual_consumer_source(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / 'hostile/src/main.rs'
            source.parent.mkdir(parents=True)
            source.write_text('fn main() {}')
            observed = {'spans': [{'file_name': 'hostile/src/main.rs', 'label': 'schema requirements',
                'expansion': {'span': {'file_name': 'hostile/src/main.rs'}}}],
                'children': [{'spans': [{'file_name': '<generated>'}]}]}
            resolve_locations(observed, root)
            self.assertEqual(observed['spans'][0]['file_name'], str(source.resolve()))
            self.assertEqual(observed['spans'][0]['expansion']['span']['file_name'], str(source.resolve()))
            self.assertEqual(observed['spans'][0]['label'], 'schema requirements')
            self.assertEqual(observed['children'][0]['spans'][0]['file_name'], '<generated>')

    def test_compiler_receipts_require_observation_and_preserve_denial(self):
        source = str(Path('consumer.rs').resolve())
        target = {'src_path': source, 'name': 'consumer', 'kind': ['bin']}
        targets = {source: ('consumer-package', target)}
        artifact = {'reason': 'compiler-artifact', 'package_id': 'consumer-package', 'target': target}
        denial = dict(artifact, reason='compiler-message', message={'level': 'error', 'message': 'unmet schema requirement'})
        self.assertEqual(attributed([artifact], targets)[source]['fate'], 'compiled')
        for events in ([artifact, denial], [denial, artifact]):
            self.assertEqual(attributed(events, targets)[source]['fate'], 'denied')
        for events in ([], [dict(artifact, package_id='foreign')],
                       [dict(denial, package_id='foreign')]):
            with self.assertRaises(BatchCompilationFailure):
                attributed(events, targets)


if __name__ == '__main__':
    unittest.main()
