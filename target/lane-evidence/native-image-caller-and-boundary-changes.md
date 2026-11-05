# Exact caller and facade changes

This is the producer's handoff manifest, not a completed reopen claim.
The coordinator's grant allowing caller migrations for new Native variants
covers these exhaustive match additions; there is no lifecycle/fencing redesign.

## Mechanical new-variant projections in 4215bd9518

All Native paths below are relative to crates/worth-relational/src/.

| Path | Projection |
|---|---|
| durability/authority/recovery.rs | Propagate AdmissionImage without emitting a durable-failure observation |
| durability/checkpoint_history/capture.rs | Preserve the image refusal in checkpoint capture |
| durability/checkpoints/symbol_pages.rs | Classify an image refusal as damaged storage at the symbol storage-only door |
| durability/derived_index_artifacts/checkpoint_delta/readmission.rs | Explicitly unreachable for uncontrolled index DATA decode, which has no image issuer |
| history/commit/envelope_residency/artifact_export.rs | Preserve typed readmission refusal |
| history/commit/envelope_residency/demand.rs | Project to the existing storage-only damaged-envelope outcome |
| mvcc/publication/authority.rs | Reject invalid original-input capture in the existing publication denial |
| replay/authority/execution.rs | Propagate typed image refusal |
| replay/authority/strategy_replay.rs | Propagate typed image refusal |
| replay/authority/strategy_replay/lowering.rs | Propagate typed image refusal |
| runtime/initial_schema_installation/denial.rs | Reject image refusal as invalid initial inputs |
| runtime/state/subsystems/history_preparation_binding.rs | Reject invalid original payload capture in the existing publication denial |
| tests/support/durability.rs | Explicitly reject an unexpected image refusal when asserting durable damage |
| durability/authority/runtime_rebuild/envelope_replay/interruption_tests.rs | Explicitly reject an unexpected image refusal; original damage assertion is unchanged |

Other exact callers:

- crates/worth-runtime-world/src/lifecycle/owner/branch_service/creation/execution/forks.rs:
  AdmissionImage maps to the existing OwnerUnavailable branch-admission denial.
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/bootstrap/preparation.rs:
  image refusal maps to CheckpointRecoveryRejected.
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_checkpoint/denial.rs:
  8e06ce9e12 adds the missing exhaustive capture conversion and preserves a
  boxed typed image cause, matching the existing denial layout.
- workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/provider/idempotency/snapshot_resolution/source_denial.rs:
  three new nested image refusal arms map to Unavailable. Receipt minting,
  idempotency storage and close/retirement logic are unchanged.

The owner enum/conversions live in durability/data/recovery_stop.rs and
native_checkpoint_capture_denial.rs. Request/allocation/native failures are
normalized before becoming AdmissionImage; actual control stops retain their
classification. New checkpoint_admission/refusal.rs maps those stops back.

## Facades

- worth-proof lib and prelude export NativeCheckpointOriginalsVerified.
- worth-foundational durability and facade export cut/scope/product-binding
  DATA and logical Native checkpoint observation/reference/target DATA.
- worth-relational durability facade exports the captured projection, parsed
  artifact, role and refusal classifications. Decoding is not an admission door.

No Cargo feature, dependency or compatibility wrapper was introduced.

## Boundary configuration

Exactly one entry changed, in 766cf768e7:
tools/boundary-check/config/control-callers.txt replaces the caller pair

    workspaces/worth-store/crates/worth-store-relational/ workspaces/worth-store/crates/worth-store-application/src/home_controls_for_test.rs

with

    workspaces/worth-store/crates/worth-store-relational/ workspaces/worth-store/crates/worth-store-application/src/world_section/readmission/controls_for_test.rs

The controls moved to the actual coupled receiver. The captured publication
control frames its genuine Native Complete through the actual Store owner.
No pattern was widened. No src-court-baseline row or root set grew. The three
local providers and runtime_attachment baseline shrink remain unfinished.

## Remaining coordination seams

Native image admission and constructor cutover remain P-A work. Store package
framing/selection, Query interim capture/restore and World memory registry
transport are the design's future exact seams, not implemented claims. The
merged Host opening split is present; no new Host opening or Bank edit is made
by this producer step. Genuine Home codec-retirement image admission and all
current-HEAD filesystem qualification remain unrun.
