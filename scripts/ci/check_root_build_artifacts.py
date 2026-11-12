"""Keep repository-root target output out of Git, even after git add --force."""

import argparse
from pathlib import Path
import subprocess
import sys


ROOT = Path(__file__).resolve().parents[2]


def tracked_build_artifacts(root: Path) -> list[str]:
    result = subprocess.run(
        [
            "git", "-C", str(root), "ls-files", "--cached", "-z", "--",
            ":(top)target", ":(top,glob)target-*/**",
        ],
        check=True, capture_output=True, text=True, encoding="utf-8",
    )
    return sorted(set(result.stdout.split("\0")) - {""})


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=ROOT)
    arguments = parser.parse_args()
    try:
        artifacts = tracked_build_artifacts(arguments.root)
    except subprocess.CalledProcessError as error:
        print(f"[root-build-artifacts] Git inspection failed: {error.stderr.strip()}",
              file=sys.stderr)
        return 2
    if not artifacts:
        print("[root-build-artifacts] PASS: no tracked root target output")
        return 0
    print("[root-build-artifacts] FAIL: root target output is tracked:", file=sys.stderr)
    for artifact in artifacts:
        print(f"    {artifact}", file=sys.stderr)
    print("Untrack these paths with git rm --cached; retain raw logs locally or in CI artifacts.",
          file=sys.stderr)
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
