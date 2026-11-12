"""The Git index, not local build output or source module names, governs hygiene."""

from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


CHECKER = Path(__file__).with_name("check_root_build_artifacts.py")


class RootBuildArtifactTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.git("init", "--quiet")
        self.write(".gitignore", "/target/\n/target-*/\n")
        self.git("add", ".gitignore")

    def git(self, *arguments):
        return subprocess.run(
            ["git", "-C", str(self.root), *arguments],
            check=True, capture_output=True, text=True, encoding="utf-8",
        )

    def write(self, relative, contents="local evidence\n"):
        destination = self.root / relative
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_text(contents, encoding="utf-8")

    def check(self):
        return subprocess.run(
            [sys.executable, str(CHECKER), "--root", str(self.root)],
            capture_output=True, text=True, encoding="utf-8",
        )

    def test_local_output_and_source_target_modules_are_allowed(self):
        self.write("target/lane-evidence/local.log")
        self.write("target-native/debug/local.bin")
        for source in ("src/target/owner.rs", "src/target-admission/owner.rs",
                       "scripts/test_cost/slots/sample.json"):
            self.write(source)
            self.git("add", source)
        result = self.check()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("PASS", result.stdout)

    def test_force_added_root_output_is_rejected_even_after_local_deletion(self):
        artifacts = ("target/lane-evidence/raw log.txt",
                     "target/test-cost/selection.json", "target-native/debug/cache.bin")
        for artifact in artifacts:
            self.write(artifact)
            self.git("add", "--force", artifact)
        (self.root / artifacts[0]).unlink()
        result = self.check()
        self.assertEqual(result.returncode, 1, result.stderr)
        for artifact in artifacts:
            self.assertIn(artifact, result.stderr)

    def test_untracking_output_passes_without_deleting_local_evidence(self):
        artifact = "target/lane-evidence/retained.log"
        self.write(artifact)
        self.git("add", "--force", artifact)
        self.assertEqual(self.check().returncode, 1)
        self.git("rm", "--cached", "--", artifact)
        result = self.check()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual((self.root / artifact).read_text(encoding="utf-8"), "local evidence\n")

    def test_git_inspection_failure_never_reports_a_clean_tree(self):
        result = subprocess.run(
            [sys.executable, str(CHECKER), "--root", str(self.root / "missing")],
            capture_output=True, text=True, encoding="utf-8",
        )
        self.assertEqual(result.returncode, 2, result.stderr)
        self.assertIn("Git inspection failed", result.stderr)
        self.assertNotIn("PASS", result.stdout)


if __name__ == "__main__":
    unittest.main()
