# Actual backing owner lanes ? IN PROGRESS

All listed logs record the exact command, HEAD, dirty state, UTC start/end, exit and counts. They identify dirty source at `0215c2872b`; no clean funding landing is claimed. Owned build and selection component logs are retained under each FULL_OWNED_LOG_PATH. Source leaf hashes for the final four-case run are in `actual-owner-qualified-source.json`.

Counts below are selected passed/failed/ignored. Nextest also emits unselected ignored inventory entries; none was selected or executed. Compiler/format lanes have zero test cases.

| Log | Exit | Passed/failed/selected ignored |
|---|---:|---:|
| [actual-owner-complete-preflight-check.log](actual-owner-complete-preflight-check.log) | 0 | 0/0/0 |
| [actual-owner-complete-preflight-format.log](actual-owner-complete-preflight-format.log) | 0 | 0/0/0 |
| [actual-owner-complete-preflight-native.log](actual-owner-complete-preflight-native.log) | 0 | 4/0/0 |
| [actual-owner-conversion-check.log](actual-owner-conversion-check.log) | 101 | 0/0/0 |
| [actual-owner-conversion-corrected-check.log](actual-owner-conversion-corrected-check.log) | 0 | 0/0/0 |
| [actual-owner-conversion-format.log](actual-owner-conversion-format.log) | 0 | 0/0/0 |
| [actual-owner-full-mirror-lifetime.log](actual-owner-full-mirror-lifetime.log) | 0 | 1/0/0 |
| [actual-owner-layered-preflight-format.log](actual-owner-layered-preflight-format.log) | 0 | 0/0/0 |
| [actual-owner-mirror-control-check.log](actual-owner-mirror-control-check.log) | 0 | 0/0/0 |
| [actual-owner-mirror-control-format.log](actual-owner-mirror-control-format.log) | 0 | 0/0/0 |
| [actual-owner-native-cold-and-lifetime.log](actual-owner-native-cold-and-lifetime.log) | 100 | 4/1/0 |
| [actual-owner-prefix-and-quote-format.log](actual-owner-prefix-and-quote-format.log) | 0 | 0/0/0 |
| [actual-owner-runtime-controls-check.log](actual-owner-runtime-controls-check.log) | 0 | 0/0/0 |
| [actual-owner-runtime-controls-format.log](actual-owner-runtime-controls-format.log) | 0 | 0/0/0 |

## Failures and corrections

The first owner compile failed with eight implementation errors: private re-exports (and their dependent visibility errors), a duplicate existing adjacency count method (and dependent ambiguity), and using a non-Copy adjacency policy after moving it. These were this implementation?s errors. Visibility is now restricted to the checkpoint owner, the existing count accessor is reused, and the policy moves after its borrows. The corrected owner and subsequent court checks pass.

The first runtime selection passed Native3 and the Head lifetime control, but failed the newly added `full_materialization_budget_survives_edit_and_last_basis_drop`. Its assertion wrongly assumed Complete?s root-cell Full became the mirror?s next edit lease. Independent review traced the existing mirror?s separately retained Head. The test stimulus was wrong. The corrected public retention-pass stimulus promotes and edits the actual mirror Full; `full_materialization_budget_survives_mirror_edits_until_runtime_drop` observes that owner, with strong payload retention/refund assertions. The first corrected run passed1/0/0. After adding zero-budget denial/retry and completing the layered temporary preflight, the final Native3-plus-Full run passed4/0/0. No existing court?s workload, assertion or policy changed.

The earlier Head lifetime pass remains in the mixed4/1/0 log. The final layered-quote change is exclusive to Full reconstruction; no new Head outcome is claimed after that change.

## Still unrun for this actual-owner source

Host2 cold selections, Store recovery, cold filesystem zero-covered-read, growth, unchanged bounded-tail, final CDC/recovery selections, full Root/Store Clippy, phase line-cap/boundary/context gates, new instrumented split, and integration-with-master proof repetition. The scoped owner seam is unfinished, so these are not represented as green. Formatting and focused compile/runtime results above do not certify funding. Decoder overflow, D13, named digests and BC9002 remain in the ordered queue. No ignored test, query-scale, exhaustive Home oracle or Store scenario suite ran.
