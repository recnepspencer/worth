# P-A takeover progress (Fable 5.2)

Resumed 2026-11-10 after the usage-limit stop. HEAD still f9f15bf8c8, NOTHING
COMMITTED YET. Usage is the scarcest resource: no polling, tail/grep logs only.

Counts so far: clippy 4 (batch1..4; batch2 first attempt was a cached no-op),
focused builds 2 (court, native lib), test runs 3 (public 41/0/0; native 20/1/0;
control 0/2/0), tool runs: boundary 2, agent-context 1, line caps 1.

## COMMITS
- 780fe3ecfe test(native): port merge execution courts to public owners
  (slice A; independent review APPROVE, claude-review-A.md; 2 config/spec
  minors fixed before commit).
  Deferred MINOR follow-ups from that review (need a compile round):
  (a) rename `merge_commit_context_rejects_mismatched_parent_branch_metadata`
      in merge_unknown_source_admission.rs to its real claim, or retire it if
      merge_basis_journeys.rs:145 already covers it;
  (b) expose entity/relation counts from VisibleTruthSummary and delete
      `visible_named_counts` in merge_execution_certification.rs;
  (c) move `prepare_feature_merge` from merge_prepared_verification.rs to
      merge_feature_execution.rs;
  (d) tidy ledgers; narrow two `pub(super)`; "counters" still in two test names.

## REVIEW OF DC AND B1 (claude-review-DC-B1.md): both REJECT, one MAJOR each
- DC-1: kind-commitment preimage oracle dropped -> FIXED by new owner test
  storage/residency/region_publication/kind_commitment_preimage_tests.rs.
- B1-1: four Memory-only roots wrongly moved (canonical_commit_artifact/refusals.rs
  x3, descriptor_continuity semantics-version test) -> RESTORED in place; spec
  entries, baseline row and ledgers corrected (108 roots moved now: 90 C4... see
  claude-durability-moved-roots.json).
- B1 minor: empty impl in lifecycle_order_control.rs removed; the remaining
  cfg(test) lifecycle control is now unarmable (follow-up: remove the whole
  control or keep it whole).
- Review-fix round logs: claude-reviewfix-{fmt,clippy,native}.log. After it is
  green: rerun claude_slice_files.py, resume the reviewer for a re-verdict, then
  `python target/lane-evidence/claude_stage_slice.py DC` + `git commit -F
  target/lane-evidence/claude-commit-DC.txt`, same for B1.

## STATE 2026-11-10 ~08:45 UTC
- Clippy all-targets clean (claude-batch4-clippy.log): lib 0 warnings; 2
  test-build warnings in tests/support/durable_medium.rs (TestRecoverySourceRefusal
  payload), left until the adapter is deleted.
- Public court 41/0/0 (claude-merge-execution-public.log). Native 20/1
  (claude-batch-native.log): the 1 red is PRE-EXISTING (private CountedSource
  fixture lacks funded reads; control claude-counted-source-control.log 0/2 on
  untouched siblings). Reported in the shared notes folder
  (p-a-finding-counted-source-red-tests.md). ~19 residency owner test files may
  be red at HEAD for this reason; not in my closure.
- Boundary 12 findings (claude-batch-boundary2.log): only runtime_attachment.rs
  is mine. Seams added: certification-construction-callers.txt line for
  merge_descriptive_touches.rs (native-home-complete-checkpoint; court already
  has approved callers); baseline: 7c_phase_g/mod.rs row takes RelationalRuntime
  root from deleted recovery_certification.rs row (net shrink).
- Slice A is STAGED in the index (claude_stage_slice.py A; names in
  claude-staged-A-names.txt). Per-slice shared-file states in claude-shared/{A,DC,B1}/.
- Complete/ACK named lane GREEN on the dirty tree: claude-complete-ack-store.log
  (selected_checkpoint_survives_observer_drop_online_history_and_mutation_before_ack_recovery 1/0/0)
  and claude-complete-ack-query.log
  (interrupted_owner_ack_resumes_original_attempt_and_reopens_unacknowledged_selection 1/0/0).
  Rerun after commit for a clean-HEAD header: python target/lane-evidence/claude_lane_run.py
  <label> python target/lane-evidence/claude_lane_tests.py <store|query> <label> <cases.json>.
- Commit messages ready: claude-commit-{A,DC,B1}.txt (git commit -F).
- Reviews running: claude-review-A.md, claude-review-DC-B1.md.
- Next: fix findings -> commit A -> stage+commit DC -> stage+commit B1
  (python target/lane-evidence/claude_stage_slice.py DC / B1), each with
  trailer `Co-Authored-By: Claude Fable 5.2 <noreply@anthropic.com>`.

## Tree now holds three uncommitted slices (commit separately, in this order)

### A. Merge-execution port (65 roots) - source complete, awaiting compile
- Court (crates/worth-court-relational-certification/.../durable_publication/):
  4 helper modules (merge_execution_{observation,recovery,schema,authoring}),
  merge_feature_execution, 23 test files, 41 public tests
  (claude-merge-execution-public-cases.json). mod.rs lines added.
- Native: 39 originals retired (claude-merge-execution-retirement.json), 15
  C2/C4 roots moved by previous owner, orphan fixtures deleted
  (claude-merge-execution-fixture-deletions.txt): dirs milestone_7c_phase_e,
  milestone_7d_phase_b, files milestone_7d_phase_c.rs, milestone_7d_phase_e.rs,
  7c_phase_g/{artifacts,certification_cases,recovery_certification}.rs.
  7c_phase_g/mod.rs rewritten as owner unit
  `executed_merge_history_rebuilds_directly_from_its_audit_recovery_plan`
  (Memory, private rebuild_runtime_from_plan). 7d_phase_d/mod.rs trimmed to the
  registration fixture. Owner cases: claude-merge-execution-owner-cases.json (11).
- Spec: 6 C2 instrumentation remainders + previous owner's 15 entries.
- Merge seam: facade.rs diagnostics re-export of `RelationalDiagnosticValue`.
- Reviewer points: merge_history_shape / merge_execution_certification use
  `actual_basis` instead of cfg(test) LCA/inspect_merge; merge_descriptive_touches
  proves Complete generation preservation by public restore; unknown-source test
  asserts binding denial `UnknownBranch`; readiness test asserts Execute over
  public `artifact().lowered_plan.records` (exact copied array stays owner).

### DC. 32 dead-code warnings - source complete, awaiting compile
Notes: landing2a-dead-code-deletion-notes.md (29 modified, 15 deleted, 2 created).
8 tests removed: 4 retired with implementation; 4 C2 candidates
(cell/resident_full/tests.rs x2, region_publication/full/tests.rs x2) - need
spec C2 entries + baseline rows 2 (lines for those two test files).
Flag for review: pub serde variant `RecordAllocationDenial::ReplayRecordMismatch`
deleted (only producer was the deleted cursor).

### B1. Durability move-out - applied, orphan cleanup NOT done
112 pure C2/C4 adapter-causal roots removed from 41 files
(claude-durability-moved-roots.json), spec entries added. Orphaned fixtures in
tests/durability must be cleaned (compiler warnings from batch1 clippy tell).
Map of all 215 rows: claude-durability-contract-claims.json.

## Commit mechanics (shared files)
plans/database-foundation/milestone-3.md and
tools/boundary-check/config/src-court-baseline.json carry hunks of several
slices. Per slice, regenerate the file state for that slice (HEAD + that
slice's entries), `git add` it, commit, then restore the full version.

## After these three
- Slice B2: ~75 durability roots with HAPPY parts + 24 owner-extension roots.
- Slice C: 154 other roots (landing2a-other-root-classification.json).
- Delete runtime_attachment.rs + durable helpers in tests/support.
- Complete/ACK named lane (two public flow courts in
  landing2a-complete-ack-boundary-review.md).
- Boundary check zero findings in my files.

## Tooling (target/lane-evidence/)
claude_lane_run.py <label> <cmd...>; claude_lane_tests.py <root|native> <label> <cases.json>;
claude_retire_merge_execution.py; claude_move_out_durability_roots.py;
claude_orphan_fns.py (untested).
