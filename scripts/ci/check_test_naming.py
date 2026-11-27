"""Reject review-round and process-artifact names on Rust source files and tests.

Every Rust module filename and `#[test]` function name should describe the
property, invariant, or behavior it protects rather than a review round or
hardening pass (for example `_r2`, `_round1`, `hardening_r3`, `sol_round2`).

Usage:
    python scripts/ci/check_test_naming.py
    python scripts/ci/check_test_naming.py --root <path>
    python scripts/ci/check_test_naming.py --self-test
"""

from __future__ import annotations

import argparse
from dataclasses import dataclass
from pathlib import Path
import re
import subprocess
import sys
import tempfile


ROOT = Path(__file__).resolve().parents[2]
_QUALITY_DIR = str(ROOT / "scripts" / "quality")
if _QUALITY_DIR not in sys.path:
    sys.path.insert(0, _QUALITY_DIR)

from rust_function_scrutiny import sanitize_rust  # noqa: E402

SKIP_DIRS = {".git", "target", ".lake", "node_modules"}
GIT_CANDIDATE_PATTERN = r"[A-Za-z0-9_]*(_r[0-9]+|round_?[0-9]+|sol_blocker)"
CANDIDATE_NAME_RE = re.compile(
    r"[A-Za-z0-9_]*(?:_r\d+|round_?\d+|sol_blocker)",
    re.IGNORECASE,
)
TEST_FN_RE = re.compile(
    r"#\s*\[\s*(?:(?:(?:core|std)\s*::\s*prelude\s*::\s*v1|(?:tokio|async_std))\s*::\s*)?"
    r"test(?:\s*\([^)]*\))?\s*\]"
    r"(?:\s*#\s*\[[^\]]*\])*"
    r"\s*(?:pub(?:\s*\([^)]*\))?\s+)?(?:async\s+)?(?:unsafe\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)",
    re.MULTILINE,
)


@dataclass(frozen=True, order=True)
class Violation:
    path: str
    line: int
    kind: str
    name: str
    reason: str

    def format(self) -> str:
        loc = f"{self.path}:{self.line}" if self.line > 0 else self.path
        return f"{loc}: forbidden {self.kind} `{self.name}` ({self.reason})"


def _fs_path(path: Path) -> Path:
    """Use extended-length path prefix on Windows so >260-char paths stay readable."""
    if sys.platform == "win32":
        raw = str(path.resolve())
        if not raw.startswith("\\\\?\\"):
            return Path("\\\\?\\" + raw)
    return path


def review_round_reason(identifier: str) -> str | None:
    """Return a concise diagnostic reason if `identifier` carries a review-round tag."""
    segments = identifier.lower().split("_")
    for index, seg in enumerate(segments):
        if index > 0 and len(seg) >= 2 and seg[0] == "r" and seg[1:].isdigit():
            return f"review-round suffix `_{seg}`"
        if seg.startswith("round") and len(seg) > 5 and seg[5:].isdigit():
            return f"review-round token `{seg}`"
        if seg == "round" and index + 1 < len(segments) and segments[index + 1].isdigit():
            return f"review-round token `round_{segments[index + 1]}`"
        if seg == "sol" and index + 1 < len(segments) and segments[index + 1] == "blocker":
            return "review-blocker token `sol_blocker`"
    return None


def _check_test_functions(rel: str, file_path: Path, violations: list[Violation]) -> None:
    fs_file = _fs_path(file_path)
    if not fs_file.is_file():
        return
    raw = fs_file.read_text(encoding="utf-8", errors="replace").replace("\r\n", "\n")
    if "#" not in raw or "test" not in raw or CANDIDATE_NAME_RE.search(raw) is None:
        return
    sanitized = sanitize_rust(raw)
    for match in TEST_FN_RE.finditer(sanitized):
        fn_name = match.group(1)
        fn_reason = review_round_reason(fn_name)
        if fn_reason is not None:
            line = sanitized.count("\n", 0, match.start(1)) + 1
            violations.append(Violation(rel, line, "#[test] fn", fn_name, fn_reason))


def _scan_git_repository(root: Path) -> tuple[int, list[Violation]]:
    listed = subprocess.run(
        [
            "git",
            "-C",
            str(root),
            "ls-files",
            "-z",
            "--cached",
            "--others",
            "--exclude-standard",
            "--",
            "*.rs",
        ],
        check=True,
        capture_output=True,
        text=True,
        encoding="utf-8",
    )
    paths = {p for p in listed.stdout.split("\0") if p}
    deleted = subprocess.run(
        ["git", "-C", str(root), "ls-files", "--deleted", "-z", "--", "*.rs"],
        check=True,
        capture_output=True,
        text=True,
        encoding="utf-8",
    )
    paths.difference_update(p for p in deleted.stdout.split("\0") if p)

    violations: list[Violation] = []
    for rel in sorted(paths):
        name = rel.rsplit("/", 1)[-1]
        stem = name[:-3] if name.endswith(".rs") else name
        reason = review_round_reason(stem)
        if reason is not None:
            violations.append(Violation(rel, 0, "filename", name, reason))

    # Fast C-level pre-filter over tracked + untracked non-ignored Rust files.
    grep = subprocess.run(
        [
            "git",
            "-C",
            str(root),
            "grep",
            "--untracked",
            "--exclude-standard",
            "-I",
            "-z",
            "-l",
            "-E",
            "-i",
            "-e",
            GIT_CANDIDATE_PATTERN,
            "--",
            "*.rs",
        ],
        capture_output=True,
        text=True,
        encoding="utf-8",
    )
    if grep.returncode not in (0, 1):
        raise subprocess.CalledProcessError(grep.returncode, grep.args, grep.stdout, grep.stderr)
    candidates = [p for p in grep.stdout.split("\0") if p and p in paths]
    for rel in sorted(candidates):
        _check_test_functions(rel, root / rel, violations)

    violations.sort()
    return len(paths), violations


def _scan_filesystem(root: Path) -> tuple[int, list[Violation]]:
    files: list[Path] = []
    for path in sorted(root.rglob("*.rs")):
        if any(part in SKIP_DIRS for part in path.relative_to(root).parts):
            continue
        if _fs_path(path).is_file():
            files.append(path)
    violations: list[Violation] = []
    for file_path in files:
        rel = file_path.relative_to(root).as_posix()
        reason = review_round_reason(file_path.stem)
        if reason is not None:
            violations.append(Violation(rel, 0, "filename", file_path.name, reason))
        _check_test_functions(rel, file_path, violations)
    violations.sort()
    return len(files), violations


def scan_repository(root: Path) -> tuple[int, list[Violation]]:
    if (root / ".git").exists():
        return _scan_git_repository(root)
    return _scan_filesystem(root)


def run_self_test() -> int:
    forbidden = [
        "bounded_naming_r2",
        "bounded_naming_r2_ack",
        "bounded_naming_r15",
        "hardening_r4",
        "sol_round1_hygiene_court",
        "sol_round2_durable_key_courts",
        "adversarial_round2_identity_bounds",
        "foo_round_3_checks",
        "round1_regression",
        "sol_r2_blocker_6_seed_invariant",
        "sol_blocker_4_zero_work_session",
    ]
    for name in forbidden:
        assert review_round_reason(name) is not None, f"expected forbidden: {name}"

    allowed = [
        "naming_ack_retention_and_forks",
        "continuation_bounds_and_retention_floor",
        "macro_hygiene_court",
        "r2d2_connection_pool",
        "envelope_roundtrips",
        "region_fork_roundtrips",
        "r14_kill_then_rail_restart_with_lost_state_never_claims_completion",
        "v2_manifest_roundtrip",
    ]
    for name in allowed:
        assert review_round_reason(name) is None, f"unexpected violation for: {name}"

    for use_git in (False, True):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            if use_git:
                subprocess.run(
                    ["git", "-C", str(root), "init", "--quiet"],
                    check=True,
                    capture_output=True,
                )
            (root / "tests").mkdir()
            (root / "tests" / "hardening_r2.rs").write_text(
                "#[test]\nfn semantic_property_holds() {}\n", encoding="utf-8"
            )
            (root / "tests" / "semantic_property.rs").write_text(
                "// #[test] fn ignored_r2_in_comment() {}\n"
                "const QUOTE_CHAR: char = '\"';\n"
                'const SAMPLE: &str = "#[test] fn ignored_round1_in_string() {}";\n'
                "#[test]\n#[ignore]\nfn catches_round2_regression() {}\n"
                "#[test]\nfn valid_r2d2_pool_roundtrip() {}\n",
                encoding="utf-8",
            )
            (root / "tests" / "isolated_multiline.rs").write_text(
                "#[\n    test\n]\nfn /* split */ \n    catches_multiline_r3() {}\n"
                "#[\n    tokio :: test\n]\nasync fn\n    catches_multiline_tokio_round4() {}\n",
                encoding="utf-8",
            )
            if use_git:
                subprocess.run(
                    ["git", "-C", str(root), "add", "tests/hardening_r2.rs"],
                    check=True,
                    capture_output=True,
                )
            scanned, violations = scan_repository(root)
            assert scanned == 3, f"use_git={use_git}: expected 3 scanned files, got {scanned}"
            assert len(violations) == 4, f"use_git={use_git}: expected 4 violations, got {violations}"
            assert violations[0].kind == "filename" and violations[0].name == "hardening_r2.rs"
            assert (
                violations[1].path == "tests/isolated_multiline.rs"
                and violations[1].kind == "#[test] fn"
                and violations[1].name == "catches_multiline_r3"
                and violations[1].line == 5
            )
            assert (
                violations[2].path == "tests/isolated_multiline.rs"
                and violations[2].kind == "#[test] fn"
                and violations[2].name == "catches_multiline_tokio_round4"
                and violations[2].line == 10
            )
            assert (
                violations[3].path == "tests/semantic_property.rs"
                and violations[3].kind == "#[test] fn"
                and violations[3].name == "catches_round2_regression"
                and violations[3].line == 6
            )
    print("[test-naming] SELF-TEST PASS")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=ROOT)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        return run_self_test()
    scanned, violations = scan_repository(args.root.resolve())
    if not violations:
        print(f"[test-naming] PASS: {scanned} Rust files checked, 0 review-round names")
        return 0
    print(
        f"[test-naming] FAIL: {len(violations)} review-round name violation(s) found:",
        file=sys.stderr,
    )
    for violation in violations:
        print(f"    {violation.format()}", file=sys.stderr)
    print(
        "Rename each flagged file or #[test] function to describe the property or behavior it protects.",
        file=sys.stderr,
    )
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
