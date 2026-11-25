#!/usr/bin/env python3
r"""Run Lean 4 (`lake build` or `lean`) with Lean-kernel axiom & type verification.

Usage:
  python tools/lean/run_lean.py build <project_dir> [--manifest <theorems.json>] [target...]
  python tools/lean/run_lean.py check <file.lean> [--theorem <QualifiedName> ...]
  python tools/lean/run_lean.py version

Instead of fragile regex parsing, certification queries Lean's own kernel directly:
  - Discovers all theorems declared in the module via `env.constants.map₂` (or checks
    every explicitly requested/manifested qualified theorem name 1-to-1).
  - Parses each manifest `type` through `Lean.Parser.runParserCategory env \`term`
    (exactly one type expression, zero syntax injection) and checks `isDefEq`.
  - Collects transitive axioms via `Lean.collectAxioms`, unconditionally rejecting
    `sorryAx` and any axiom outside `{propext, Classical.choice, Quot.sound}` (+ allowlist).
"""

from __future__ import annotations

import json
import pathlib
import re
import shutil
import subprocess
import sys
import tempfile


FORBIDDEN_TOKEN_RE = re.compile(r"\b(sorry|admit|native_decide)\b")
IMPLEMENTED_BY_RE = re.compile(r"@\[[^\]]*\bimplemented_by\b")
STANDARD_AXIOMS = ["propext", "Classical.choice", "Quot.sound"]


def find_tool(name: str) -> str:
    elan_bin = pathlib.Path.home() / ".elan" / "bin"
    for suffix in (".exe", ""):
        cand = elan_bin / f"{name}{suffix}"
        if cand.is_file():
            return str(cand)
    in_path = shutil.which(name)
    if in_path:
        return in_path
    raise SystemExit(f"ERROR: `{name}` not found. Install Lean 4 via elan.")


def collect_lean_files(target: pathlib.Path) -> list[pathlib.Path]:
    if target.is_file():
        return sorted(p for p in target.parent.glob("*.lean") if p.is_file())
    return [
        p
        for p in sorted(target.rglob("*.lean"))
        if ".lake" not in p.parts and p.name != "lakefile.lean"
    ]


def scan_shortcuts(files: list[pathlib.Path]) -> list[str]:
    hits: list[str] = []
    for path in files:
        try:
            text = path.read_text(encoding="utf-8")
        except OSError as exc:
            hits.append(f"{path}: unreadable file: {exc}")
            continue
        in_block = 0
        for idx, raw_line in enumerate(text.splitlines(), 1):
            out: list[str] = []
            i = 0
            while i < len(raw_line):
                if raw_line[i : i + 2] == "/-":
                    in_block += 1
                    i += 2
                elif raw_line[i : i + 2] == "-/" and in_block > 0:
                    in_block -= 1
                    i += 2
                else:
                    if in_block == 0:
                        out.append(raw_line[i])
                    i += 1
            code = "".join(out).split("--", 1)[0]
            if FORBIDDEN_TOKEN_RE.search(code):
                hits.append(f"{path}:{idx}: forbidden token: {raw_line.strip()}")
            if IMPLEMENTED_BY_RE.search(code):
                hits.append(f"{path}:{idx}: forbidden @[implemented_by]: {raw_line.strip()}")
    return hits


def lean_name_expr(qual_name: str) -> str:
    parts = [p for p in qual_name.split(".") if p]
    expr = "Lean.Name.anonymous"
    for p in parts:
        expr = f"Lean.Name.mkStr ({expr}) {json.dumps(p)}"
    return expr


def build_kernel_audit_block(
    requested: list[tuple[str, str | None]],
    allowed_axioms: list[str],
    discover_all_in_module: bool,
) -> str:
    allowed_lean = "[" + ", ".join(lean_name_expr(a) for a in sorted(set(allowed_axioms))) + "]"
    req_items = []
    for name, ty in requested:
        ty_opt = f"some {json.dumps(ty)}" if ty is not None else "none"
        req_items.append(f"({lean_name_expr(name)}, {ty_opt})")
    req_lean = "[" + ", ".join(req_items) + "]"
    discover_flag = "true" if discover_all_in_module else "false"
    return f"""
-- automated Lean kernel audit
#eval show Lean.Elab.Command.CommandElabM Unit from do
  let env ← Lean.getEnv
  let allowed : List Lean.Name := {allowed_lean}
  let requested : List (Lean.Name × Option String) := {req_lean}
  let targets : List (Lean.Name × Option String) ←
    if {discover_flag} then
      let mut discovered : List (Lean.Name × Option String) := []
      for (n, info) in env.constants.map₂.toList do
        if let Lean.ConstantInfo.thmInfo _ := info then
          if !n.isInternal then
            discovered := (n, none) :: discovered
      pure discovered
    else
      pure requested
  if targets.isEmpty then
    throwError "WORTH_AUDIT_FAIL: zero theorems to audit"
  for (n, tyOpt) in targets do
    let some info := env.find? n
      | throwError s!"WORTH_AUDIT_FAIL: theorem '{{n}}' not found in environment"
    let Lean.ConstantInfo.thmInfo _ := info
      | throwError s!"WORTH_AUDIT_FAIL: '{{n}}' is not a theorem"
    if let some tyStr := tyOpt then
      match Lean.Parser.runParserCategory env `term tyStr with
      | .error err =>
        throwError s!"WORTH_AUDIT_FAIL: invalid type expression for '{{n}}': {{err}}"
      | .ok stx =>
        let ok ← Lean.Elab.Command.liftTermElabM do
          let expectedTy ← Lean.Elab.Term.elabType stx
          let expectedTy ← Lean.instantiateMVars expectedTy
          Lean.Meta.isDefEq info.type expectedTy
        if !ok then
          throwError s!"WORTH_AUDIT_FAIL: type mismatch for theorem '{{n}}'"
    let axs ← Lean.collectAxioms n
    for ax in axs do
      if ax == `sorryAx || !allowed.contains ax then
        throwError s!"WORTH_AUDIT_FAIL: theorem '{{n}}' depends on disallowed axiom '{{ax}}'"
  IO.println s!"WORTH_AUDIT_OK: {{targets.length}}"
"""


def run_audit_code(
    cmd_prefix: list[str], cwd: pathlib.Path, code: str
) -> int:
    with tempfile.NamedTemporaryFile(
        "w", suffix=".lean", encoding="utf-8", delete=False, dir=str(cwd)
    ) as tf:
        tf.write(code)
        audit_path = pathlib.Path(tf.name)
    try:
        proc = subprocess.run(
            [*cmd_prefix, str(audit_path)],
            cwd=str(cwd),
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
        )
    finally:
        audit_path.unlink(missing_ok=True)
    if proc.returncode != 0 or "WORTH_AUDIT_OK:" not in proc.stdout:
        sys.stderr.write(proc.stdout)
        sys.stderr.write(proc.stderr)
        return proc.returncode or 1
    for line in proc.stdout.splitlines():
        if line.startswith("WORTH_AUDIT_OK:"):
            print(f"Lean kernel audit PASSED ({line.split(':', 1)[1].strip()} theorem(s)).", flush=True)
            break
    return 0


def audit_file(lean: str, target: pathlib.Path, explicit_theorems: list[str]) -> int:
    if len(set(explicit_theorems)) != len(explicit_theorems):
        print("ERROR: duplicate --theorem names requested.", file=sys.stderr)
        return 2
    src = target.read_text(encoding="utf-8")
    requested = [(n, None) for n in explicit_theorems]
    block = build_kernel_audit_block(
        requested=requested,
        allowed_axioms=STANDARD_AXIOMS,
        discover_all_in_module=(len(requested) == 0),
    )
    code = "import Lean\n" + src + "\n" + block
    return run_audit_code([lean], target.parent, code)


def audit_manifest(lake: str, project_dir: pathlib.Path, manifest_path: pathlib.Path) -> int:
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    imports: list[str] = manifest.get("imports", [])
    theorems: list[dict[str, str]] = manifest.get("theorems", [])
    raw_allowed: list[str] = manifest.get("allowed_axioms", [])
    if "sorryAx" in raw_allowed:
        print("ERROR: manifest `allowed_axioms` cannot include `sorryAx`.", file=sys.stderr)
        return 1
    if not theorems:
        print("ERROR: manifest lists zero required theorems.", file=sys.stderr)
        return 1

    requested: list[tuple[str, str | None]] = []
    seen: set[str] = set()
    for idx, entry in enumerate(theorems):
        name = (entry.get("name") or "").strip()
        expected_type = (entry.get("type") or "").strip()
        if not name or not expected_type:
            print(f"ERROR: manifest entry #{idx} requires non-empty `name` and `type`.", file=sys.stderr)
            return 1
        if name in seen:
            print(f"ERROR: duplicate theorem '{name}' in manifest.", file=sys.stderr)
            return 1
        seen.add(name)
        requested.append((name, expected_type))

    header = "\n".join(["import Lean", *(f"import {m}" for m in imports)])
    block = build_kernel_audit_block(
        requested=requested,
        allowed_axioms=[*STANDARD_AXIOMS, *raw_allowed],
        discover_all_in_module=False,
    )
    return run_audit_code([lake, "env", "lean"], project_dir, header + "\n" + block)


def main(argv: list[str]) -> int:
    if len(argv) < 2:
        print(__doc__)
        return 2
    mode = argv[1]
    if mode == "version":
        return subprocess.run([find_tool("lean"), "--version"]).returncode or subprocess.run(
            [find_tool("lake"), "--version"]
        ).returncode
    if len(argv) < 3:
        print(__doc__)
        return 2

    target = pathlib.Path(argv[2]).resolve()
    if not target.exists():
        raise SystemExit(f"ERROR: target not found: {target}")

    rest = argv[3:]
    manifest_path: pathlib.Path | None = None
    explicit_theorems: list[str] = []
    extra: list[str] = []
    i = 0
    while i < len(rest):
        if rest[i] == "--manifest" and i + 1 < len(rest):
            manifest_path = pathlib.Path(rest[i + 1]).resolve()
            i += 2
        elif rest[i] == "--theorem" and i + 1 < len(rest):
            explicit_theorems.append(rest[i + 1])
            i += 2
        else:
            extra.append(rest[i])
            i += 1

    hits = scan_shortcuts(collect_lean_files(target))
    if hits:
        print("ERROR: forbidden Lean proof shortcuts found:", file=sys.stderr)
        for h in hits:
            print(f"  {h}", file=sys.stderr)
        return 1

    if mode == "build":
        lake = find_tool("lake")
        rc = subprocess.run([lake, "build", *extra], cwd=str(target)).returncode
        if rc != 0:
            return rc
        return audit_manifest(lake, target, manifest_path) if manifest_path else 0

    if mode == "check":
        lean = find_tool("lean")
        rc = subprocess.run([lean, str(target), *extra], cwd=str(target.parent)).returncode
        if rc != 0:
            return rc
        return audit_file(lean, target, explicit_theorems)

    print(__doc__)
    return 2


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
