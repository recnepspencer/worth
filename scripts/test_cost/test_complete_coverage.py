import unittest

from complete_coverage import merge


def test(name, binary='owner', ignored=False):
    return {'binary': binary, 'test': name, 'ignored': ignored}


class CompleteCoverageTests(unittest.TestCase):
    def test_native_and_doc_owners_are_distinct_and_ignored_snippets_remain_scheduled(self):
        native = test('ordinary')
        snippet = test('src/lib.rs (line 1)', 'owner::doc')
        ignored = test('src/lib.rs (line 9)', 'owner::doc', True)
        known, selections, proof = merge([native], {'ordinary': [native]},
                                        {'inventory': [snippet, ignored], 'selections': [snippet]})
        self.assertEqual(len(known), 3)
        self.assertEqual(selections['documentation'], [snippet])
        self.assertEqual(proof['status'], 'pass')
        self.assertEqual(proof['scheduled_only'], 1)
        self.assertEqual([r['selections'] for r in proof['tests'] if not r['ignored']], [1, 1])

    def test_native_selection_gaps_remain_visible_after_documentation_is_added(self):
        _, _, proof = merge([test('missing')], {}, {'inventory': [], 'selections': []})
        self.assertEqual(proof['status'], 'fail')
        self.assertEqual(proof['zero'][0]['test'], 'missing')

    def test_partial_invocations_can_omit_documentation_without_inventing_inventory(self):
        native = test('ordinary')
        known, selections, proof = merge([native], {'ordinary': [native]}, None)
        self.assertEqual(known, [native])
        self.assertNotIn('documentation', selections)
        self.assertEqual(proof['status'], 'pass')

    def test_missing_ignored_duplicate_unknown_or_conflicting_doc_selections_are_refused(self):
        snippet = test('src/lib.rs (line 1)', 'owner::doc')
        ignored = dict(snippet, ignored=True)
        for docs in ({'inventory': [snippet], 'selections': []},
                     {'inventory': [ignored], 'selections': [ignored]},
                     {'inventory': [snippet], 'selections': [snippet, snippet]},
                     {'inventory': [], 'selections': [snippet]},
                     {'inventory': [test('wrong-owner')], 'selections': [test('wrong-owner')]}):
            with self.subTest(docs=docs), self.assertRaises(ValueError):
                merge([], {}, docs)
        with self.assertRaises(ValueError):
            merge([snippet], {}, {'inventory': [snippet], 'selections': [snippet]})
        with self.assertRaises(ValueError):
            merge([], {'documentation': []}, {'inventory': [], 'selections': []})


if __name__ == '__main__':
    unittest.main()
