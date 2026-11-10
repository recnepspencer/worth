# CP2b assertion changes

This ledger covers existing court assertions that CP2b changed, from base `39ad4063e0` (the merged master) to the CP2b handoff, and the CP2a-era changes that `p5-cp2a-changed-assertions.md` does not name. CP2b loosened no assertion silently and added or removed no `#[ignore]`. Purely new courts are listed at the end for completeness.

## Owned born-stale courts: none moved

Slice 3b (verifying an owned route whose own effect moved its source read, inside the edit) is parked (D1571, public #137). Owned born-stale routes keep CP2a's delivery successor, 1 contact and 1 commit, exactly as on master. So no court from D1474 condition A's inventory moved. `synchronous_edit/convergent.rs` keeps 1/1 on the synchronous program (`:169-176`); the two witness-withheld courts added for slice 3a (#126) assert the same counts.

## Preimage retention denials are typed pre-effect refusals (D1551, public #122)

Commit `0621393ad2`. Master `e063f9b063` made every retention denial a pre-effect refusal that carries its cause, so these courts were red on clean master. The typed denial is the contract.

- `worth-query-court-execution-primary-graph/.../application_attempt/preimage_retention/mod.rs` `assert_retention_denied`:
  - It used to be `matches!(outcome, Aborted)`. It now requires `Denied` with kind `ProviderRejected`, stage `ProviderCommit`, and an exact `preimage_retention_denial()`.
  - Right record, wrong field gives `MissingDemandedField`. Two fields on two records gives `AmbiguousDemandedField`.
  - Both courts keep the head-unchanged check and add `next_valid_commit_publishes`: after the refusal, the same application commits a valid operation that publishes.
- `worth-query-execution/.../commit_preparation/preimage_retention/tests.rs`: the semantic clause now expects `Stop::PreEffectDenied` (exact cause, `ProviderRejected`, `Closed`) instead of `Stop::Denied` for `EmptyDemand`, and adds `MissingDemandedField`, `AmbiguousDemandedField` and `NoMutatedRecord`. The cancellation clause is unchanged.
- Faults (`d1551-faults.log` on the Mac):
  - removing `e063f9b063`'s line fails both courts (`Aborted`) and the unit test;
  - letting a refused commit go past retention fails both courts (`Committed`).

## The demand after a synchronous shrink settles (public #126, slice 3a)

Commit `b530c059a5`, `synchronous_edit/reconciliation/demand_after_shrink.rs`.

- The courts written to pin the defect (`a_demand_after_a_synchronous_shrink_refuses_until_public_126_is_fixed` plus its worker variant: `RetainedBasisUnavailable`, Terminal, 0/0) are replaced by the settled journey their own comment asked for.
- The four journeys go through the performed handle and through a fresh demand, hot and worker. Each asserts (2 contacts, 1 commit), named rows equal to the legacy oracle, then (0, 0).
- `reconciliation/kept_refusals.rs` keeps what still refuses. The factless undecidable record refuses `RetainedBasisUnavailable`, now hot and worker after #132 (`cda3fa96f4`). Every stopped run asserts its stop fired, which is stricter.

## Ignore reasons on the two legacy-oracle courts

`reconciliation/comparison.rs:131,136` (`synchronous_shrink_matches_the_legacy_complete_row_oracle` plus its worker variant, `4ee2127921`, D1472). The reason changed from "CP2b pending: net sequential producer rounds before atomic publication" to "Postponed: public #119 / proprietary #100 retirement metadata; assertions retained". The assertions are unchanged, and both courts stay ignored.

## CP2a-era changes the CP2a ledger does not name

- **`ececd430a1`, testkit `terminal_unavailable.rs` `terminal_owner`.** Besides the changes ledgered under D1462/D1463, it dropped two checks:
  - the intermediate close refusal `NativeObligationRetained`;
  - the Outputs/Demand loop's `recovery_posture() == Terminal`.

  The atomic refusal they described no longer occurs. The court now asserts zero publications on every branch, an unchanged head, `SEED_RETENTION` and `Terminal{NativeTerminal}`.
- **`e937293a69`.**
  - Managed cleanup is now plural: `managed_cleanup_failures()` and `managed_cleanup_receipts()` replace the single accessors. The courts assert `len() == 1` or `len() == 0` where they asserted `is_some()` or `is_none()`, which is stricter. Files: `source_only/capability/grant_commands`, `native_workflow_operations/capability/grant_commands`, `source_only/managed_cleanup`, `managed_cleanup/retained_capacity`.
  - `scripts/test_topology/test_src_court_doors.py:77-89` now expects `worth-query-publication/` in the owner set.
- **`3b21dbe1ec`, `worth-court-relational-certification/.../predecessor_basis/fixture.rs` `reopen`.** It readmits through `readmit_current_owned_output_successor` instead of a caller-supplied digest and roles. This is an API migration; the callers' assertions are unchanged.
- **`badd828cb7`, `generated_source/courts.rs:23-47`.** The protected variant first records a real seed edit, so its pre-edit C holds a genuine grant. `member_edit` delivery is now `SeedDelivery`.
- **`662084fc2f`, `entity_resolution/tests/mod.rs:150`.** `identity_index_generation()` returns an `Option`, so the court calls `.expect(..)` first; its `assert_ne!` is kept.
- **`8c098f2a2e`, `worth-relational/src/storage/residency/mixed_history_tests.rs:176`.** The test name `..._append_preserve_the_...` became `..._append_publication_the_...`, which looks like a mechanical rename side effect; the body is unchanged. `scripts/test_cost/slots/root/baselines-00-000026-3773263ac501c210.jsonl:19` still names the old test. Left for a follow-up issue, since it is not a CP2b change.

## New CP2b courts (additions, for completeness)

- **Slice 1 and the slope slices:**
  - `synchronous_edit/graph/many_routes/*`, covering the round limits and the attempt total;
  - `touch_work`, `ownership_reads` and `rebase_walk` (cost courts);
  - `synchronous_edit/round_budget`;
  - `corruption` (continued-round guards).
- **Slice 3a:** `reconciliation/kept_refusals`.
- **#132:** `reconciliation/late_stop`.
- **Slice 4:**
  - `owned_outputs/predecessor/origin_kind` (an exact-kind predecessor court: a historical-selection predecessor is a typed data posture, with mutant M-P1);
  - `reconciliation/genuine_retirement` (N1: a genuine C retirement is Retire in the edit's origin and gone from live state at the edit commit, with mutant M-N1).
- **Mutants M-G1 and M-A1** ran against the existing `publication_count` and `failure` courts (`d1572-mutants.log` on the Mac).

## Follow-ups noticed while ledgering

`reconciliation/required_output_courts.rs` `settle` still has an `eprintln!` debug line from CP2a (around `:283`). It is harmless, so it is left for whoever next touches that court.
