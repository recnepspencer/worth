# Integration (Fable 5.2 takeover) progress

Brief: `C:/programming/worth worktees/orchestration/m3/prompts/fable_integration_takeover.md`.
Predecessor notes: `integration-progress.md` (ends "Current short handoff 18:05"), `master2-merge-families.md`,
`master2-store-runtime-families.md`. Re-read this file after any context compaction.

## State at takeover (2026-11-09 ~22:20 local)

- HEAD `171d380829` on `codex/m3-host-home`; source clean. Dirty at takeover: two evidence notes
  (`integration-progress.md`, `master2-compile-handoff.md`) plus owned runner telemetry under
  `scripts/test_cost/slots/{root,bank}` (388 D / 488 ?? / index.json M). Telemetry is runner output, not my edit;
  left untouched and NOT part of the repair commit.
- C: free 145.7 GB at takeover. No Cargo of mine running.
- Store continuation driver (session 30748) died with the old session; `master2-store-runtime-families.md`
  (18:58) holds the live classification; final Store counts still to be read from raw logs.

## Repair batch (uncommitted, applied to the working tree)

Applied with `git apply`, all exit 0:

| Patch (target/lane-evidence) | sha256 prefix | files |
|---|---|---|
| master2-ready-runtime-batch.patch | e67c0992a2cf7d05 | 40 |
| master2-query-pdb-link-proposal-v2.patch | 497f5f6f66775d1e | 3 |
| master2-fixture-build-config-proposal.patch | 20f7320787249011 | 2 |
| master2-store-diagnostic-profile-proposal.patch | b7747103a3be84e8 | 1 |
| master2-store-tooling-compiler-proposal-v2.patch | 2b020030d162e157 | 4 |
| fable-h8-stored-text-readmission-worktree.patch (H8's uncommitted fix, copied verbatim) | ea2f0d40644e9eb3 | 3 |

- Byte identity verified (git blob ids) against H8 commit `c76b0502ce` for all 6 files of semantic-v2 + Store
  profile, and against H8's working tree for the 3 stored-text readmission files.
- MERGE SEAM: the stored-text fix is still uncommitted on `fable/m3-h8-dispatch`. If H8 commits different bytes,
  reconcile at the H8 batch merge (files: `home_publication/readmission.rs`, `idempotency/home_readmission.rs`,
  `branch_activation_history/source.rs` under worth-query-execution primary_graph).

- `tools/worth-query-release/Cargo.lock`: `cargo update -p cc --precise 1.9.0 --offline` (Cargo-generated; also adds
  the merged `im` closure). Master 4405a563d2 itself carries the 1.11.0 mismatch.
- NEW HAPPY Native fix (classification by read-only subagent, ~75% confidence from source):
  `crates/worth-relational/src/durability/authority/runtime_rebuild/index_catalog.rs::readmit` now calls
  `index_authority().recover_installed_index_routing(execution)` after a non-empty `readmit_catalog_delta`.
  Cause: master clears installed routing on every definition insert; live `register()` reinstalls it, journal-delta
  readmission did not, so journal-only replay derived Unavailable evidence vs original Complete
  ("recovered authored index writes disagree with original source", authored_index_writes.rs:71).
  Confirming courts (both track H, Host): `journal_only_killed_home_reopens_original_inputs_and_commits_without_initializing`,
  `close_checkpoint_exhaustion_reports_failure_and_preserves_journal_reopen` — must be green in Query T2.
- Store census (pre-repair, 171d): run was CUT OFF at 3327/4506 when the old session died (493 failed, >=2834 passed,
  16 binaries/997 cases never started, 3 bounded supplementary cases never ran). Families and names:
  `master2-store-final-failed-names.json`. 4 HAPPY cases have unprinted causes (ordinals 1132, 1164, 1166, 1168=D18).

- Ruling 1 (D13) APPLIED (subagent, uncompiled): 31 tests held out (not just six files — `coupled/history.rs`
  and `owned_outputs/world.rs` are shared reopen fixtures; every dependent cold court left with them).
  24 files deleted (12 authored_ownership, 8 owned_outputs, whole crate
  `worth-query-court-decl-host-owned-codec-transition`, testkit `source_owned_transition.rs`); partial removals in
  `cold_owned_revision.rs`, `current_fence.rs`, `coupled/multi.rs`, `owned_outputs/world.rs`,
  `predecessor/courts.rs`; registrations: `court-frontier-splits.toml` (codec split removed),
  `control-callers.txt` (11 stale rows removed), Query `Cargo.lock` (codec package block removed).
  Interim-door references afterwards: 5 owner + 53 frozen callers + 3 boundary-check sources, zero others.
  Spec "Moved courts" entries written in `plans/database-foundation/milestone-3.md` (H6 and H9 groups).
  Known leftovers: unused dev-deps in authored-ownership Cargo.toml (rmp-serde, sha2, serde, worth-execution);
  `plans/WORTH-query/authored-retirement-current-ownership.md:132` still names the codec court; `_for_test`
  production doors now callerless until restore.
- NEXT SLICE (not started): Native-owned courts for D13 rows 1/3/4/5 subclaims:
  row1 co-anchor closure -> `worth-court-relational-certification/.../relational/authored_retirement/` beside
  `corruption_subjects.rs` (reuse `registration.rs`, `restoration.rs:17-50`); row3 duplicate coupled edge -> same dir,
  group check `history/data/codec_retirement_closure/source_scope_closure/coupled.rs`; row4 current roster ->
  `durability/data/native_checkpoint/roster_corruption.rs` + `publication/ownership/cold_checkpoint_source.rs`
  following `cold_pairing_court.rs`; row5 codec retirement -> `durability/authority/recovery/codec_transition/tests/`.

- Landing 2a prep DRAFTS written by read-only subagent (no evidence yet, every Supported row "pending"):
  `landing2a-coherence.md` (38 rows: 23 Supported, 11 Refused, 4 PARTIAL — workflow families on a Directory Home
  after P-C 6ea7a7126e, AftermathCausality, program adoption, reopen with pending transition) and
  `landing2a-cleanup-moveout-draft.md` (8 tests/4 files ready; the rest need re-observation after the repair).
  Flags to verify: refusal type is `WorthQueryDurableStateAdmissionDenial::Unsupported` (no `UnsupportedDurableFamily`
  type); support table disagrees with code for 4 families; Store court
  `latest_native_owner_rows_refuse_their_actual_family_kind_and_record_before_service` predicted stale.

- Ruling 2 APPLIED (subagent): `RELATIONAL_SHARING_INSPECTION_VERSION` 6 -> 7 (single version, no compat read);
  per-Extra nested extent now `CanonicalAspectStateMap::retained_storage_bytes()` (a checked upper bound;
  the old `owned_allocation_capacity_bytes` was neither exact nor a bound); typed
  `StorageAllocationExtent::{Checked, Overflow}` refused at already-fallible owners
  (`PartitionContentDigestError::AllocationAccountingOverflow`, new
  `RelationalBranchSharingInspectionDenial::AuthoritativeByteBoundOverflow`); `unique_physical_*` names kept with
  truthful docs. No court deleted (none asserted cross-Extra dedup); `check_delta` kept (same-fold identity).
  New tests: `canonical_map/shared_holder_bound_tests.rs`, `values/tests/shared_state_bound.rs`,
  `sharing_accounting/byte_bound_overflow_tests.rs`, inline test in `partition_allocation_walk.rs`.
  Spec C2 "Moved courts" claim entry written. Still saturating (documented, C2): full region inventories at
  initial capture, reclamation counters, recorded cost counters.
- Compile round fable1 (Root only reached): exit 101, 1 diagnostic: E0624 private
  `RelationalRuntime::current_version_id` used by the semantic-v2 court
  `content/proposed_state_invariants.rs:92`. Fixed in the court: predecessor now read from the proposal's own
  basis (`basis.observation().version_id()`), nearest working analogue `basis_observation/mod.rs:19`.
  MERGE SEAM: this file is no longer byte-identical to H8 `c76b0502ce` (2-line court-only difference; H8 will hit
  the same E0624 when it compiles that court crate).
- Compile round fable2 GREEN: root 0 / query 0 / store 0, zero diagnostics each
  (`fable2-check-driver.log`, `master2-fable2-{root,query,store}-check.log`, ended 2026-11-10T06:42Z).
- Lanes chain launched in background (`fable_repair_lanes.ps1`, serial, one Cargo at a time):
  root-focused (`fable_focused_runtime.py root`, out `fable-repair-root-focused/`), query-tier2
  (`master2-tier2-permitted.py --execute --only query`, out `fable-repair-tier2/`), bank-focused, store-focused.
  Stage logs: `fable-repair-<stage>.log` (header + EXIT). `fable_focused_runtime.py` = reviewed
  `master2_focused_runtime.py` with the changed-source set taken from the live batch (git diff HEAD + untracked).
  DO NOT relaunch a stage whose out dir exists; read its log/JSON instead.
- Review packages ready: `fable-repair-batch.patch` (whole batch, 116 tracked files + 8 new, sha256 69f974b2…)
  and `fable-repair-unreviewed-delta.patch` (only what is NOT in the pre-approved patches: Ruling 1, Ruling 2,
  index_catalog fix, court E0624 fix, cc lock; also re-shows the 3 PDB-v2 files; 74 files +471/−3198,
  sha256 0acd4d13…). Regenerate both if any source changes before review.
## Lanes pass 1 (2026-11-10 07:33Z–09:40Z)

- root-focused FINISHED, exit 1: 2306 run, **2215 passed, 91 failed**. 2180 of 2272 prior Root failures now
  pass. The 91 are mostly second-layer causes the seed defect was masking. Ledger (7 families, none fixed, none
  claimed by the commit): `fable-repair-root-families.md`; per-test list `fable-repair-root-focused-families.txt`.
  The Ruling-2 new tests and the changed court `proposed_state_invariants` ran and passed.
- query-tier2 DID NOT FINISH: build died with `rustc-LLVM ERROR: IO failure on output stream: no space on device`
  (C: reached 0; `workspaces/worth-query/target/debug/incremental` had grown to 143 GB). Not a code error.
  Evidence kept: `fable-repair-query-tier2.log`, `fable-repair-tier2/`.
- bank-focused and store-focused REFUSED by the disk floor (never started).

## Lanes pass 2 (launched 2026-11-10, background, `fable_repair_lanes_b.ps1`)

- Deleted `workspaces/worth-query/target/debug/incremental` (regenerable, own worktree; `fable-repair-disk.log`):
  C free 108 -> 229 GB.
- Stages, serial: `query-tier2-b` (out `fable-repair-tier2-b/`, log `fable-repair-query-tier2-b.log`),
  `store-focused`, `bank-focused` (logs `fable-repair-<stage>.log`, out `fable-repair-<stage>/`).
  DO NOT relaunch a stage whose out dir exists; read its log/JSON instead.
- Read results with the scratch scripts' logic (failed names + panic site/message per `run-*.log`); the mixed
  utf8/utf16 stage logs need NULs stripped before grep.

- query-tier2-b FINISHED: **7633 passed, 107 failed** (`fable-repair-query-families.md`; 64 are index-after-reopen,
  P-C `2dce9c178c` expected to cure). Both Host journal-replay courts PASSED (index_catalog fix confirmed).
  One test ended by hand (pid 3432, 46 CPU min): `authority_cause::tests::authority_evidence_exposes_request_
  resources_by_value_through_the_commit_door` = a 9402-probe sweep at probe 5800, still progressing, not a spin;
  came with master, crate untouched by the batch -> named follow-up (Q8).
- store-focused then bank-focused were running when the repair commit was made (source unchanged by the commit).

## Static gates (see `fable-repair-static-gates.md`)

- boundary-check RED: 4 findings INTRODUCED by pre-approved patches (PDB v2 build.rs in the authored-ownership
  court; 3 new/grown source-test roots from e67c0992) + pre-existing BC9002/BC7014 debt. Follow-ups, not fixed.
- agent-context: worth-query-host stale (pre-existing) -> regenerated by the tool, committed.

- `scripts/ci/check_lockfile_alignment.py`: PASS (18 lockfiles agree).
- Line cap: none of the 90 changed/added Rust files exceeds 400 lines (counted directly).
- NOT yet run (they build with Cargo; run after the lanes, one Cargo at a time): boundary-check, agent-context.

## Review

- ONE independent reviewer (Opus subagent, read-only) on `fable-repair-unreviewed-delta.patch` with brief Rulings
  1/2 + Rules, milestone-3 "Moved courts", AGENTS.md. **VERDICT: APPROVE**, no BLOCKING. Three SHOULD-FIX:
  (1) spec said "re-add control-callers rows on restore" -> FIXED in milestone-3.md (lists never grow; fresh
  ruling needed). (3) C2 entry over-claimed -> FIXED (says F4's 24 courts are pending classification).
  (2) OPEN, not fixed: full-inventory totals still saturate (`record_arena/arena/allocation.rs:37`,
  `overlay/partition.rs:89-97`, `branch/root.rs:304` -> `fork_materialized_authoritative_bytes`); a total
  saturated at u64::MAX would enter the sharing inspection's logical lane through `checked_add` as Ok (an
  under-count shown as a bound). Practically unreachable (a single overflowing carrier already refuses in the
  unique walk). Not small (return types ripple) and Query T2 was building, so it is a NAMED FOLLOW-UP, reported
  to the coordinator; smallest sound fix: refuse a region/fork total equal to u64::MAX at the two admission
  points with the existing typed overflow denials, or carry `StorageAllocationExtent::Overflow` through the
  inventory visitors. Spec C2 entry states it is open until the coordinator accepts it as C2.
  Reviewer also confirmed: all 31 deleted tests listed, all 29 restore paths exist at 171d380829, no dangling
  references, caller lists only shrink, index_catalog fix correct and sufficient on its path.

## Commits (2026-11-10)

- `54b4fa230d` repair batch (reviewed APPROVE); sha line sent to the coordinator.
- `2610c94ef3` chore: runner telemetry under `scripts/test_cost/slots/**` (data only).
- Evidence commit after it: Store/Bank ledger `fable-repair-store-bank-families.md` (Bank GREEN 324/324;
  Store 52 red in families S1-S10), classifications `fable-classifications-2026-11-10.md` (authored_index_writes
  replay = HAPPY fixed in 54b4fa230d; 60 s close maintenance deadline = C2 with an H court rule), F1 root cause
  (FIXTURE: court cold sources lack the mandatory owned-read protocol) in `fable-repair-root-families.md`.
- The coordinator lifted the bank-only rule: continue without ending the turn.

## Merge batch 1 state (2026-11-10)

Commits: `c7c2ade607` P-C 2dce9c178c; `8283728aff` H8 1ce3832b78 (Store: P-C's `query_journal` is the one
transport, `query_dispatch` rebuilt over it, `journal_frame_publication` dropped; Query custody takes
obligations+material+dispatch; `DispatchOutbox|DispatchAttempts => Supported(8)`); `32341309d8` P-A e4d3b01c8d
(`fable-merge-pa-resolution.md`; Full region tests kept on the live producer, resident_full exact-charge courts
C2); `4d05497ab0` Windows linker wrapper (`scripts/windows_linker.cmd`, court build.rs removed). P-C tip
`765e8ccdb0` (partial-surface gates + lane linker refusal) HELD for the next batch.
Lanes: `fable_batch_lanes.ps1 -Stages check|root-t2|query-t2|fixture-t2|store-t2|bank-t2 -Tag batch1` (clears
the machine's CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER override for its own process). Pending: compile check
-> fixes -> one Tier 2 -> one reviewer over the merge resolutions + fixes + linker commit.

## Next: merge batch (coordinator, 2026-11-10)

One batch, one Tier 2; H5 NOT landed, do not merge it. In order, a merge commit each:
1. `codex/m3-p3-lifecycle` (P-C tip `2dce9c178c`, base `171d380829`): checkpoint index selections.
2. `fable/m3-h8-dispatch` (tip `1ce3832b78`, includes `2d6822abbe`; base `171d380829`): take H8's bytes for the
   3 stored-text files (supersedes the copied patch in 54b4fa230d).
3. `codex/m3-p3-reopen` (P-A tip `e4d3b01c8d`: `780fe3ecfe`, `b0ac2e133e`; base `2a51552fc5`, old: expect
   conflicts).
Then: disk check, one compile per workspace, one Tier 2 via `affected_lanes.py` against `54b4fa230d`
(DO NOT relaunch a stage whose out dir exists), one reviewer on the conflict resolutions, commit, sha line.
Then reclassify Q1/F3/S1/S2/S3 against the new results, then F3, F6, F1, Landing 2a prep.

## Follow-ups after the commit (not started)

- F3 (index routing after checkpoint/catalog restore, 10 Root courts) and F6 (server durable Query retry,
  5 courts): likely HAPPY, see `fable-repair-root-families.md`. F1 (27 cold-capture courts) unclassified.
- Native-owned courts for D13 rows 1/3/4/5 (plan above). Root Tier 2 after the repair has never run.
- Incoming seams at the next merge: `orchestration/m3/prompts/integrate_pending.md` (H5 merge `f15ff306ec`);
  take H8 `2d6822abbe` (stored-text reopen repair) identically — it supersedes the copied working-tree patch
  here, reconcile the 3 files; P-C checkpoint-binding fix sha to follow.
- Landing 2a prep (do not start the gate before H5 LANDED): refine `landing2a-coherence.md` with post-repair
  evidence; gate the 4 PARTIAL rows to typed refusals; finish the cleanup-court move-out list with restore refs
  (`landing2a-cleanup-moveout-draft.md`: 8 tests/4 files ready); spec 2a/2b update incl. P-C seam (durable World
  retirement -> C2) and the H9 bullet amendment; Store full permitted Tier 2 + 3 bounded cases never completed.

## Cargo counts this turn

checks 4 (fable1 Root red; fable2 Root, Query, Store green) / clippy 0 / builds: pass 1 = Root focused build 1
(finished), Query T2 build 1 (died, disk); pass 2 = see lane logs / test runs: Root focused 1 (26 nextest
invocations inside one lane).
