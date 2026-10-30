from pathlib import Path
import tempfile
import unittest

from supported_programs import emit, expose_entry


class SupportedProgramCutoverTests(unittest.TestCase):
    def test_only_top_level_main_becomes_visible(self):
        source = 'mod nested { fn main() {} }\nfn main() { nested(); }'
        changed, invocation = expose_entry(source)
        self.assertIn('mod nested { fn main() {} }', changed)
        self.assertIn('pub(crate) fn main() { nested(); }', changed)
        self.assertEqual(invocation, 'program::main();')

    def test_ordinary_test_executes_the_program_and_preserves_a_moved_test_name(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            source = root / 'tests/ui/valid.rs'
            source.parent.mkdir(parents=True)
            source.write_text('fn main() { assert_eq!(2+2,4); }')
            writes = emit(root, 'court', [source], [('public_input_is_supported', [source])])
            wrapper = next(value for path, value in writes.items() if path.name == 'public_input_is_supported.rs')
            self.assertIn('fn public_input_is_supported()', wrapper)
            self.assertIn('program::main();', wrapper)
            self.assertIn('assert_eq!(2+2,4)', writes[source])

    def test_result_returning_program_keeps_its_failure_posture(self):
        _, invocation = expose_entry('fn main() -> Result<(),Error> { Ok(()) }')
        self.assertIn('.expect(', invocation)

    def test_visibility_precedes_an_async_entry_modifier(self):
        changed, _ = expose_entry('#[runtime::main]\nasync fn main() {}')
        self.assertIn('pub(crate) async fn main()', changed)


if __name__ == '__main__':
    unittest.main()
