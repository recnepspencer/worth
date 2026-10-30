import unittest
from rust_calls import arguments, calls


class RustCallSpanTests(unittest.TestCase):
    def test_nested_calls_literals_and_trailing_comma_keep_exact_arguments(self):
        text = 'read(root, "suite", dependency(&[("a", root, &["feature"])]), /*note*/ "production",)'
        found = calls(text, {'read'})
        self.assertEqual(len(found), 1)
        self.assertEqual(arguments(text, found[0]),
                         ['root', '"suite"', 'dependency(&[("a", root, &["feature"])])', '"production"'])

    def test_definition_is_not_a_call_and_qualified_call_is_found(self):
        text = 'fn read(value: Value) {} scope::read(value);'
        found = calls(text, {'read'})
        self.assertEqual(len(found), 1)
        self.assertEqual(arguments(text, found[0]), ['value'])


if __name__ == '__main__':
    unittest.main()
