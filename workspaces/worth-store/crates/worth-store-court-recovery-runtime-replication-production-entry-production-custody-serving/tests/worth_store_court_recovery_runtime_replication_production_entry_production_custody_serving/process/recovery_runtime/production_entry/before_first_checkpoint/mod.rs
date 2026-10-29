//! A store killed before its first checkpoint recovers from the
//! generation-zero basis: the whole WAL from the canonical origin.

#[allow(unused_imports)] use crate::production_entry::before_first_checkpoint::*;
use {std::fs, std::path::Path, std::path::PathBuf, std::process::Command, std::process::Stdio, std::thread, std::time::Duration, std::time::Instant};

use {worth_store::physical_runtime::ExternalPhysicalRecordLocator, worth_store::physical_runtime::PhysicalCheckpointStartDenial, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationOutcome, worth_store::physical_runtime::PhysicalMutationPreparationSuccess, worth_store::physical_runtime::PhysicalMutationRequest, worth_store::physical_runtime::RecordAppendBatch, worth_store::physical_runtime::RecordByteLimit, worth_store::physical_runtime::RecordReadLimits, worth_store::physical_runtime::ServingPhysicalRuntime};
use {worth_store_recovery_runtime::PhysicalRecoveryOutcome, worth_store_recovery_runtime::RecoveredPhysicalRuntimeHandoff, worth_store_recovery_runtime::WorthStoreRecovery};

use {worth_store::physical_runtime::BlobCheckpointLimit, worth_store_blob_chunks::BlobChunkSize, worth_store::physical_runtime::BlobIngestDeclaration, worth_store::physical_runtime::BlobIngestFailure, worth_store::physical_runtime::BlobReadLimits, std::num::NonZeroU64, worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld, worth_proof::TransitionOutcome, worth_store_testkit_recovery_runtime::production_entry::blob_scope_admission::admitted_blob_scope, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving};

#[test]
fn a_store_killed_after_initialization_reopens_clean() {
    assert_recovers_reads_back_and_reopens(crate::production_entry::before_first_checkpoint::FirstWrite::Nothing, 0xd1);
}

#[test]
fn a_record_written_before_the_first_checkpoint_recovers_byte_exact() {
    assert_recovers_reads_back_and_reopens(crate::production_entry::before_first_checkpoint::FirstWrite::Record, 0xd2);
}

#[test]
fn a_two_chunk_blob_published_before_the_first_checkpoint_recovers_byte_exact() {
    assert_recovers_reads_back_and_reopens(crate::production_entry::before_first_checkpoint::FirstWrite::TwoChunkBlob, 0xd3);
}

#[test]
fn a_checkpoint_appearing_before_construction_rereads_denies_the_generation_zero_basis() {
    let world = crate::production_entry::before_first_checkpoint::KilledWorld::launch(crate::production_entry::before_first_checkpoint::FirstWrite::Record);
    let current = world.root.join("families/checkpoint.current");
    let outcome = worth_store_recovery_runtime::WorthStoreRecovery::certification_recover_with_custody_pauses(
        worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request(&world.root),
        |_| {},
        move || std::fs::write(current, []).unwrap(),
    );
    assert!(
        !matches!(outcome, worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(_)),
        "construction must re-read the absent checkpoint: {outcome:?}"
    );
}

#[test]
fn before_first_checkpoint_child() {
    let Some(role) = std::env::var_os(ROLE_ENV) else {
        return;
    };
    let marker = std::path::PathBuf::from(std::env::var_os(MARKER_ENV).expect("child marker"));
    let write = crate::production_entry::before_first_checkpoint::FirstWrite::ALL
        .into_iter()
        .find(|write| Some(write.role()) == role.to_str())
        .expect("generation-zero child role");
    child(&marker, write);
}
#[cfg(all())]
#[path = "still_denied/mod.rs"]
pub(crate) mod still_denied;
