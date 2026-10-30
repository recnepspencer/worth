from pathlib import Path
import subprocess
import tempfile
import unittest

from workspace import read_tree


class ExplicitModuleDirectoryTests(unittest.TestCase):
    def test_raw_identifier_uses_the_unprefixed_source_filename(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            (root / 'lib.rs').write_text('mod r#async;')
            (root / 'async.rs').write_text('')
            tree = read_tree(root / 'lib.rs')
            self.assertEqual([node.source.name for node in tree.walk()], ['lib.rs', 'async.rs'])

    def test_path_attribute_loads_child_modules_beside_the_selected_file(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            (root / 'lib.rs').write_text('#[path="selected.rs"] mod renamed;')
            (root / 'selected.rs').write_text('mod sibling;')
            (root / 'sibling.rs').write_text('')
            observed = subprocess.run(['rustc', '--crate-type=lib', '--emit=metadata',
                                       str(root / 'lib.rs'), '-o', str(root / 'probe.rmeta')],
                                      capture_output=True, text=True)
            self.assertEqual(observed.returncode, 0, observed.stderr)
            tree = read_tree(root / 'lib.rs')
            self.assertEqual([node.source.name for node in tree.walk()],
                             ['lib.rs', 'selected.rs', 'sibling.rs'])


if __name__ == '__main__':
    unittest.main()
