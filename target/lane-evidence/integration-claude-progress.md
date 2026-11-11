# Integration (Fable 5.2 takeover) progress

## HANDOVER TO CODEX (2026-11-10, Claude integration lane ends here)

Worktree `C:/programming/worth worktees/worktrees/m3-host-home`, branch `codex/m3-host-home`, nothing pushed.
**HEAD `105e888f4e`.** Windows; use PowerShell (Git Bash is very slow). Standing rules: never push, stash,
reset, `checkout --`, or rewrite history; never touch other worktrees; never run `#[ignore]` tests,
query-scale, the exhaustive Home oracle or the Store scenario suite; clean cutover, never weaken/delete/ignore a
test to pass; every commit gets an independent reviewer; check disk >= 25 GB before any build.

### Integration commits since master merge `171d380829` (oldest first; lane commits arrive inside the merges)
- `54b4fa230d` repair batch for master merge #2 + Ruling 1 (D13 hold-out, 31 courts moved with restore rows) +
  Ruling 2 (sharing inspection is a checked upper bound) + index_catalog journal readmission fix. Reviewed APPROVE.
- `2610c94ef3` chore: runner telemetry `scripts/test_cost/slots/**` (data only).
- `fbd994d981` evidence: Store/Bank ledgers + classifications (subject line has a stray BOM; left, no rewrite).
- `c7c2ade607` merge P-C `2dce9c178c` (index selections through Complete checkpoints; cures IdentityIndexUnavailable
  after reopen). Conflicts: semantic readmission -> P-C's `custom_observation` (mirrors production).
- `8283728aff` merge H8 `1ce3832b78` (durable dispatch custody, stored-text reopen `2d6822abbe`). P-C's Store
  `query_journal::ApplicationQueryJournal` is the ONE journal transport; H8's `journal_frame_publication` dropped;
  `query_dispatch` rebuilt over it; `read_frame` returns None only for head+1; Query custody readmission takes
  obligations + correction material + dispatch; Host admits corrections, obligations, dispatch;
  `DispatchOutbox | DispatchAttempts => Supported(8)` in P-C's phase model.
- `32341309d8` merge P-A `e4d3b01c8d` (Native courts ported to court crates; cleanup courts moved out with
  restore rows). Row-by-row: `fable-merge-pa-resolution.md`. Master had kept Full `region_initialization.rs` /
  `cell/resident_full.rs` live, so P-A's orphan deletion of those is NOT applied (follow-up for P-A).
- `4d05497ab0` Windows linker wrapper `scripts/windows_linker.cmd` (`rust-lld -flavor link %* /pdbpagesize:16384`)
  via `.cargo/config.toml`; court `build.rs` + toml build-dep + lock entries removed (coordinator PDB ruling).
- `f1dd72fe78` compile fallout (Store owner-admission arm, Stored text arm in a P-A court, StringInterner import,
  example `SetRetentionInput: Deserialize` - red on P-C's own branch, dead cfg(test) seams, unit-only
  `TestRecoverySourceRefusal`). Check round `fable-batch1c`: Root/Query/Store/Bank 0 diagnostics.
- `1e8bd528be` root Cargo.lock sha2 disambiguation (lockfile alignment PASS).
- `105e888f4e` batch review fixes: layout test expects the wrapper (was BLOCKING), 3 restore rows -> `8283728aff`,
  stale Full-region spec paragraph, `.gitattributes` `scripts/*.cmd eol=crlf`.
Batch review (independent, `fable-batch1-review.patch`): resolutions confirmed equivalent on both sides; its
fixes in `105e888f4e` have NOT had a reviewer re-check.

### Uncommitted / not done
- `target/lane-evidence/integration-progress.md`, `master2-compile-handoff.md`: pre-existing dirt from the
  previous owner; never part of a commit.
- NOT DONE (review SHOULD-FIX 3 remainder): rename the mount `kind_commitment_preimage_tests` back to
  `full_tests` in `crates/worth-relational/src/storage/residency/region_publication.rs:203-204` (P-A's name for
  HEAD's file). Deferred so no Rust source changed under the running Tier 2. Then one reviewer re-check of
  `105e888f4e` + the rename.
- This progress note itself (commit with `git add -f`).

### Tier 2 running (started 2026-11-10T21:22Z, on clean source at `1e8bd528be`)
`powershell -File target/lane-evidence/fable_batch_lanes.ps1 -Stages "root-t2,query-t2,fixture-t2,store-t2,bank-t2" -Tag batch1`,
serial, permitted policy (`master2-tier2-permitted.py`). Per stage: log `target/lane-evidence/fable-batch1-<stage>.log`
(header: command, HEAD, SOURCE_CLEAN, LINKER_OVERRIDE_CLEARED, C_FREE_GB, START), out dir
`target/lane-evidence/fable-batch1-tier2-<root|query|fixture|store|bank>/` (`run-*.log` per binary).
Finished when `fable-batch1-bank-t2.log` ends with `END:` and `EXIT:` lines (each stage writes them; a red stage
does not stop later ones). Do NOT relaunch a stage whose out dir exists; read it. Read the mixed utf8/utf16 logs
with NULs stripped (the scratch `logrep.py`/`groups.py` idea: group failures by `thread '...' panicked at <site>`).
Expected: the Query/Store index-after-reopen reds (Q1, S3) cured by P-C; reclassify everything against the
families below. Note: `105e888f4e` changed only a Python test, the spec and .gitattributes.

**Tier 2 RESULT (finished 2026-11-11T01:29Z):** root-t2 EXIT 1, the FIRST Root Tier 2 after the repair. Per
binary (`fable-batch1-tier2-root/run-*.log`): execution 400/400, signal 2409/2409, determinism 15/15, relational
2755 pass / 52 fail, root-rest 2886 / 70 fail, root-cost 60 / 23 fail (F4, C2), root-exhaustive 108 / 2 fail,
root-process 152 / 4 fail, root-compile-fail 51 / 1 fail; totals 9036 passed, 152 failed. Not yet grouped into
families (do that first; compare against F1-F7). query/fixture/store/bank stages did NOT run: the runner exits 75
"DEFERRED BEFORE PREPARATION: finish and commit Root repair batch" when Root is red. Their out dirs do not
exist, so launch them (after the Root classification) with the runner's Root deferral handled honestly, not
bypassed silently. Disk at deferral: 39 GB free (delete regenerable `incremental` caches in this worktree first).

### Linker override (machine finding)
This machine sets user env `CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER` = rust-lld 1.94.0; env beats config, so the
committed linker (and before it `rust-lld`) was silently bypassed. `fable_batch_lanes.ps1` runs
`Remove-Item Env:\CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER` for its own process only (never change the user's
environment). P-C `765e8ccdb0` makes qualification refuse an inherited `CARGO_TARGET_*_LINKER`; any lane script
must clear it first. Wrapper verified on a scratch crate (17k-char link via rustc response file, PDB block 16384);
the 6.57 GB-PDB court `worth-query-court-decl-host-authored-ownership` is first linked by this Tier 2.

### Known red families (before this batch's Tier 2; none claimed fixed)
Root (`fable-repair-root-families.md`, selected failures 2215 pass / 91 fail):
- F1 cold checkpoint capture `UnsupportedProvider` (27): FIXTURE - court cold sources implement only legacy reads,
  owned reads are mandatory. Next: move fixtures onto the testkit durable medium (P-A ports may cover some).
- F2 hostile-frame reseal composes v5, frames are v8 (12): CLEANUP fixture; helper must compose current commitment.
- F3 `ColdReconstructionRequired` after checkpoint/catalog restore (10): likely HAPPY; first follow-up after Tier 2.
- F4 exact accounting/cost courts (24): C2 by Ruling 2 per court (exact equality moves out; bound courts stay).
- F5 version/tamper fixtures (11): CLEANUP C1/C4 unless refusal order regressed.
- F6 server durable Query retry `ProviderUnavailable` (5): likely HAPPY (Query Home idempotency reopen).
- F7 medium fate (2): C1.
Query (`fable-repair-query-families.md`, T2 7633 pass / 107 fail): Q1 index-after-reopen 64 (P-C cure expected);
Q2 `ConflictingCreateCorrespondence` 8; Q3 `ReceiptMismatch` 8; Q4 injected unwind 4 (C3); Q5 cold capture 2 (=F1);
Q6 `DurableBranchPublicationRequired` 2; Q7 other 18; Q8 `authority_cause` 9402-probe sweep ended by hand after
46 CPU min (not a spin, from master) - named follow-up.
Store (`fable-repair-store-bank-families.md`, 52 red; Bank GREEN 324/324): S1 Query idempotency
`ProviderUnavailable` after reopen ~13 (=F6); S2 `OriginalBindingUnavailable` at Host opening incl.
`partial_released_key_admission_retries_same_home_without_duplicate_retention`; S3 IdentityIndexUnavailable 2 (P-C);
S4 `UncleanRecoveryUnsupported{Relational}` 10 (4 `world_durable_release` = the D18 gate `d27064d826` question);
S5 `WorldSectionRequired` 7 (unclassified); S6 exact cost/residency 8 (C2); S7 ColdReconstruction 1 (=F3);
S8 retired-format refusal text 3 (CLEANUP); S9 phase-eight process timeouts/C8 6 (C1); S10 other 2.
Classifications (`fable-classifications-2026-11-10.md`): authored_index_writes replay = HAPPY, fixed in
`54b4fa230d`; 60 s close maintenance deadline = C2 (graceful-close courts must not depend on wall time; never
retry-until-green or raise the timeout).
Boundary (`fable-repair-static-gates.md`, boundary-check RED, not re-run after the batch): the court `build.rs`
finding is now resolved by `4d05497ab0`; 3 source tests rooted on `ExecutionRequest` from the runtime batch
(`field_authoring_candidate/tests/storage_footprint.rs`, `record_arena/values/tests/optional_output_funding.rs`,
`commit_semantic_result/validation/tests/proposed_families.rs`) must move to court crates (never grow
`src-court-baseline.json`); pre-existing BC9002 (~20) and BC7014 debt. Reviewer SHOULD-FIX still open from
`54b4fa230d`: saturating inventory totals (`record_arena/arena/allocation.rs:37`, `overlay/partition.rs:89-97`,
`branch/root.rs:304`) - open until accepted as C2. Also open: `unsupported_root_version.rs` uses ExecutionRequest
without a baseline row (pre-existing); Query `Cargo.toml:89-99` `debug = false` reasons cite the 4 GiB PDB limit
the wrapper may lift; `scripts/macos_link/test_response.py` 2 Windows temp-path errors (pre-existing).
Static gates after the batch NOT yet run: boundary-check, agent-context check, fmt.

### Next merge batch (one batch, one Tier 2, reviewer)
P-C `765e8ccdb0` (partial H6/H7/H9 surfaces gated to typed Directory refusals; lane linker-override refusal;
stored-text decoders overlap H8 `2d6822abbe` - take identically), then `e235a9b75a`, `10d117085f`; later H8 and P-A
heads. H5: ONLY after H5 LANDED (then: remove the two retired World `durability/medium.rs` variants; single
`WORLD_CHECKPOINT_VERSION` 2 with a v1-refused court).

### Open items from `orchestration/m3/prompts/integrate_pending.md`
- Classify the 4 Store `world_durable_release` killed-child courts vs D18 gate `d27064d826` (S4): move out under
  the right C phase with restore rows, or fix the gate.
- `genuine_complete_section_retains_capture...` goes green with P-A's Complete/ACK gate item; don't fix here.
- P-C follow-up commit migrating five in-source Native `ExecutionRequest` test files (from `2dce9c178c`,
  `bf01d4d731`) to owner units + court proofs: take with the next P-C batch; baseline must not grow.
- P-C H9 seams; P-C coherence rows go into `landing2a-coherence-pc.md`.
- PDB ruling: DONE (`4d05497ab0`) except proving the large court links (this Tier 2).

### Landing 2a prep remaining (do not start the 2a gate before H5 LANDED)
- Refine `landing2a-coherence.md` with post-batch evidence, folding in `landing2a-coherence-pc.md`.
- Gate the 4 PARTIAL rows to typed refusals (P-C `765e8ccdb0` does much of this - take it first).
- Cleanup-court move-out with restore refs (`landing2a-cleanup-moveout-draft.md`).
- Spec 2a/2b update: P-C seam (durable World retirement -> C2; `AuthoritativeDurableHistory`,
  `DurableBranchPublicationRequired` as Refused rows), H9 bullet amendment, C2 close-deadline item.
- Store full permitted Tier 2 plus the 3 bounded cases (the running Tier 2's store stage is the first full one).
- Follow-ups F3, F6, F1 in that order after the Tier 2 results.

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

Batch review (Opus, read-only, `fable-batch1-review.patch`): BLOCK on one item, resolutions otherwise confirmed
equivalent. Fixed in the working tree (UNCOMMITTED, waiting for Tier 2 to finish so no Rust file changes under
the running build): (1) BLOCKING `scripts/macos_link/test_layout.py` expects the wrapper path; (2) milestone-3
restore rows for cold_read_matrix/exhaustive, cold_symbol_read_doors, cold_index_frontier now restore from
`8283728aff` (HEAD-added assertions); (3) stale Full-region paragraph rewritten; `.gitattributes`
`scripts/*.cmd text eol=crlf`. STILL TO DO after Tier 2: rename `mod kind_commitment_preimage_tests` back to
`full_tests` in `storage/residency/region_publication.rs:203-204`, commit, reviewer re-check of the fix diff.
Pre-existing, not ours: `scripts/macos_link/test_response.py` 2 Windows temp-path errors. Follow-up: Query
`Cargo.toml:89-99` `debug = false` reasons cite the 4 GiB PDB limit the wrapper may lift.
Tier 2 running: `fable_batch_lanes.ps1 -Stages "root-t2,query-t2,fixture-t2,store-t2,bank-t2" -Tag batch1`
at `1e8bd528be` (out `fable-batch1-tier2-<ws>/`, logs `fable-batch1-<stage>.log`).

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
