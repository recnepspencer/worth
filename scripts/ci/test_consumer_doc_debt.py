"""Focused predecessor-ratchet behavior; no compiler or product worlds."""
import unittest
from check_consumer_doc_debt import compare, rows


class ConsumerDebtTests(unittest.TestCase):
    def test_paid_and_unchanged_rows_are_allowed(self):
        self.assertEqual(compare({"a.md": 2, "b.md": 0}, {"a.md": 1, "b.md": 0}), [])

    def test_simultaneous_fence_and_baseline_growth_cannot_hide(self):
        self.assertEqual(compare({"a.md": 1}, {"a.md": 2}),
                         ["a.md: debt rose from 1 to 2"])

    def test_new_document_starts_without_debt_and_coverage_cannot_disappear(self):
        self.assertEqual(compare({}, {"a.md": 0}), [])
        self.assertTrue(compare({}, {"a.md": 1}))
        self.assertEqual(compare({"a.md": 0}, {}),
                         ["a.md: consumer document coverage was removed"])

    def test_duplicate_and_negative_rows_are_refused(self):
        for text in [
            "[[documents]]\npath='a.md'\nunclassified=-1\n",
            "[[documents]]\npath='a.md'\nunclassified=0\n"
            "[[documents]]\npath='a.md'\nunclassified=0\n",
        ]:
            with self.assertRaises(ValueError):
                rows(text)


if __name__ == "__main__":
    unittest.main()
