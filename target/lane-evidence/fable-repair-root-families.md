# Root selected-failure rerun after the master-merge repair batch: remaining red families

Lane: `fable-repair-root-focused.log` (HEAD `171d380829` + uncommitted repair batch, 2026-11-10 07:33Z–09:08Z,
exit 1). Out dir `fable-repair-root-focused/`. Full per-test list with panic site and message:
`fable-repair-root-focused-families.txt` (53 groups, 91 tests).

Selection: the 2272 tests that failed in `master2-combined-tier2/root` plus every test body the batch changed or
added (2306 run). Result: **2215 passed, 91 failed.** Of the 2272 prior failures 2180 now pass, 4 were renamed
(their replacements ran), 88 still fail. Of those 88, 12 fail for the same cause as before; 76 fail for a NEW
visible cause that the shared seed defect (`original semantic result: ValidationMismatch`) was masking. 3 of the
91 are renamed or changed bodies. None of the 91 was fixed in this commit; none is claimed by it.

No test in this list was passing before the batch (the lane selects prior failures and changed bodies only;
a Root Tier 2 over unselected tests has NOT been run after the repair).

| # | Family (cause as printed) | Tests | Track call | Evidence / next step |
|---|---|---|---|---|
| F1 | Cold checkpoint capture refused: `UnsupportedLegacySemantics "original protected canonical source unsupported"` (`durability/checkpoint_history/capture.rs:105`, from `Refusal::UnsupportedProvider`; default refusals live in `durability/medium/cold_source.rs`) | 27 | UNCLASSIFIED (HAPPY if the product medium also lacks the provider; fixture-only otherwise) | sites `cost/relational/cold_checkpoint.rs:132` (15), `history_ownership/fixture/cold.rs:119` (10), `content/historical_kind/retention_roundtrip.rs:322` (2). One cause; find which medium method the fixtures' medium leaves at its default. |
| F2 | Hostile-frame reseal helper composes the v5 commitment; product frames are v8 (`left: 8 right: 5`) | 12 | CLEANUP fixture (tamper courts) unless the bump is unintended | `tests/support/durability/canonical_commit_frame.rs:6`, court `support/canonical_commit_frame.rs:6`, `durable_publication/semantic_result.rs:117`. Helper must compose the current commitment. |
| F3 | `derived index maintenance denied: ColdReconstructionRequired` on publication after a checkpoint/catalog restore; one `CorruptIndexEntries` | 10 | likely HAPPY product (same root as the fixed `authored_index_writes` red: master clears installed routing on every definition insert, `definition_catalog.rs:91`) | sites `worth-testkit-relational/src/durable_publication.rs:72` (4), `durable_publication/index_catalog.rs:50,83`, `checkpoint_admission/indexes.rs:121`, `retained_indexes.rs:148`, `cold_index_refresh.rs:285`, `tests/support/records.rs:127`. The journal-delta path is fixed in this batch; these paths are not. FIRST follow-up. |
| F4 | Exact accounting / cost / memory-ceiling equality courts (constant deltas such as 8192; `PolicyMemoryExhausted`; work 22897 vs 23433) | 24 | C2 by Ruling 2 where the court asserts exact equality; each court still needs its own read (bound >= actual courts must stay and pass) | root COW x3, `history/commit/artifact_tests.rs:260`, `checkpoint_stage.rs` x4, `content/durable_publication.rs` x3, `byte_payloads.rs` x2, `work_account.rs` x2, others in the families file. Cannot be split between master's `im` sharing and this batch's upper-bound extent: all were masked before. |
| F5 | Version / tamper refusal courts with stale fixtures (branch-root format 6 vs 5, relabelled roots refused earlier as `CorruptCheckpoint`, checkpoint model bytes, SoA column, digest mismatch) | 11 | CLEANUP (C1/C4) unless a refusal order regressed | `tests/durability/contracts/branch_root_*`, `relational/checkpoint_versions.rs:47`, `kind_authentication_tests.rs:160`, `restore_tests.rs:327`, `required_outputs.rs:73`, `historical_artifact.rs:32`, `payload_encoding_tests.rs:106`. |
| F6 | Server durable Query retry: `WorthQueryApplicationIdempotencyResolutionDenial { kind: ProviderUnavailable }` | 5 | likely HAPPY (restart returns the original receipt; depends on Query Home idempotency reopen, H7/H8) | `worth-court-server-certification-suite/process/durable_query_retry/*` (4), `phase-eleven/server/compat_http_phase_three/mod.rs:222` (1). Re-run after H8's stored-text reopen repair `2d6822abbe` merges. SECOND follow-up. |
| F7 | Medium fate courts (refused publication proven no effect; unresolved fate named) | 2 | CLEANUP (C1 faults) | `tests/durability/medium/durable_before_live.rs:86`, `unresolved_fate.rs:55`. |

F1 root cause (read 2026-11-10, after the repair commit): FIXTURE, not product. Checkpoint capture
(`durability/checkpoint_history/capture.rs:25-36`) now takes the provider's owned read protection
(`RelationalDurableReadProtection::capture` -> `checkpoint_read_protection`), and master made the owned
read protocol mandatory ("No legacy read fallback"). The courts' hand-rolled cold sources implement only the
legacy methods (`retain_members`, `read_member`, `read_commit`): e.g. `ColdCheckpointSource` in
`cost/relational/cold_checkpoint.rs:58-98`, so every owned method falls to the trait default
`UnsupportedProvider` (`durability/medium/cold_source.rs:22-128`). Fix: move these fixtures onto the testkit's
real durable medium (or implement the owned protocol honestly). Per family: the cold-restore cost-law courts
are C2/C4; the `history_ownership` cold courts need a per-court call before moving.

Track calls marked "likely" or "UNCLASSIFIED" are from the printed cause and the emitting source line only; no
family was analysed to root cause in this turn.
