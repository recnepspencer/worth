# Reopen merge qualification

Pinned incoming7d0e5189bc into accepted0f2a6c705c. All validation logs retain
the actual HEAD0f and dirty merge status. Source/configuration equals Git tree
4eebf6e09efd30a6f15f9dc4430a7473cc158901; only evidence artifacts are edited after
that validated index. This is source-equivalence evidence, not relabeling logs
as a later merge HEAD. Exact commands, times and exits are in the full logs;
safe-reopen-validation.json indexes results.

| Lane | Status | Pass/fail/executed ignored | Log |
|---|---|---:|---|
| safe-reopen-root-check | FAIL | 0/0/0 | target/lane-evidence/safe-reopen-root-check.log |
| safe-reopen-root-check-v2 | FAIL | 0/0/0 | target/lane-evidence/safe-reopen-root-check-v2.log |
| safe-reopen-root-check-v3 | PASS | 0/0/0 | target/lane-evidence/safe-reopen-root-check-v3.log |
| safe-reopen-query-check | PASS | 0/0/0 | target/lane-evidence/safe-reopen-query-check.log |
| safe-reopen-store-check | PASS | 0/0/0 | target/lane-evidence/safe-reopen-store-check.log |
| safe-reopen-query-tests | PASS | 10/0/0 | target/lane-evidence/safe-reopen-query-tests.log |
| safe-reopen-root-tests | PASS | 5/0/0 | target/lane-evidence/safe-reopen-root-tests.log |
| safe-reopen-world-unit-tests | PASS | 6/0/0 | target/lane-evidence/safe-reopen-world-unit-tests.log |
| safe-reopen-store-tests | EXPECTED RED | 9/1/0 | target/lane-evidence/safe-reopen-store-tests.log |
| safe-reopen-query-default-release | PASS | 1/0/0 | target/lane-evidence/safe-reopen-query-default-release.log |
| safe-reopen-query-default-continue | PASS | 1/0/0 | target/lane-evidence/safe-reopen-query-default-continue.log |
| safe-reopen-query-default-prepared | PASS | 1/0/0 | target/lane-evidence/safe-reopen-query-default-prepared.log |
| safe-reopen-resolution-format-root | PASS | 0/0/0 | target/lane-evidence/safe-reopen-resolution-format-root.log |
| safe-reopen-resolution-format-query | PASS | 0/0/0 | target/lane-evidence/safe-reopen-resolution-format-query.log |
| safe-reopen-resolution-format-store | PASS | 0/0/0 | target/lane-evidence/safe-reopen-resolution-format-store.log |
| safe-reopen-line-caps | PASS | 0/0/0 | target/lane-evidence/safe-reopen-line-caps.log |
| safe-reopen-query-audit | PASS | 0/0/0 | target/lane-evidence/safe-reopen-query-audit.log |
| safe-reopen-root-audit | PASS | 0/0/0 | target/lane-evidence/safe-reopen-root-audit.log |
| safe-reopen-world-unit-audit | PASS | 0/0/0 | target/lane-evidence/safe-reopen-world-unit-audit.log |
| safe-reopen-store-audit | PASS | 0/0/0 | target/lane-evidence/safe-reopen-store-audit.log |

Root first check failed four raw-array fixture fields after named-digest
cutover; convert those descriptive fields to CanonicalDigestId. Second check
failed the Server adapter exhaustive map; retain distinct ReleasedOriginal and
ProductUnpublished wire meanings. Third full Root check is green. Query and
Store full workspace checks are green. No weakened assertions or ignored tests.

The Store red is exactly the coordinator-excluded post-release ProductBasisStale
HIGH, before partial-prefix admission. The D8 public service escape is also
carried unchanged as instructed; it is a source finding, not an executed court.
No new CLEANUP red was observed in these focused selections.

No boundary/context/strict/broad Tier-2 claim at this intermediate merge. Those
run on the master-composed source. No owner WIP, Lifecycle merge or push.

The first broad whitespace check flagged two historical review-note EOF blanks
and a literal blank context line in a proposed patch. Trimmed the note EOFs and
reframed that draft patch with the exact same19added/0deleted lines; git apply
--numstat parses it. Full cached whitespace check v2 passes. No code or assertion
changed. Logs safe-reopen-whitespace{,-v2}.log retain both outcomes.
