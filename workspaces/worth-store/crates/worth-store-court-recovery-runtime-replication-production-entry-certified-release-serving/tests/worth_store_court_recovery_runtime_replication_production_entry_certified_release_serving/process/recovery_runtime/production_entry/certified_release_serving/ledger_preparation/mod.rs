//! A genuine recovered seal enters Serving only under the residency policy its
//! recovery was funded by; a different Serving policy is refused before any
//! publication, and the same media then reopens under the original policy.
//!
//! The Serving open policy must equal the recovery policy
//! (`PhysicalResidencyOwner::validate_recovered_policy` returns
//! `RecoveredResidencyPolicyMismatch`), so a world with an adequate recovery
//! policy and a tighter Serving ceiling cannot be built. The combined
//! seal-plus-bootstrap resident ceiling is unit-tested by
//! `recovery_residency::tests::retained_state_and_reducing_ceiling_precede_allocation`.

#[allow(unused_imports)] use crate::production_entry::certified_release_serving::ledger_preparation::*;
use {std::num::NonZeroU32, std::num::NonZeroU64, std::path::Path};

use {worth_store::physical_runtime::AdmittedPhysicalRecordFormat, worth_store::physical_runtime::CheckpointMemoryLimit, worth_store::physical_runtime::FilesystemAccessPosture, worth_store::physical_runtime::FilesystemMediaAdmission, worth_store::physical_runtime::GroupCommitDelay, worth_store::physical_runtime::GroupCommitLimit, worth_store::physical_runtime::IdempotencyRetentionGenerations, worth_store::physical_runtime::LiveIdempotencyBindingLimit, worth_store::physical_runtime::PendingUnresolvedMutationLimit, worth_store::physical_runtime::PhysicalCheckpointPolicy, worth_store::physical_runtime::PhysicalDurabilityDeclaration, worth_store::physical_runtime::PhysicalIdempotencyPolicy, worth_store::physical_runtime::PhysicalOperationAllocationScope as Scope, worth_store::physical_runtime::PhysicalRecordAccessPolicy, worth_store::physical_runtime::PhysicalRecordFormatDeclaration, worth_store::physical_runtime::PhysicalRecordOpen, worth_store::physical_runtime::PhysicalRecordResidencyPolicy, worth_store::physical_runtime::PhysicalRuntimeAdmission, worth_store::physical_runtime::PhysicalSpeculativeWorkKind as Speculation, worth_store::physical_runtime::PhysicalStore, worth_store::physical_runtime::PhysicalWalPolicy, worth_store::physical_runtime::RecordBootstrapDenial, worth_store::physical_runtime::RecoveredPhysicalCheckpointCustody, worth_store::physical_runtime::RetainedWalTailLimit, worth_store::physical_runtime::WalSegmentByteLimit, worth_store::physical_runtime::WalSegmentInventoryLimit};
use {worth_store_recovery_runtime::PhysicalRecoveryOutcome, worth_store_recovery_runtime::WorthStoreRecovery};

use {worth_proof::TransitionOutcome, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::open_serving_with_seal, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery::recover, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request_with_memory};



#[test]
fn recovered_seal_refuses_a_different_serving_policy_then_reopens_under_its_own() {
    let worker = std::thread::Builder::new()
        .name("recovered-serving-ledger-preparation".to_owned())
        .stack_size(16 * 1024 * 1024)
        .spawn(move || {
            let world = crate::production_entry::certified_release_serving::ledger_preparation::world::checkpointed_partial_release_world();
            let producer_policy = world.serving().residency_observation().admitted_policy();
            // Size this fixture from the producer's admitted scope. Fresh C8
            // still obtains its own allocation authority for this request.
            let original_recovery_bytes = producer_policy
                .operation_bytes()
                .min(producer_policy.scope_bytes(worth_store::physical_runtime::PhysicalOperationAllocationScope::Recovery));
            let retained = world.retained_root();
            let root = retained.path().to_path_buf();
            drop(world);

            let selected_checkpoint = root.join("families/checkpoint.current");
            let before = std::fs::read(&selected_checkpoint).expect("selected checkpoint bytes");
            let selected_root = root.join("families/records/root-current.selector");
            let root_before = std::fs::read(&selected_root).expect("selected root selector bytes");
            let seal = recover_seal(&root, original_recovery_bytes);
            assert_policy_mismatch_refusal(&root, seal, original_recovery_bytes / 2);
            assert_eq!(
                std::fs::read(&selected_checkpoint).expect("checkpoint after refusal"),
                before,
                "policy refusal must precede a new publication"
            );
            assert_eq!(
                std::fs::read(&selected_root).expect("root selector after refusal"),
                root_before,
                "policy refusal must preserve the selected root"
            );

            // The seal was consumed by the attempted open. A fresh C8/Store
            // rejoin under the original policy still opens and checkpoints
            // through the ordinary production path.
            worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::open_serving_with_seal(&root, recover_seal(&root, original_recovery_bytes));
        })
        .expect("recovery worker");
    worker.join().expect("recovery worker did not panic");
}
