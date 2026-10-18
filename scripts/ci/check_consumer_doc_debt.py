"""Refuse consumer fence debt growth or coverage removal against a Git predecessor."""
import argparse
import os
from pathlib import Path
import subprocess
import sys
import tomllib

ROOT = Path(__file__).resolve().parents[2]
DEBT = "tools/boundary-check/snapshots/consumer-doc-fence-debt.toml"
CONFIG = "tools/boundary-check/config/consumer-docs.toml"


def rows(text):
    result = {}
    for row in tomllib.loads(text)["documents"]:
        path, count = row["path"], row["unclassified"]
        if path in result or type(count) is not int or count < 0:
            raise ValueError(f"invalid or duplicate debt row: {path}")
        result[path] = count
    return result


def compare(previous, current):
    errors = []
    for path, count in current.items():
        if count > previous.get(path, 0):
            errors.append(f"{path}: debt rose from {previous.get(path, 0)} to {count}")
    for path in previous.keys() - current.keys():
        errors.append(f"{path}: consumer document coverage was removed")
    return errors


def git(*arguments):
    return subprocess.run(["git", *arguments], cwd=ROOT, capture_output=True,
                          text=True, encoding="utf-8", check=False)


def predecessor(explicit):
    if explicit:
        result = git("rev-parse", "--verify", f"{explicit}^{{commit}}")
        if result.returncode:
            raise ValueError(f"cannot resolve debt predecessor {explicit}")
        return result.stdout.strip()
    candidates = ([f"origin/{os.environ['GITHUB_BASE_REF']}"]
                  if os.environ.get("GITHUB_BASE_REF") else [])
    candidates += ["origin/master", "origin/main", "master", "main"]
    for ref in candidates:
        result = git("merge-base", "HEAD", ref)
        if result.returncode == 0:
            return result.stdout.strip()
    raise ValueError("no Git predecessor available; pass --base or fetch the base branch")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--base", help="pull-request base or push predecessor commit")
    args = parser.parse_args()
    try:
        base = predecessor(args.base)
        current = rows((ROOT / DEBT).read_text(encoding="utf-8"))
        documents = tomllib.loads((ROOT / CONFIG).read_text(encoding="utf-8"))["documents"]
        if len(set(documents)) != len(documents) or set(documents) != current.keys():
            raise ValueError("debt rows must match the exact configured document inventory")
        previous = git("show", f"{base}:{DEBT}")
        if previous.returncode:
            listing = git("ls-tree", "--name-only", base, "--", DEBT)
            if listing.returncode or listing.stdout.strip():
                raise ValueError("cannot read predecessor debt")
            # The first introduction is seeded and reviewed against actual documents.
            # Every subsequent change is compared, including simultaneous fence/debt edits.
            print("consumer-doc debt: initial baseline; no predecessor debt exists")
            return 0
        errors = compare(rows(previous.stdout), current)
        if errors:
            raise ValueError("\n".join(errors))
        print("consumer-doc debt: no baseline growth or coverage removal")
        return 0
    except (OSError, ValueError, KeyError) as error:
        print(f"consumer-doc debt: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
