"""Record exact source hashes and completed lane headers for independent review."""
from pathlib import Path
import hashlib, json, subprocess

root = Path.cwd()
tracked = subprocess.check_output(['git', 'diff', '--name-only', '--', 'crates', 'workspaces'], text=True).splitlines()
untracked = subprocess.check_output(['git', 'ls-files', '--others', '--exclude-standard', '--', 'crates', 'workspaces'], text=True).splitlines()
paths = sorted({p for p in tracked + untracked if p.endswith('.rs') and (root / p).is_file()})
sources = {p: hashlib.sha256((root / p).read_bytes()).hexdigest() for p in paths}
head = subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip()
Path('target/lane-evidence/area-funding-qualified-source.json').write_text(json.dumps({'head': head, 'dirty_rust_source_sha256': sources}, indent=2) + '\n')
rows = []
for p in sorted(Path('target/lane-evidence').glob('area-funding-*.log')):
    s = p.read_text(encoding='utf-8', errors='replace')
    def last(prefix):
        return next((line[len(prefix):] for line in reversed(s.splitlines()) if line.startswith(prefix)), None)
    if not s.startswith('command: '):
        continue
    rows.append((p.name, last('exit_code: '), '/'.join(last(k) or '?' for k in ['tests_passed: ', 'tests_failed: ', 'tests_ignored: '])))
text = '# Area funding lane index\n\n'
text += f'Recorded HEAD `{head}`, dirty source; exact source hashes in `area-funding-qualified-source.json`. Logs carry exact command, HEAD, dirty flag, UTC times and exit. Counts are selected passed/failed/ignored; inventory and skipped nextest events are not executed tests. An absent exit is still running. This index does not claim a completed funding landing.\n\n'
text += '| Log | Exit | P/F/I |\n|---|---:|---:|\n'
for name, code, counts in rows:
    text += f'| [{name}]({name}) | {code or "RUNNING"} | {counts} |\n'
text += '\nThe Host16 packet failed 14/2/0: the cold-read assertion used the writer cut before a later committed original; the exact corrected-cut case passed separately. The unchanged bounded writer hit its 120-second deadline in the packet and passed when selected alone. Neither its deadline, workload, budget nor its <=40 assertion changed. See `cold-capture-read-cut-independent-review.md` for the independent assertion review.\n'
text += '\nRunner-generated Root slot telemetry is retained as generated. Its additions are the three focused CDC/recovery/scalar observations; changes to content-addressed catalog shards are not test deletions. No ignored, scale, exhaustive Home oracle or Store scenario test was selected.\n'
text += '\nInitial Root Clippy stopped on11 lint errors. Six equivalent-expression corrections are applied in the active Complete/storage funding owners; final Root Clippy confirms only the five exact outside-area Native sites remain. One outside-area Store close site stops Store Clippy. All six source files are untouched; `proposed-outside-area-clippy.patch` is prepared only. Exact paths are in `area-funding-outside-area-blockers.md`.\n'
text += '\nBoundary initially refused two CP1252 em-dash bytes in an earlier review Markdown. Re-encoding preserves exact decoded wording/newlines against the retained original bytes; corrected boundary evaluation reaches only the three queued BC9002 sites. The first patch-apply dry run failed because Windows text output normalized LF contexts to CRLF; the patch writer now preserves each source newline form, and the corrected dry run passes. Neither dry run changes source. Agent-context, dirty line caps and diff whitespace checks pass at the logged source states.\n'
text += '\nThe first wide-contract court compile failed: the front door changes type after the first field, and the court lacked the facade import for its denial type. The corrected builder/import compile passes. Its first runtime run passed the existing six Native cases but failed the new outer-wrapper assertion. SDK ExecutionWorkCeiling::run_within deliberately returns the callback value once entered; the corrected court requires outer success while preserving the exact inner WorkExhausted, zero memory and successful retry. Independent review confirmed this API contract correction. Only the failed case was rerun.\n'
text += '\nInitial Native12, Host16 plus focused corrections, Store4, CDC/recovery3 and head41 measurement precede the final contract-walk work precharge. Later area-funding-contract-* logs identify the focused post-correction family: existing Native6, the separately corrected public wide-contract case, Host2 and Store1. Do not present earlier all-family observations as exhaustive post-correction qualification. No budget was raised. The final source hash snapshot covers the latest dirty Rust files.\n'
text += '\nThe final boundary run adds two BC7013 findings for the new wide-contract court to the three queued BC9002 sites. The two exact closed-caller entries are proposed, not applied, in `proposed-contract-court-callers.patch`; its dry-run passes. Config is outside the present area and remains untouched. Final dirty line caps pass36 files and agent-context check passes. Full Clippy and boundary remain red; no funding landing is certified.\n'
Path('target/lane-evidence/area-funding-lanes.md').write_text(text)
print(f'Recorded {len(paths)} source hashes and {len(rows)} lane records at {head}')
