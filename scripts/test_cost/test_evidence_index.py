from pathlib import Path
import tempfile
import unittest

from evidence_index import entry


class EvidenceIndexTests(unittest.TestCase):
    def observed(self, output):
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / 'lane.log'
            path.write_text('Command: ["python", "lane.py"]\nHEAD: reviewed\n'
                            'Git status --short empty: False\nStart: 2026-10-30T00:00:00+00:00\n\n' + output +
                            '\nEnd: 2026-10-30T00:00:02+00:00\nExit code: 0\n', encoding='utf-8')
            return entry(path)

    def test_unknown_counts_are_not_zero_and_nested_headers_do_not_replace_owner(self):
        row = self.observed('HEAD: child\nCommand: ["child"]\n')
        self.assertIsNone(row['counts'])
        self.assertEqual(row['headers']['HEAD'], 'reviewed')
        self.assertTrue(row['complete_header'])

    def test_unittest_aggregates_preserve_failures_and_skipped_counts(self):
        row = self.observed('Ran 5 tests in 0.1s\n\nFAILED (failures=1, errors=1, skipped=1)\n')
        self.assertEqual(row['counts'], {'pass': 2, 'fail': 2, 'ignored': 1})

    def test_a_missing_outer_header_cannot_borrow_a_nested_crlf_header(self):
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / 'missing.log'
            path.write_bytes(b'Command: []\r\n\r\nHEAD: child\r\n')
            row = entry(path)
            self.assertIsNone(row['headers']['HEAD'])
            self.assertFalse(row['complete_header'])

    def test_native_plain_summaries_use_observed_counts(self):
        row = self.observed('test result: ok. 7 passed; 0 failed; 2 ignored; 0 measured;\n')
        self.assertEqual(row['counts'], {'pass': 7, 'fail': 0, 'ignored': 2})

    def test_windows_child_newline_translation_preserves_unittest_counts(self):
        row = self.observed('Ran 3 tests in 0.1s\r\r\n\r\r\nOK\r\r\n')
        self.assertEqual(row['counts'], {'pass': 3, 'fail': 0, 'ignored': 0})


if __name__ == '__main__':
    unittest.main()
