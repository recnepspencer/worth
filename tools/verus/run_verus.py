#!/usr/bin/env python3
"""Run Verus formal verification with strict TCB gating, flag allowlist & module closure.

Usage:
  python tools/verus/run_verus.py check <path/to/file.rs> [--min-verified <N>] [--crate-type lib]
  python tools/verus/run_verus.py version

Locates `verus` via VERUS_BIN env var, ~/.local/tools/verus/**/verus[.exe], or PATH.
Certification checks:
  1. Strict CLI argument & value allowlist: only non-weakening compiler/resource flags
     are permitted (`--crate-type`, `--edition`, `-L`, `--extern`, `--cfg`, `--rlimit`,
     `--num-threads`, `--multiple-errors`, `--time`, `--output-json`), and EVERY option
     value is validated strictly (rejecting any flag-like `-...` value or malformed arg).
  2. Requires `--min-verified >= 1` strictly (default 1).
  3. Computes full Rust module closure starting from `<file.rs>` (following `mod foo;`,
     `#[path = "..."] mod foo;`, and ALL `include!(...)` / `include![...]` / `include!{...}`
     delimiter forms including raw strings, failing closed on unresolved or dynamic
     inclusions/paths) unioned with the target directory tree.
  4. Scans complete comment-stripped token streams (catching multiline attributes!)
     for `admit(`, `assume(`, `unsafe`, and unreviewed
     `#[verifier::external_body|external|external_fn_specification|external_type_specification]`.
     Reviews MUST appear in an actual Rust comment immediately adjacent to the item
     with format `verus:tcb_reviewed(<item_name>: <non-empty reason>)` matching the
     affected item (string literals and mismatched/empty review markers are rejected).
  5. Parses `verification results:: <V> verified, <E> errors` and requires
     `E == 0` and `V >= min_verified`.
"""

from __future__ import annotations

import os
import pathlib
import re
import shutil
import subprocess
import sys


ADMIT_RE = re.compile(r"\b(admit|assume)\s*\(")
UNSAFE_RE = re.compile(r"\bunsafe\b")
TCB_ATTR_RE = re.compile(
    r"#\s*\[\s*(?:[^\]]*?\b)?verifier\s*::\s*(external_body|external|external_fn_specification|external_type_specification)\b[^\]]*\]",
    re.DOTALL,
)
MOD_DECL_RE = re.compile(
    r"(?:(?:#\s*\[\s*path\s*=\s*([^\]]+)\]\s*)+)?(?:\bpub(?:\s*\([^)]*\))?\s+)?\bmod\s+([A-Za-z_][A-Za-z0-9_]*)\s*;",
    re.DOTALL,
)
INCLUDE_START_RE = re.compile(r"\binclude\s*!\s*([(\[{])")
REVIEW_COMMENT_RE = re.compile(
    r"\bverus:tcb_reviewed\(\s*([A-Za-z_][A-Za-z0-9_:]*)\s*:\s*([^)\r\n]+?)\s*\)"
)
RESULT_RE = re.compile(r"verification results::\s*(\d+)\s+verified,\s*(\d+)\s+errors")

ALLOWED_CRATE_TYPES = {
    "lib",
    "rlib",
    "bin",
    "staticlib",
    "cdylib",
    "dylib",
    "proc-macro",
}
ALLOWED_EDITIONS = {"2018", "2021", "2024"}
ALLOWED_VALUE_FLAGS = {
    "--crate-type",
    "--edition",
    "-L",
    "--extern",
    "--cfg",
    "--rlimit",
    "--num-threads",
    "--multiple-errors",
}
ALLOWED_STANDALONE_FLAGS = {
    "--time",
    "--output-json",
}

POS_INT_RE = re.compile(r"^[1-9]\d*$")
CFG_VAL_RE = re.compile(r'^[A-Za-z_][A-Za-z0-9_]*(?:=(?:"[^"\r\n]*"|[A-Za-z0-9_]+))?$')
LIB_PATH_RE = re.compile(r"^(?:(?:dependency|crate|native|framework|all)=)?[^-\s]\S*$")
EXTERN_VAL_RE = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*(?:=[^-\s]\S*)?$")


def find_verus() -> str:
    env_bin = os.environ.get("VERUS_BIN")
    if env_bin and pathlib.Path(env_bin).is_file():
        return env_bin
    home = pathlib.Path.home()
    root = home / ".local" / "tools" / "verus"
    if root.is_dir():
        for name in ("verus.exe", "verus"):
            for hit in sorted(root.rglob(name)):
                if hit.is_file():
                    return str(hit)
    in_path = shutil.which("verus")
    if in_path:
        return in_path
    raise SystemExit(
        "ERROR: verus not found. Set VERUS_BIN or install a Verus release under ~/.local/tools/verus/."
    )


def lex_rust_comments_and_code(
    text: str,
) -> tuple[str, dict[int, list[str]], dict[int, bool]]:
    """Strip Rust comments preserving line numbers, and record real comment text per line.

    Properly skips string literals, raw string literals (`r"..."`, `r#"..."#`),
    and character literals so comment markers inside strings are never treated as comments
    and string contents are never mistaken for review comments.
    """
    out: list[str] = []
    comments_by_line: dict[int, list[str]] = {}
    i = 0
    n = len(text)
    line = 1

    def add_comment(ln: int, content: str) -> None:
        comments_by_line.setdefault(ln, []).append(content)

    while i < n:
        ch = text[i]
        nxt = text[i : i + 2]

        # Raw string literal: r"..." or r#*"..."#* (or br#*"..."#*)
        if ch in ("r", "b"):
            j = i + 1 if ch == "r" else (i + 2 if text[i : i + 2] == "br" else -1)
            if j != -1:
                hashes = 0
                while j < n and text[j] == "#":
                    hashes += 1
                    j += 1
                if j < n and text[j] == '"':
                    end_delim = '"' + ("#" * hashes)
                    end_idx = text.find(end_delim, j + 1)
                    if end_idx == -1:
                        seg = text[i:]
                        out.append(seg)
                        line += seg.count("\n")
                        i = n
                    else:
                        seg = text[i : end_idx + len(end_delim)]
                        out.append(seg)
                        line += seg.count("\n")
                        i = end_idx + len(end_delim)
                    continue

        # Standard string literal
        if ch == '"':
            out.append(ch)
            i += 1
            while i < n:
                c = text[i]
                out.append(c)
                if c == "\n":
                    line += 1
                if c == "\\" and i + 1 < n:
                    nxt_c = text[i + 1]
                    out.append(nxt_c)
                    if nxt_c == "\n":
                        line += 1
                    i += 2
                    continue
                if c == '"':
                    i += 1
                    break
                i += 1
            continue

        # Character literal (distinguish from lifetime like 'a)
        if ch == "'" and i + 2 < n:
            if text[i + 1] == "\\" and i + 3 < n:
                # Escaped char literal like '\'' or '\n' or '\x7f'
                end_q = text.find("'", i + 2)
                if end_q != -1 and end_q - i <= 8 and "\n" not in text[i:end_q]:
                    out.append(text[i : end_q + 1])
                    i = end_q + 1
                    continue
            elif text[i + 2] == "'" and text[i + 1] != "\n":
                out.append(text[i : i + 3])
                i += 3
                continue

        # Line comment
        if nxt == "//":
            start = i + 2
            while i < n and text[i] != "\n":
                out.append(" ")
                i += 1
            add_comment(line, text[start:i])
            continue

        # Nested block comment
        if nxt == "/*":
            depth = 1
            start_line = line
            buf: list[str] = []
            out.append("  ")
            i += 2
            while i < n and depth > 0:
                if text[i : i + 2] == "/*":
                    depth += 1
                    out.append("  ")
                    i += 2
                elif text[i : i + 2] == "*/":
                    depth -= 1
                    out.append("  ")
                    i += 2
                elif text[i] == "\n":
                    add_comment(start_line, "".join(buf))
                    buf.clear()
                    out.append("\n")
                    line += 1
                    start_line = line
                    i += 1
                else:
                    buf.append(text[i])
                    out.append(" ")
                    i += 1
            if buf:
                add_comment(start_line, "".join(buf))
            continue

        out.append(ch)
        if ch == "\n":
            line += 1
        i += 1

    cleaned = "".join(out)
    is_comment_only_line: dict[int, bool] = {}
    for ln_idx, cl in enumerate(cleaned.splitlines(), 1):
        is_comment_only_line[ln_idx] = (cl.strip() == "") and (ln_idx in comments_by_line)
    return cleaned, comments_by_line, is_comment_only_line


def parse_rust_string_literal(expr: str) -> str | None:
    s = expr.strip()
    if len(s) >= 2 and s[0] == '"' and s[-1] == '"':
        # Standard string literal (no unescaped internal quote)
        inner = s[1:-1]
        out: list[str] = []
        i = 0
        while i < len(inner):
            if inner[i] == "\\":
                if i + 1 >= len(inner):
                    return None
                esc = inner[i + 1]
                mapping = {"n": "\n", "r": "\r", "t": "\t", "\\": "\\", '"': '"', "'": "'"}
                out.append(mapping.get(esc, esc))
                i += 2
            elif inner[i] == '"':
                return None
            else:
                out.append(inner[i])
                i += 1
        return "".join(out)
    m = re.fullmatch(r'r(#*)"(.*)"\1', s, re.DOTALL)
    if m:
        hashes = m.group(1)
        inner = m.group(2)
        if ('"' + hashes) in inner:
            return None
        return inner
    return None


def extract_balanced_delim(text: str, open_pos: int) -> tuple[str, int] | None:
    open_ch = text[open_pos]
    close_ch = {"(": ")", "[": "]", "{": "}"}[open_ch]
    depth = 1
    i = open_pos + 1
    n = len(text)
    while i < n and depth > 0:
        ch = text[i]
        if ch == '"':
            i += 1
            while i < n:
                if text[i] == "\\" and i + 1 < n:
                    i += 2
                    continue
                if text[i] == '"':
                    i += 1
                    break
                i += 1
            continue
        if ch == "r":
            j = i + 1
            hashes = 0
            while j < n and text[j] == "#":
                hashes += 1
                j += 1
            if j < n and text[j] == '"':
                end_delim = '"' + ("#" * hashes)
                end_idx = text.find(end_delim, j + 1)
                if end_idx == -1:
                    return None
                i = end_idx + len(end_delim)
                continue
        if ch == open_ch:
            depth += 1
        elif ch == close_ch:
            depth -= 1
            if depth == 0:
                return text[open_pos + 1 : i], i + 1
        i += 1
    return None


def resolve_module_file(
    declaring_file: pathlib.Path,
    is_crate_root: bool,
    mod_name: str,
    explicit_path: str | None,
) -> pathlib.Path | None:
    if is_crate_root or declaring_file.name in ("lib.rs", "main.rs", "mod.rs"):
        base_dir = declaring_file.parent
    else:
        base_dir = declaring_file.parent / declaring_file.stem

    if explicit_path is not None:
        cand = (base_dir / explicit_path).resolve()
        if cand.is_file():
            return cand
        fallback = (declaring_file.parent / explicit_path).resolve()
        if fallback.is_file():
            return fallback
        return None

    for cand in (
        base_dir / f"{mod_name}.rs",
        base_dir / mod_name / "mod.rs",
    ):
        if cand.is_file():
            return cand.resolve()
    return None


def collect_module_closure(target: pathlib.Path) -> tuple[list[pathlib.Path], list[str]]:
    errors: list[str] = []
    discovered: set[pathlib.Path] = set()
    queue: list[tuple[pathlib.Path, bool]] = []

    if target.is_file():
        queue.append((target.resolve(), True))
        root_dir = target.parent.resolve()
    else:
        root_dir = target.resolve()
        for entry in (
            root_dir / "src" / "lib.rs",
            root_dir / "src" / "main.rs",
            root_dir / "lib.rs",
            root_dir / "main.rs",
        ):
            if entry.is_file():
                queue.append((entry.resolve(), True))

    for rs in sorted(p.resolve() for p in root_dir.rglob("*.rs") if "target" not in p.parts):
        if rs not in discovered:
            queue.append((rs, rs == target.resolve()))

    while queue:
        current, is_root = queue.pop(0)
        if current in discovered:
            continue
        discovered.add(current)
        try:
            raw = current.read_text(encoding="utf-8")
        except OSError as exc:
            errors.append(f"{current}: unreadable file: {exc}")
            continue
        cleaned, _, _ = lex_rust_comments_and_code(raw)

        for m in MOD_DECL_RE.finditer(cleaned):
            raw_path_expr, mod_name = m.group(1), m.group(2)
            explicit_path: str | None = None
            if raw_path_expr is not None:
                explicit_path = parse_rust_string_literal(raw_path_expr)
                if explicit_path is None:
                    errors.append(
                        f"{current}: unsupported non-literal #[path = {raw_path_expr.strip()}] on `mod {mod_name};`"
                    )
                    continue
            resolved = resolve_module_file(current, is_root, mod_name, explicit_path)
            if resolved is None:
                desc = (
                    f'#[path = "{explicit_path}"] mod {mod_name}'
                    if explicit_path is not None
                    else f"mod {mod_name}"
                )
                errors.append(f"{current}: unresolved module declaration `{desc};`")
            elif resolved not in discovered:
                queue.append((resolved, False))

        for inc_m in INCLUDE_START_RE.finditer(cleaned):
            open_pos = inc_m.end() - 1
            extracted = extract_balanced_delim(cleaned, open_pos)
            if extracted is None:
                errors.append(f"{current}: malformed `include!` macro invocation")
                continue
            inner_expr, _ = extracted
            inc_rel = parse_rust_string_literal(inner_expr)
            if inc_rel is None:
                errors.append(
                    f"{current}: dynamic/non-literal `include!({inner_expr.strip()})` cannot be audited in certification mode"
                )
                continue
            inc_path = (current.parent / inc_rel).resolve()
            if not inc_path.is_file():
                errors.append(f'{current}: unresolved `include!` target "{inc_rel}"')
            elif inc_path not in discovered:
                queue.append((inc_path, False))

    return sorted(discovered), errors


def line_number_at(text: str, pos: int) -> int:
    return text.count("\n", 0, pos) + 1


def extract_following_item_name(cleaned: str, start_pos: int) -> str | None:
    """Identify the Rust item name governed by an attribute or `unsafe` keyword."""
    pos = start_pos
    n = len(cleaned)
    while pos < n:
        # Skip whitespace
        while pos < n and cleaned[pos].isspace():
            pos += 1
        # Skip subsequent attributes #[...]
        if pos + 1 < n and cleaned[pos] == "#" and cleaned[pos + 1] == "[":
            res = extract_balanced_delim(cleaned, pos + 1)
            if res is None:
                break
            _, pos = res
            continue
        break

    tail = cleaned[pos : pos + 400]
    item_m = re.match(
        r"^(?:(?:pub(?:\s*\([^)]*\))?|async|const|unsafe|extern(?:\s+\"[^\"]*\")?|open|closed|spec|proof|exec|broadcast|default)\s+)*"
        r"(?:fn|struct|enum|trait|type|static|const|mod)\s+([A-Za-z_][A-Za-z0-9_]*)\b",
        tail,
        re.DOTALL,
    )
    if item_m:
        return item_m.group(1)

    impl_m = re.match(
        r"^(?:(?:unsafe|default)\s+)*impl(?:\s*<[^>]*>)?\s+(?:[A-Za-z_][A-Za-z0-9_:<>]*\s+for\s+)?([A-Za-z_][A-Za-z0-9_]*)",
        tail,
        re.DOTALL,
    )
    if impl_m:
        return impl_m.group(1)

    return None


def extract_enclosing_fn_name(cleaned: str, pos: int) -> str | None:
    prefix = cleaned[:pos]
    matches = list(re.finditer(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\b", prefix))
    if matches:
        return matches[-1].group(1)
    return None


def has_valid_tcb_review(
    start_line: int,
    end_line: int,
    expected_item: str | None,
    comments_by_line: dict[int, list[str]],
    is_comment_only_line: dict[int, bool],
    cleaned_lines: list[str],
) -> bool:
    if not expected_item:
        return False

    candidate_comments: list[str] = []
    # Comments on the matched lines themselves
    for ln in range(start_line, end_line + 1):
        candidate_comments.extend(comments_by_line.get(ln, []))

    # Walk upward through immediately preceding contiguous comment-only lines or attribute lines
    ln = start_line - 1
    while ln >= 1:
        if is_comment_only_line.get(ln, False):
            candidate_comments.extend(comments_by_line.get(ln, []))
            ln -= 1
            continue
        cl = cleaned_lines[ln - 1].strip()
        if cl.startswith("#[") or cl == "":
            candidate_comments.extend(comments_by_line.get(ln, []))
            ln -= 1
            continue
        break

    for ctext in candidate_comments:
        for m in REVIEW_COMMENT_RE.finditer(ctext):
            reviewed_item = m.group(1).strip()
            reason = m.group(2).strip()
            if len(reason) < 4:
                continue
            if reviewed_item == expected_item or reviewed_item.split("::")[-1] == expected_item:
                return True
    return False


def scan_tcb(files: list[pathlib.Path]) -> list[str]:
    hits: list[str] = []
    for path in files:
        try:
            raw = path.read_text(encoding="utf-8")
        except OSError as exc:
            hits.append(f"{path}: unreadable file: {exc}")
            continue
        lines = raw.splitlines()
        cleaned, comments_by_line, is_comment_only_line = lex_rust_comments_and_code(raw)
        cleaned_lines = cleaned.splitlines()

        for m in ADMIT_RE.finditer(cleaned):
            ln = line_number_at(cleaned, m.start())
            hits.append(f"{path}:{ln}: forbidden admit/assume: {lines[ln - 1].strip()}")

        for m in UNSAFE_RE.finditer(cleaned):
            ln = line_number_at(cleaned, m.start())
            item = extract_following_item_name(cleaned, m.start()) or extract_enclosing_fn_name(
                cleaned, m.start()
            )
            if not has_valid_tcb_review(
                ln, ln, item, comments_by_line, is_comment_only_line, cleaned_lines
            ):
                hits.append(
                    f"{path}:{ln}: unreviewed unsafe (item={item or '<unknown>'}): {lines[ln - 1].strip()}"
                )

        for m in TCB_ATTR_RE.finditer(cleaned):
            start_ln = line_number_at(cleaned, m.start())
            end_ln = line_number_at(cleaned, m.end())
            item = extract_following_item_name(cleaned, m.end())
            if not has_valid_tcb_review(
                start_ln, end_ln, item, comments_by_line, is_comment_only_line, cleaned_lines
            ):
                snippet = " ".join(
                    lines[ln - 1].strip() for ln in range(start_ln, end_ln + 1)
                )
                hits.append(
                    f"{path}:{start_ln}: unreviewed verifier TCB escape (item={item or '<unknown>'}): {snippet}"
                )
    return hits


def validate_flag_value(key: str, val: str) -> str | None:
    if not val or val.startswith("-"):
        return f"invalid value `{val}` for flag `{key}` (values cannot be empty or flag-like)"
    if key == "--crate-type":
        if val not in ALLOWED_CRATE_TYPES:
            return f"disallowed --crate-type `{val}` (allowed: {sorted(ALLOWED_CRATE_TYPES)})"
    elif key == "--edition":
        if val not in ALLOWED_EDITIONS:
            return f"disallowed --edition `{val}` (allowed: {sorted(ALLOWED_EDITIONS)})"
    elif key in ("--rlimit", "--num-threads", "--multiple-errors"):
        if not POS_INT_RE.fullmatch(val):
            return f"flag `{key}` requires a positive integer, got `{val}`"
    elif key == "--cfg":
        if not CFG_VAL_RE.fullmatch(val):
            return f"disallowed/malformed --cfg value `{val}`"
    elif key == "-L":
        if not LIB_PATH_RE.fullmatch(val):
            return f"disallowed/malformed -L value `{val}`"
    elif key == "--extern":
        if not EXTERN_VAL_RE.fullmatch(val):
            return f"disallowed/malformed --extern value `{val}`"
    return None


def validate_allowed_args(extra: list[str]) -> list[str]:
    errors: list[str] = []
    i = 0
    while i < len(extra):
        arg = extra[i]
        if arg in ALLOWED_STANDALONE_FLAGS:
            i += 1
            continue
        if "=" in arg:
            key, val = arg.split("=", 1)
            if key in ALLOWED_VALUE_FLAGS:
                err = validate_flag_value(key, val)
                if err:
                    errors.append(err)
                i += 1
                continue
            errors.append(
                f"disallowed Verus certification flag: `{arg}` (only allowlisted compiler/resource flags are permitted)"
            )
            i += 1
            continue
        if arg in ALLOWED_VALUE_FLAGS:
            if i + 1 >= len(extra):
                errors.append(f"flag `{arg}` requires a value")
                break
            val = extra[i + 1]
            err = validate_flag_value(arg, val)
            if err:
                errors.append(err)
            i += 2
            continue
        errors.append(
            f"disallowed Verus certification flag: `{arg}` (only allowlisted compiler/resource flags are permitted)"
        )
        i += 1
    return errors


def main(argv: list[str]) -> int:
    if len(argv) < 2:
        print(__doc__)
        return 2
    cmd = argv[1]
    verus = find_verus()

    if cmd == "version":
        return subprocess.run([verus, "--version"]).returncode

    if cmd != "check" or len(argv) < 3:
        print(__doc__)
        return 2

    target = pathlib.Path(argv[2]).resolve()
    if not target.exists():
        raise SystemExit(f"ERROR: target not found: {target}")

    rest = argv[3:]
    min_verified = 1
    extra: list[str] = []
    i = 0
    while i < len(rest):
        arg = rest[i]
        if arg == "--min-verified":
            if i + 1 >= len(rest):
                print("ERROR: --min-verified requires a positive integer", file=sys.stderr)
                return 2
            try:
                min_verified = int(rest[i + 1])
            except ValueError:
                print(
                    f"ERROR: --min-verified must be a positive integer, got `{rest[i + 1]}`",
                    file=sys.stderr,
                )
                return 2
            i += 2
            continue
        if arg.startswith("--min-verified="):
            val = arg.split("=", 1)[1]
            try:
                min_verified = int(val)
            except ValueError:
                print(
                    f"ERROR: --min-verified must be a positive integer, got `{val}`",
                    file=sys.stderr,
                )
                return 2
            i += 1
            continue
        if arg == "--":
            extra.extend(rest[i + 1 :])
            break
        extra.append(arg)
        i += 1

    if min_verified < 1:
        print(
            f"ERROR: --min-verified must be >= 1 in certification mode, got {min_verified}",
            file=sys.stderr,
        )
        return 2

    arg_errors = validate_allowed_args(extra)
    if arg_errors:
        for err in arg_errors:
            print(f"ERROR: {err}", file=sys.stderr)
        return 2

    files, closure_errors = collect_module_closure(target)
    if closure_errors:
        print("ERROR: Verus module closure check failed:", file=sys.stderr)
        for err in closure_errors:
            print(f"  {err}", file=sys.stderr)
        return 1

    hits = scan_tcb(files)
    if hits:
        print("ERROR: forbidden Verus TCB escapes or shortcuts found:", file=sys.stderr)
        for h in hits:
            print(f"  {h}", file=sys.stderr)
        return 1

    run_cmd = [verus, str(target), *extra]
    print("Running:", " ".join(run_cmd), flush=True)
    proc = subprocess.run(
        run_cmd,
        cwd=str(target.parent if target.is_file() else target),
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    sys.stdout.write(proc.stdout)
    sys.stderr.write(proc.stderr)
    if proc.returncode != 0:
        return proc.returncode

    combined = proc.stdout + "\n" + proc.stderr
    matches = RESULT_RE.findall(combined)
    if not matches:
        print(
            "ERROR: Verus produced no `verification results:: N verified, 0 errors` summary.",
            file=sys.stderr,
        )
        return 1
    verified, errors = int(matches[-1][0]), int(matches[-1][1])
    if errors != 0 or verified < min_verified:
        print(
            f"ERROR: Verus verification count check failed: verified={verified} (min={min_verified}), errors={errors}",
            file=sys.stderr,
        )
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
