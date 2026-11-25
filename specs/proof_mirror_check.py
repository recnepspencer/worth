"""Drift guard for the hand-mirrored proofs under specs/verus and specs/lean.

Each proof file carries one header line per source range it mirrors:

    proof-mirror: <repo-path>:<start>-<end> sha256:<hex>

where <hex> is the SHA-256 of those exact lines (1-based, inclusive, joined
with "\\n", line endings normalized to LF). This script recomputes every hash
and exits 1 naming each range whose lines changed, or 0 when all match.

Usage: python specs/proof_mirror_check.py            (check)
       python specs/proof_mirror_check.py --print    (also list every range)
Stdlib only; run from anywhere inside the repository.
"""
import hashlib
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
PROOFS = ["specs/verus", "specs/lean"]
LINE = re.compile(r"proof-mirror:\s+(\S+):(\d+)-(\d+)\s+sha256:([0-9a-f]{64})")


def digest(path: str, start: int, end: int) -> str:
    lines = (ROOT / path).read_text(encoding="utf-8").replace("\r\n", "\n").split("\n")
    if start < 1 or end > len(lines) or start > end:
        return "out-of-range"
    return hashlib.sha256("\n".join(lines[start - 1:end]).encode("utf-8")).hexdigest()


def main(argv: list[str]) -> int:
    failures, checked = [], 0
    for folder in PROOFS:
        for proof in sorted((ROOT / folder).rglob("*")):
            if not proof.is_file() or proof.suffix not in {".rs", ".lean"}:
                continue
            for match in LINE.finditer(proof.read_text(encoding="utf-8")):
                path, start, end, expected = match.group(1), int(match.group(2)), int(match.group(3)), match.group(4)
                checked += 1
                actual = digest(path, start, end) if (ROOT / path).is_file() else "missing-file"
                if "--print" in argv:
                    print(f"{proof.relative_to(ROOT)}: {path}:{start}-{end} {'ok' if actual == expected else 'CHANGED'}")
                if actual != expected:
                    failures.append(f"{proof.relative_to(ROOT)} mirrors {path}:{start}-{end}, which changed ({actual})")
    if checked == 0:
        print("proof-mirror: no mirrored ranges found")
        return 1
    for failure in failures:
        print(failure)
    print(f"proof-mirror: {checked} ranges checked, {len(failures)} changed")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
