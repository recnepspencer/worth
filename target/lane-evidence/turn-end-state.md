# P-A turn-end state: D13 IN PROGRESS

Last accepted FUNDING MERGED: bd3ac03d2f. Current source/evidence HEAD:
eb875cdf0433eebd4335e88551c219a6e7eeb24e, plus owned uncommitted D13 work.
The next commit records documents/evidence only. No D13 implementation is landed,
no source is discarded or parked, and no push/merge is performed this turn.

## Green evidence (scoped, dirty-tree evidence)

- D13 Native/World initial controls14/0/0; carried-original controls3/0/0.
- Latest accounting/cold materialization/recapture correction controls9/0/0:
  d13-final-corrections-root.log.
- Both required historical terminal-and-close cases now pass. Fixed passed in
  d13-consumer-exhaustive-retry.log (that packet1/1/0); discovered passed alone
  d13-discovered-replay-wrapper-correction.log1/0/0.
- Latest complete packet d13-head-work-proof-batch.log38/16/0:
  Root21/4, consumer7/2, Query9/10, Store1/0. These are not disjoint from later
  corrections and must NOT be summed into a supposed final suite count.
- Query direct same_directory_reopens_reads_and_commits_without_replaying_initializer_or_old_key
  passed with RUST_MIN_STACK absent in d13-head-work-query.log.
- Last completed Root Clippy, Query check and Native/World Clippy passed before
  later source corrections. They are not claimed final qualification.
- d13-static-reach.log:54 frozen caller paths, no old interim identifiers.

## Open items and ownership

1. HAPPY, P-A D13: finish the image-consuming codec schema reader, one original
   recovery scope, exact-image guard and original-DATA installation.9 named Query
   failures share this cause. Exact proposed design: d13-codec-source-schema-owner-cutover.md.
   Standing ownership check returns1 for codec_transition.rs and its source_schema.rs
   versus Lifecycle; its worktree is clean for both. Exact assignment question is
   pending. Reader remains unchanged; no fake inline roots or replay fallback.
2. HAPPY, P-A investigation: exact retirement census equality remains red.
   Current root11 and mirror have the expected4E/4R removals. Actual retained
   roots4..9 preserve those identities in the required inspection union.
   No actual holder was established. Do not weaken the court or add speculative
   maintenance. d13-retirement-census-attribution.md and census JSON/log record it.
3. HAPPY, P-A queue/qualification: three private storage providers require actual
   external courts/value decomposition. Do not extend CountedSource or add an
   observer. Exact unreachable claims and closest public flows are in
   d13-private-storage-qualification-boundary.md. Cases: head_prefixes_rebuild_owned_history_after_endpoint_edits_and_generation_reuse;
   adopting_a_different_head_carries_its_owned_historical_projection_with_the_record;
   cancellation_during_readmission_preserves_the_request_and_does_not_damage_the_cell.
4. HAPPY, P-A named digests and six Native BC9002 files/root consumers remain
   queued, not completed. Root assertion-equivalence design/review not yet done.
   Other owners' boundary findings are not assigned to P-A and are not blockers
   claimed here. No other-owner caller entries were applied in this continuation.
5. HAPPY, P-A final qualification: formatting, phase line-cap/boundary/context
   commands and final fresh approving review are unrun/incomplete for D13. Do not
   reuse funding certification as D13 certification. No full suite was run.

## Review and validation cost

Fresh independent D13 review: REJECT, two P1 findings (incomplete codec cutover;
incomplete runtime qualification). Both accepted and still open. No additional
supported ordinary-path authority defect. Retirement attribution reviewer ran no
builds/tests and did not recommend a speculative fix.

Cumulative this turn:9 cargo check/Clippy calls;20 focused prebuild invocations
(including compiler-failed attempts);16 selected nextest runs plus1 direct test
command. One separate selection-only refusal, and one diagnostic patch assertion
failed before Cargo. No ignored test, query-scale, exhaustive oracle or Store
scenario suite ran. Budget exceptions and all failures are retained in progress.

The full lane ledger is d13-validation-index.json (each exact command, times,
exit and pass/fail/ignored counts) and d13-status-report.md. Source fingerprints:
d13-source-status-record.json. Temporary census source was restored byte-for-byte;
its restoration assertion passed and inspection.rs is clean. No allowance raised.
