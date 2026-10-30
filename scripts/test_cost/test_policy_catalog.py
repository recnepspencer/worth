import json
from pathlib import Path
import tempfile
import unittest

from policy_catalog import files, read, write


class BoundedPolicyCatalogTests(unittest.TestCase):
    def test_large_source_archive_and_reference_collections_have_bounded_tree_nodes(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            catalogs = {}
            for family in ('sources', 'archives', 'references'):
                rows = [{'family': family, 'ordinal': index} for index in range(10_101)]
                catalogs[family] = write(root, family, rows)
                self.assertEqual(read(root, catalogs[family]), rows)
                self.assertGreater(len(files(root, catalogs[family])), 100)
            (root/'index.json').write_text(json.dumps({'schema': 2, 'catalogs': catalogs}, indent=2), encoding='utf-8')
            self.assertTrue(all(len(path.read_text().splitlines()) <= 400 for path in root.iterdir()))
            self.assertTrue(all(len(path.read_text().splitlines()) <= 101 for path in root.glob('*.jsonl')))
            for path in root.glob('*.jsonl'):
                self.assertLessEqual(json.loads(path.read_text().splitlines()[0])['count'], 100)

    def test_repeated_rows_remain_distinct_occurrences_and_newline_translation_preserves_hashes(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            rows = [{'value': 1}] * 201
            descriptor = write(root, 'samples', rows)
            for path in root.glob('*.jsonl'):
                path.write_bytes(path.read_bytes().replace(b'\r\n', b'\n'))
            self.assertEqual(read(root, descriptor), rows)
            self.assertEqual(read(root, write(root, 'empty', [])), [])

    def test_changed_non_newline_bytes_and_escaping_references_are_refused(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            descriptor = write(root, 'samples', [{'value': 1}])
            path = root/descriptor['file']
            path.write_text(path.read_text().replace('"value": 1', '"value": 2'), encoding='utf-8')
            with self.assertRaisesRegex(ValueError, 'hash mismatch'):
                read(root, descriptor)
            with self.assertRaisesRegex(ValueError, 'escapes its owner'):
                read(root, dict(descriptor, file='../outside.jsonl'))


if __name__ == '__main__':
    unittest.main()
