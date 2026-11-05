"""Behaviour proofs for the visibility revert behind sealed test doors."""
import unittest

from src_visibility_revert import reverted


class VisibilityRevertTests(unittest.TestCase):
    def test_visibility_only_changes_return_to_base(self):
        base = 'mod a;\npub(crate) fn f() {}\nstruct S {\n    x: u8,\n}\n'
        current = 'pub mod a;\npub fn f() {}\nstruct S {\n    pub x: u8,\n}\n'
        text, count = reverted(base, current)
        self.assertEqual(text, base)
        self.assertEqual(count, 3)

    def test_other_changes_are_kept(self):
        base = '#[cfg(test)]\npub(crate) fn f() {}\nfn g() -> u8 { 1 }\n'
        current = 'pub fn f() {}\nfn g() -> u8 { 2 }\npub fn added() {}\n'
        text, count = reverted(base, current)
        self.assertEqual(text, 'pub(crate) fn f() {}\nfn g() -> u8 { 2 }\npub fn added() {}\n')
        self.assertEqual(count, 1)

    def test_idempotent(self):
        base = 'pub(super) struct T;\n'
        once, _ = reverted(base, 'pub struct T;\n')
        twice, count = reverted(base, once)
        self.assertEqual((once, count), (twice, 0))


if __name__ == '__main__':
    unittest.main()
