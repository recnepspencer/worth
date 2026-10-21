"""One immutable environment for a complete Query gate build and its tests."""
import os
from pathlib import Path
import shutil
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from application_build import application_build_environment


def environment(root):
    # One fresh embedded identity for this complete gate build invocation.
    env = application_build_environment(os.environ)
    # Test binaries are run directly, so cargo's [env] table does not reach them.
    env.setdefault("RUST_MIN_STACK", "16777216")
    env.setdefault("WORTH_REPO", str(root))
    env.setdefault("CARGO", shutil.which("cargo") or "cargo")
    if shutil.which("sccache") and "RUSTC_WRAPPER" not in env:
        if env.get("CARGO_INCREMENTAL") == "1":
            del env["CARGO_INCREMENTAL"]
        env["RUSTC_WRAPPER"] = "sccache"
    return env
