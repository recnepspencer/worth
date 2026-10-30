from pathlib import Path
import tempfile
import tomllib
import unittest

from source_roots import roots


class CargoSourceRootTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name).resolve()

    def declaration(self, body, paths, edition='2024'):
        for value in paths:
            path = self.root / value
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text('fn main() {}')
        return tomllib.loads(f'[package]\nname="world"\nversion="0.0.0"\nedition="{edition}"\n' + body)

    def identities(self, declaration, inherited=None):
        return {(row['family'], row['name'], row['source'].relative_to(self.root).as_posix())
                for row in roots(self.root, declaration, inherited)}

    def test_autodiscovered_binary_files_and_directories_are_owned_roots(self):
        declaration = self.declaration('', ['src/main.rs', 'src/bin/first.rs', 'src/bin/second/main.rs'])
        self.assertEqual(self.identities(declaration), {
            ('bin', 'world', 'src/main.rs'), ('bin', 'first', 'src/bin/first.rs'),
            ('bin', 'second', 'src/bin/second/main.rs')})

    def test_direct_main_files_keep_their_target_name(self):
        declaration = self.declaration('', ['src/bin/main.rs', 'tests/main.rs', 'tests/nested/main.rs'])
        self.assertEqual(self.identities(declaration), {
            ('bin', 'main', 'src/bin/main.rs'), ('test', 'main', 'tests/main.rs'),
            ('test', 'nested', 'tests/nested/main.rs')})

    def test_disabled_auto_families_do_not_scan_uncompiled_files(self):
        declaration = self.declaration('autobins=false\nautolib=false\nautotests=false\n'
                                       'autobenches=false\nautoexamples=false\n'
                                       '[[bin]]\nname="owned"\npath="owned.rs"\n',
                                       ['owned.rs', 'src/lib.rs', 'src/main.rs', 'src/bin/not_owned.rs',
                                        'tests/not_owned.rs', 'benches/not_owned.rs', 'examples/not_owned.rs'])
        self.assertEqual(self.identities(declaration), {('bin', 'owned', 'owned.rs')})

    def test_declared_names_infer_flat_and_directory_sources(self):
        declaration = self.declaration('[[bin]]\nname="binary"\n[[test]]\nname="court"\n'
                                       '[[bench]]\nname="cost"\n[[example]]\nname="journey"\n',
                                       ['src/bin/binary.rs', 'tests/court/main.rs',
                                        'benches/cost/main.rs', 'examples/journey.rs'])
        self.assertEqual(self.identities(declaration), {
            ('bin', 'binary', 'src/bin/binary.rs'), ('test', 'court', 'tests/court/main.rs'),
            ('bench', 'cost', 'benches/cost/main.rs'), ('example', 'journey', 'examples/journey.rs')})

    def test_declared_names_and_paths_override_autodiscovery(self):
        declaration = self.declaration('[[bin]]\nname="renamed"\npath="src/bin/original.rs"\n'
                                       '[[test]]\nname="replaced"\npath="owned.rs"\n',
                                       ['src/bin/original.rs', 'tests/replaced.rs', 'owned.rs'])
        self.assertEqual(self.identities(declaration), {
            ('bin', 'renamed', 'src/bin/original.rs'), ('test', 'replaced', 'owned.rs')})

    def test_old_edition_disables_only_the_explicitly_declared_auto_family(self):
        declaration = self.declaration('[[bin]]\nname="owned"\npath="owned.rs"\n',
                                       ['owned.rs', 'src/main.rs', 'src/bin/other.rs', 'tests/claim.rs'],
                                       edition='2015')
        self.assertEqual(self.identities(declaration), {
            ('bin', 'owned', 'owned.rs'), ('test', 'claim', 'tests/claim.rs')})

    def test_inherited_edition_preserves_modern_auto_discovery(self):
        declaration = self.declaration('[[bin]]\nname="owned"\npath="owned.rs"\n',
                                       ['owned.rs', 'src/main.rs'])
        declaration['package']['edition'] = {'workspace': True}
        with self.assertRaisesRegex(ValueError, 'inherited package edition'):
            roots(self.root, declaration)
        self.assertEqual(self.identities(declaration, '2024'), {
            ('bin', 'owned', 'owned.rs'), ('bin', 'world', 'src/main.rs')})

    def test_missing_or_ambiguous_declared_sources_refuse(self):
        declaration = self.declaration('[[test]]\nname="claim"\n', [])
        with self.assertRaisesRegex(ValueError, 'no unique source'):
            roots(self.root, declaration)
        declaration = self.declaration('[[test]]\nname="claim"\n', ['tests/claim.rs', 'tests/claim/main.rs'])
        with self.assertRaisesRegex(ValueError, 'no unique source'):
            roots(self.root, declaration)


if __name__ == '__main__':
    unittest.main()
