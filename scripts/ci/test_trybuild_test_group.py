"""Compile-session discovery follows court paths, including long Windows paths."""
import pathlib
import tempfile
import unittest
from unittest.mock import patch

import check_trybuild_test_group as group


class SessionDiscovery(unittest.TestCase):
    def test_long_court_path_retains_compile_session_membership(self):
        directory = tempfile.gettempdir()
        if group.os.name == "nt":
            directory = "\\\\?\\" + directory
        with tempfile.TemporaryDirectory(dir=directory) as temporary:
            root = pathlib.Path(temporary)
            crate = root / "crates" / ("worth-court-" + "boundary-" * 12 + "artifacts")
            tests = crate / "tests" / crate.name.replace("-", "_")
            session = tests / "compile_fail" / "basis" / "admitted_preparation" / "mod.rs"
            session.parent.mkdir(parents=True)
            session.write_text("#[test]\nfn rejects_foreign_basis() { TestCases::new(); }", encoding="utf-8")
            misplaced = tests / "basis" / "mod.rs"
            misplaced.parent.mkdir()
            misplaced.write_text("#[test]\nfn refuses_unbound_basis() { TestCases::new(); }", encoding="utf-8")
            ordinary_root = pathlib.Path(str(root).removeprefix("\\\\?\\"))
            with patch.object(group, "ROOT", ordinary_root):
                names, failures = group.session_tests()
            self.assertEqual(names, {"rejects_foreign_basis"})
            self.assertEqual(failures, [str(misplaced.relative_to(root))])


if __name__ == "__main__":
    unittest.main()
