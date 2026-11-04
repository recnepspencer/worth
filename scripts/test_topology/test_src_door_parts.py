"""Behaviour proofs for destructuring doors."""
import unittest

from src_door_members_text import struct_shape
from src_door_parts import parts_door


class PartsDoorTests(unittest.TestCase):
    def test_parts_door_returns_every_field_in_declaration_order(self):
        shape = struct_shape('pub struct R<T> where T: Clone {\n    b: Vec<T>,\n    pub(crate) a: u8,\n}')
        door = parts_door(shape)
        self.assertIn('impl<T> R<T> where T: Clone {', door)
        self.assertIn('pub fn into_parts_for_test(self) -> (Vec<T>, u8,) {', door)
        self.assertIn('(self.b, self.a,)', door)


if __name__ == '__main__':
    unittest.main()
