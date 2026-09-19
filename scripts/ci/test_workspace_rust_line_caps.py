from __future__ import annotations

import os
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CHECKER = ROOT / "scripts/ci/check_workspace_rust_line_caps.sh"


class WorkspaceRustLineCapTests(unittest.TestCase):
    def test_worth_ui_app_scope_includes_application_rust(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            app = root / "workspaces/worth-ui/apps/pulse/src/main.rs"
            app.parent.mkdir(parents=True)
            app.write_text(
                "\n".join("// over cap" for _ in range(401)) + "\n",
                encoding="utf-8",
            )
            allowlist = root / "scripts/ci/workspace_rust_line_cap_allowlist.txt"
            allowlist.parent.mkdir(parents=True)
            allowlist.write_text("", encoding="utf-8")
            subprocess.run(["git", "init", "--quiet"], cwd=root, check=True)
            subprocess.run(["git", "add", "."], cwd=root, check=True)
            environment = dict(os.environ, WORTH_WORKSPACE_ROOT=str(root))
            bash = shutil.which("bash") or "bash"
            if os.name == "nt":
                git_bash = Path(r"C:\Program Files\Git\bin\bash.exe")
                if git_bash.is_file():
                    bash = str(git_bash)
            result = subprocess.run(
                [bash, str(CHECKER), "worth-ui-apps"], cwd=root, env=environment,
                capture_output=True, text=True, check=False,
            )
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("workspaces/worth-ui/apps/pulse/src/main.rs", result.stdout)


if __name__ == "__main__":
    unittest.main()
