# P-A active continuation - 2026-11-10

Branch codex/m3-p3-reopen; inherited reviewed HEAD e4d3b01c8d.
Codex is the sole writer again. The completed handoff below is preserved.

Current B2 checkpoint subfamily: 16 public tests and the Native builder-configuration
unit pass. Exact C2/C4 portions are listed in the spec. Final scoped gates and
independent review are being recorded before its coherent commit. Detailed
plan and evidence: pa-resume-b2-checkpoint-plan.md and pa-b2-checkpoint-ledger.md.
The unmounted association_checkpoint_journeys.rs starts the next B2 family; it
is not part of this checkpoint commit and has not been compiled or qualified.

A renewed Codex login interrupted the first Native attempt at metadata only.
Reuse the public 16/0/0 log; the resumed Native result is 1/0/0. No green input
was rebuilt because of that interruption. No merge is in progress.

## Commits this takeover (base f9f15bf8c8), each with an approving independent review

| sha | subject | content |
|---|---|---|
| 780fe3ecfe | test(native): port merge execution courts to public owners | Slice A. 65 merge-history roots: 41 public court tests in durable_publication/merge_*, 15 roots moved C4/C2 (previous owner's script), 6 C2 instrumentation remainders, owner units kept on Memory, orphan fixtures and cfg(test) merge hooks deleted. Review claude-review-A.md APPROVE. |
| 40cafd5eed | refactor(native): delete the orphan original-intent executor and Full constructors | EVIDENCE ONLY (staging defect): adds landing2a-dead-code-deletion-notes.md. |
| ff723988f4 | test(native): move cleanup-class durability roots out to C4 and C2 | EVIDENCE ONLY (staging defect): adds claude-durability-contract-claims.json and claude-durability-moved-roots.json. |
| b0ac2e133e | ... Full constructors (sources) | Slice DC, the real change: the 32 worth-relational library dead-code diagnostics deleted (29 files modified, 15 deleted, 2 renamed), 8 tests removed (4 retired with implementation, 4 exact-bill tests listed under C2), preimage oracle kept on the live producer. Review claude-review-DC-B1.md REJECT -> fixed -> APPROVE. |
| e4d3b01c8d | ... C4 and C2 (sources) | Slice B1, the real change: 108 pure C4 (89) / C2 (19) adapter-causal durability contract/medium roots moved out with restore refs; orphan fixtures, 21 emptied files and 10 cfg(test) control hooks removed. Same review: REJECT -> fixed -> APPROVE. |

Staging defect: claude_stage_slice.py fed CRLF path lists to `git update-index`,
which ignored them; only the `git add -f` evidence files were committed. History
is never rewritten here, so the sources followed as "(sources)" commits. The
script is fixed (bytes stdin) and claude_verify_staged.py checks the shared files.

## Gate status

1. Adapter closure: NOT closed. `tests/support/durable_medium/runtime_attachment.rs`
   still exists (the one owned BC9002 finding). Done: 65 history roots (A), 108
   pure durability move-outs (B1). Remaining:
   - B2: the durability contract/medium roots with a HAPPY or owner part
     (about 75 main rows) and the 24 owner-extension roots. Map:
     claude-durability-contract-claims.json (215 rows; `classification`,
     `adapter_causal`, `section`), text
     landing2a-durability-contract-claim-disposition.md. The four rows wrongly
     moved and restored (refusals.rs x3, descriptor semantics-version) are
     Memory-only and stay.
   - C: 154 other roots, landing2a-other-root-classification.json
     (34 HAPPY, 32 UNCHANGED, 26 C2, 21 OWNER, 19 HAPPY+C2, 11 C4, ...).
   - Then delete runtime_attachment.rs, the local test medium
     (tests/support/durable_medium*), the mixed helpers in
     tests/support/durability.rs and tests/support/runtime.rs
     (durable_runtime_with_test_schema*, checkpoint_and_recover_with, recover_with,
     recovery_plan_for, checkpoint_durably, ...), and the
     `in_crate_test_medium` hook on the medium trait.
2. 32 Native production warnings: DONE (b0ac2e133e). clippy all-targets on
   worth-relational + its court: library 0 warnings.
3. Complete/ACK named lane: GREEN, but logged on the pre-commit dirty tree
   (source identical to HEAD): claude-complete-ack-store.log
   (selected_checkpoint_survives_observer_drop_online_history_and_mutation_before_ack_recovery, 1/0/0)
   and claude-complete-ack-query.log
   (interrupted_owner_ack_resumes_original_attempt_and_reopens_unacknowledged_selection, 1/0/0).
   For a clean-HEAD header rerun:
   python target/lane-evidence/claude_lane_run.py claude-complete-ack-store python target/lane-evidence/claude_lane_tests.py store claude-complete-ack-store target/lane-evidence/claude-complete-ack-store-cases.json
   (and the same with `query`).
4. Allocation-refusal court: moved to C2 by the previous owner (b736be6f47).
Boundary: 12 findings (claude-batch-boundary2.log, pre-commit tree): 1 owned
(runtime_attachment.rs), 11 routed elsewhere. Zero-in-my-files needs item 1.

## Known open points

- Two test-build warnings remain (`field 0 is never read`,
  tests/support/durable_medium.rs:56-57). Reviewer accepted them only if the
  adapter retirement lands in the same landing; otherwise drop the two payloads.
- lifecycle_order_control.rs: its cfg(test) control is now unarmable residue;
  remove the whole control (module, field, call sites) or restore it whole.
- Slice A review MINOR follow-ups (source, need a compile round): rename or
  retire `merge_commit_context_rejects_mismatched_parent_branch_metadata`
  (merge_unknown_source_admission.rs; maybe duplicate of merge_basis_journeys.rs:145);
  expose counts from VisibleTruthSummary and delete `visible_named_counts`
  (merge_execution_certification.rs); move `prepare_feature_merge` to
  merge_feature_execution.rs; narrow two `pub(super)`; two test names still say
  "counters".
- ASSIGNED RED (coordinator ruling): Native residency owner tests on the private
  `CountedSource` fixture (storage/residency/region_fixture.rs) fail at
  `region_load::load_head` (default `retain_members_for_request` refuses).
  `region_with_history()` has 50 call sites in 19 files. Evidence:
  claude-batch-native.log, claude-counted-source-control.log. Reported in the
  shared notes folder (p-a-finding-counted-source-red-tests.md). It will fail
  Landing 2a's Tier-3 lane. Fix the fixture's funded custody as its own reviewed commit.
- Merge seams: facade.rs re-exports `RelationalDiagnosticValue`;
  certification-construction-callers.txt adds merge_descriptive_touches.rs under
  native-home-complete-checkpoint; src-court-baseline.json only shrinks
  (milestone_7c_phase_g/mod.rs took the RelationalRuntime root of the deleted
  recovery_certification.rs row); `RecordAllocationDenial::ReplayRecordMismatch`
  (pub, serde, last variant, name-tagged) deleted with its only producer.

## Validation counts (whole takeover)

clippy 5 rounds (batch1-4 + review-fix; one extra cached no-op), focused builds 5
(court, native x2, store court, query court), test runs 6 (public 41/0/0;
native 20/1/0 with the pre-existing red; control 0/2/0; store 1/0/0;
query 1/0/0; review-fix native 6/0/0), tool runs: boundary 2, agent-context 1,
line caps 1. No ignored body, query-scale, oracle or scenario suite ran. No push.

## Tooling in target/lane-evidence/ (untracked, on disk)

- claude_lane_run.py <label> <cmd...>: runs a lane, writes <label>.log with header and exit code.
- claude_lane_tests.py <root|native|query|store> <label> <cases.json>: owned prebuild + selected nextest run.
- claude_warnings.py <log> [--json out]: located warnings/errors grouped by file.
- claude_apply_dead_items.py <warnings.json> <ledger.json> [--apply]: deletes
  reported dead items/imports. Only feed it warnings from a build with ZERO
  errors (warnings from a broken parse are wrong), and remove dangling
  `#[path]` attributes after it unmounts emptied files.
- claude_move_out_durability_roots.py, claude_retire_merge_execution.py: the
  two move/retire scripts already applied (templates for B2/C).
- claude_slice_files.py, claude_shared_states.py, claude_stage_slice.py,
  claude_verify_staged.py: per-slice commit mechanics for the two shared files
  (milestone-3.md, src-court-baseline.json).
- Port-agent brief pattern that worked (four parallel subagents, no compile,
  disjoint files, then one clippy round): see the slice A helper modules
  durable_publication/merge_execution_{observation,recovery,schema,authoring}.rs.
