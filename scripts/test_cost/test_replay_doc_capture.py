import json
from pathlib import Path
import tempfile
import unittest

from doc_contracts import DocContractFailure, DocFailureKind
from replay_doc_capture import replay


class ReplayDocCaptureTests(unittest.TestCase):
    def capture(self, root, exit_code=0):
        command = ['cargo', 'test', '--workspace', '--doc', '--profile', 'dev', '--no-fail-fast',
                   '--locked', '--manifest-path', str(root/'Cargo.toml'), '--message-format=json', '--', '--format', 'pretty']
        text = (f'Command: {json.dumps(command)}\nWorking directory: {root}\nHEAD: original-head\n'
                'Git status --short empty: False\nStart: 2026-01-01T00:00:00+00:00\n\n')
        text += json.dumps({'type': 'cargo-invocation', 'capture': 'ordered', 'command': command}) + '\n'
        text += 'Doc-tests owner\nrunning 1 test\ntest src/lib.rs (line 1) ... ok\n'
        text += 'test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out\n\n'
        text += f'End: 2026-01-01T00:00:01+00:00\nExit code: {exit_code}\n'
        capture, canonical = root/'original.log', root/'retained.log'
        for path in (capture, canonical):
            path.write_text(text, encoding='utf-8')
        metadata = {'workspace_root': str(root), 'workspace_members': ['owner'], 'packages': [
            {'id': 'owner', 'name': 'owner', 'targets': [{'kind': ['lib'], 'crate_types': ['lib'],
                                                      'name': 'owner', 'doctest': False}]}]}
        source = root/'metadata.json'
        source.write_text(json.dumps(metadata), encoding='utf-8')
        return capture, canonical, source

    def test_replay_uses_original_revision_and_retry_appends_no_duplicate_events(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            capture, canonical, metadata = self.capture(root)
            result = replay(capture, root/'output', metadata, canonical)
            first = capture.read_text()
            second = replay(capture, root/'output', metadata, canonical)
            self.assertEqual(result, second)
            self.assertEqual(first, capture.read_text())
            self.assertEqual(capture.read_text(), canonical.read_text())
            self.assertEqual(result['counts'], {'ok': 1, 'failed': 0, 'ignored': 0})
            self.assertEqual(result['provenance']['headers']['HEAD'], 'original-head')
            self.assertEqual(json.loads((root/'output/result.json').read_text()), result)

    def test_nonzero_original_exit_is_preserved_without_inventing_failed_snippets(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            capture, canonical, metadata = self.capture(root, 101)
            result = replay(capture, root/'output', metadata, canonical)
            self.assertEqual(result['status'], 'fail')
            self.assertEqual(result['counts']['failed'], 0)

    def test_incomplete_capture_or_disagreeing_canonical_log_is_refused_without_appending(self):
        for mutation in ('truncated', 'different'):
            with self.subTest(mutation=mutation), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary).resolve()
                capture, canonical, metadata = self.capture(root)
                if mutation == 'truncated':
                    capture.write_text(capture.read_text().replace('Exit code: 0\n', ''), encoding='utf-8')
                else:
                    canonical.write_text(canonical.read_text().replace('original-head', 'different-head'), encoding='utf-8')
                before = capture.read_text()
                with self.assertRaises(DocContractFailure):
                    replay(capture, root/'output', metadata, canonical)
                self.assertEqual(capture.read_text(), before)

    def test_conflicting_preexisting_derived_events_refuse_instead_of_deduplicating_real_tests(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            capture, canonical, metadata = self.capture(root)
            event = {'type': 'test', 'event': 'failed', 'name': 'owner::doc$src/lib.rs (line 1)',
                     'test-cost': {'execution': 'rustdoc', 'block': 0}}
            with capture.open('a', encoding='utf-8') as stream:
                stream.write(json.dumps(event) + '\n')
            with self.assertRaises(DocContractFailure) as failure:
                replay(capture, root/'output', metadata, canonical)
            self.assertEqual(failure.exception.kind, DocFailureKind.RETAINED_EVENTS)


if __name__ == '__main__':
    unittest.main()
