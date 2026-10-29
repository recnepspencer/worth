//! Worlds killed while Serving moved selected media: an inline rewrite and a
//! final extent copy, each killed once its WAL was durable and before its
//! root published. Completing either reads the moved bytes again.

#[allow(unused_imports)] use worth_store_testkit_recovery_runtime::production_entry::published_above_checkpoint::limit_sweep::moved_media::*;
use {worth_store_testkit_recovery_runtime::production_entry::published_above_checkpoint::limit_sweep::worlds::Killed, std::path::Path, worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld, worth_proof::TransitionOutcome, worth_store_test_support::harness::physical_residency::canonical_physical_mutation_acknowledgment, worth_store_testkit_recovery_runtime::production_entry::published_above_checkpoint::limit_sweep::reached};
use {std::fs, std::path::PathBuf, std::process::Command, std::process::Stdio, std::thread, std::time::Duration, std::time::Instant};
use {worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint, worth_store::physical_runtime::PhysicalExtentCopyPhase, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationPreparationSuccess, worth_store::physical_runtime::PhysicalMutationRequest, worth_store::physical_runtime::ServingPhysicalRuntime};

#[test]
fn moved_media_child() {
    let Some(marker) = std::env::var_os(MARKER_ENV) else {
        return;
    };
    let marker = std::path::PathBuf::from(marker);
    match std::env::var(ROLE_ENV).as_deref() {
        Ok("rewrite") => park_rewrite(&marker),
        Ok("copy") => park_copy(&marker),
        role => panic!("unknown moved-media role {role:?}"),
    }
}
