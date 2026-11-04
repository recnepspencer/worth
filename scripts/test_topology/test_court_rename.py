"""Behaviour proofs for court renames and merges."""
import tempfile
import unittest
from pathlib import Path

from court_rename import list_field, merged_mod, replace_names, set_list


class CourtRenameTests(unittest.TestCase):
    def test_names_are_replaced_exactly_never_as_a_prefix(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / 'docs').mkdir()
            doc = root / 'docs' / 'a.md'
            doc.write_text('crates/court-a/tests/court_a/x.rs court-a-b court_a_b "court-a"\n')
            self.assertEqual(replace_names(root, 'court-a', 'court-z'), 1)
            self.assertEqual(doc.read_text(), 'crates/court-z/tests/court_z/x.rs court-a-b court_a_b "court-z"\n')

    def test_shared_mod_files_unite_their_module_declarations(self):
        ours = '//! Doc.\n#[path = "a/mod.rs"]\npub mod a;\npub mod b;\n'
        theirs = '//! Other doc.\n#[path = "a/mod.rs"]\npub mod a;\n#[path = "c/mod.rs"]\npub mod c;\nuse x::y;\n'
        self.assertEqual(merged_mod(ours, theirs), ours + '#[path = "c/mod.rs"]\npub mod c;\n')

    def test_split_lists_are_read_and_written(self):
        block = 'court = "a"\nowners = ["x", "y"]\ndelegates = []\n'
        self.assertEqual(list_field(block, 'owners'), ['x', 'y'])
        self.assertIn('owners = ["x", "y", "z"]', set_list(block, 'owners', ['x', 'y', 'z']))


if __name__ == '__main__':
    unittest.main()
