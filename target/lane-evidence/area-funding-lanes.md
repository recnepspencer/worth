# Area funding lane index

Recorded HEAD `367ddc069042a3b024fea129dde92aea8bceb837`, dirty source; exact source hashes in `area-funding-qualified-source.json`. Logs carry exact command, HEAD, dirty flag, UTC times and exit. Counts are selected passed/failed/ignored; inventory and skipped nextest events are not executed tests. An absent exit is still running. This index does not claim a completed funding landing.

| Log | Exit | P/F/I |
|---|---:|---:|
| [area-funding-agent-context.log](area-funding-agent-context.log) | 0 | 0/0/0 |
| [area-funding-attribution-parser-control.log](area-funding-attribution-parser-control.log) | 0 | 0/0/0 |
| [area-funding-boundary-utf8-corrected.log](area-funding-boundary-utf8-corrected.log) | 1 | 0/0/0 |
| [area-funding-boundary.log](area-funding-boundary.log) | 2 | 0/0/0 |
| [area-funding-bounded-tail-focused.log](area-funding-bounded-tail-focused.log) | 0 | 1/0/0 |
| [area-funding-cdc-recovery.log](area-funding-cdc-recovery.log) | 0 | 3/0/0 |
| [area-funding-clippy-correction-format.log](area-funding-clippy-correction-format.log) | 0 | 0/0/0 |
| [area-funding-cold-cut-court.log](area-funding-cold-cut-court.log) | 0 | 1/0/0 |
| [area-funding-cold-cut-format.log](area-funding-cold-cut-format.log) | 0 | 0/0/0 |
| [area-funding-contract-caller-patch-check.log](area-funding-contract-caller-patch-check.log) | 0 | 0/0/0 |
| [area-funding-contract-check.log](area-funding-contract-check.log) | 101 | 0/0/0 |
| [area-funding-contract-corrected-check.log](area-funding-contract-corrected-check.log) | 0 | 0/0/0 |
| [area-funding-contract-corrected-format.log](area-funding-contract-corrected-format.log) | 0 | 0/0/0 |
| [area-funding-contract-format.log](area-funding-contract-format.log) | 0 | 0/0/0 |
| [area-funding-contract-host.log](area-funding-contract-host.log) | 0 | 2/0/0 |
| [area-funding-contract-native.log](area-funding-contract-native.log) | 100 | 6/1/0 |
| [area-funding-contract-store.log](area-funding-contract-store.log) | 0 | 1/0/0 |
| [area-funding-contract-wrapper-court.log](area-funding-contract-wrapper-court.log) | 0 | 1/0/0 |
| [area-funding-contract-wrapper-format.log](area-funding-contract-wrapper-format.log) | 0 | 0/0/0 |
| [area-funding-cutover-check.log](area-funding-cutover-check.log) | 0 | 0/0/0 |
| [area-funding-cutover-format.log](area-funding-cutover-format.log) | 0 | 0/0/0 |
| [area-funding-diff-check.log](area-funding-diff-check.log) | 0 | 0/0/0 |
| [area-funding-final-agent-context.log](area-funding-final-agent-context.log) | 0 | 0/0/0 |
| [area-funding-final-boundary.log](area-funding-final-boundary.log) | 1 | 0/0/0 |
| [area-funding-final-line-caps.log](area-funding-final-line-caps.log) | 0 | 0/0/0 |
| [area-funding-final-root-clippy.log](area-funding-final-root-clippy.log) | 101 | 0/0/0 |
| [area-funding-head41-attribution.log](area-funding-head41-attribution.log) | 0 | 0/0/0 |
| [area-funding-head41-instrumented.log](area-funding-head41-instrumented.log) | 0 | 1/0/0 |
| [area-funding-host-selection.log](area-funding-host-selection.log) | 100 | 14/2/0 |
| [area-funding-instrumentation-preflight.log](area-funding-instrumentation-preflight.log) | 0 | 0/0/0 |
| [area-funding-line-caps.log](area-funding-line-caps.log) | 0 | 0/0/0 |
| [area-funding-native-owner-format.log](area-funding-native-owner-format.log) | 0 | 0/0/0 |
| [area-funding-native-selection.log](area-funding-native-selection.log) | 0 | 12/0/0 |
| [area-funding-outside-clippy-patch-check.log](area-funding-outside-clippy-patch-check.log) | 1 | 0/0/0 |
| [area-funding-outside-clippy-patch-corrected-check.log](area-funding-outside-clippy-patch-corrected-check.log) | 0 | 0/0/0 |
| [area-funding-owner-check.log](area-funding-owner-check.log) | 0 | 0/0/0 |
| [area-funding-root-clippy.log](area-funding-root-clippy.log) | 101 | 0/0/0 |
| [area-funding-store-clippy.log](area-funding-store-clippy.log) | 101 | 0/0/0 |
| [area-funding-store-selection.log](area-funding-store-selection.log) | 0 | 4/0/0 |

The Host16 packet failed 14/2/0: the cold-read assertion used the writer cut before a later committed original; the exact corrected-cut case passed separately. The unchanged bounded writer hit its 120-second deadline in the packet and passed when selected alone. Neither its deadline, workload, budget nor its <=40 assertion changed. See `cold-capture-read-cut-independent-review.md` for the independent assertion review.

Runner-generated Root slot telemetry is retained as generated. Its additions are the three focused CDC/recovery/scalar observations; changes to content-addressed catalog shards are not test deletions. No ignored, scale, exhaustive Home oracle or Store scenario test was selected.

Initial Root Clippy stopped on11 lint errors. Six equivalent-expression corrections are applied in the active Complete/storage funding owners; final Root Clippy confirms only the five exact outside-area Native sites remain. One outside-area Store close site stops Store Clippy. All six source files are untouched; `proposed-outside-area-clippy.patch` is prepared only. Exact paths are in `area-funding-outside-area-blockers.md`.

Boundary initially refused two CP1252 em-dash bytes in an earlier review Markdown. Re-encoding preserves exact decoded wording/newlines against the retained original bytes; corrected boundary evaluation reaches only the three queued BC9002 sites. The first patch-apply dry run failed because Windows text output normalized LF contexts to CRLF; the patch writer now preserves each source newline form, and the corrected dry run passes. Neither dry run changes source. Agent-context, dirty line caps and diff whitespace checks pass at the logged source states.

The first wide-contract court compile failed: the front door changes type after the first field, and the court lacked the facade import for its denial type. The corrected builder/import compile passes. Its first runtime run passed the existing six Native cases but failed the new outer-wrapper assertion. SDK ExecutionWorkCeiling::run_within deliberately returns the callback value once entered; the corrected court requires outer success while preserving the exact inner WorkExhausted, zero memory and successful retry. Independent review confirmed this API contract correction. Only the failed case was rerun.

Initial Native12, Host16 plus focused corrections, Store4, CDC/recovery3 and head41 measurement precede the final contract-walk work precharge. Later area-funding-contract-* logs identify the focused post-correction family: existing Native6, the separately corrected public wide-contract case, Host2 and Store1. Do not present earlier all-family observations as exhaustive post-correction qualification. No budget was raised. The final source hash snapshot covers the latest dirty Rust files.

The final boundary run adds two BC7013 findings for the new wide-contract court to the three queued BC9002 sites. The two exact closed-caller entries are proposed, not applied, in `proposed-contract-court-callers.patch`; its dry-run passes. Config is outside the present area and remains untouched. Final dirty line caps pass36 files and agent-context check passes. Full Clippy and boundary remain red; no funding landing is certified.
