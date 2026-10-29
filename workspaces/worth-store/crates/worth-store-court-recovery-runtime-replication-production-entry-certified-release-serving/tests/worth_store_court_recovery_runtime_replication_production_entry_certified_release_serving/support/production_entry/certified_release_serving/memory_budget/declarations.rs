//! A selected V2 source closure has one recovery-memory budget, not a fresh
//! full-sized allowance for each roster and media observation owner.

pub(crate) use {std::fs, std::io::ErrorKind, std::path::Path};

pub(crate) use {worth_store::physical_runtime::PhysicalRecoveryRejoinResidentDenial, worth_store::physical_runtime::RecoveryWalAllocationDenial};
pub(crate) use {worth_store_physical_format::RecordArtifactFile};
pub(crate) use {worth_store_recovery_runtime::PhysicalRecoveryBlockKind, worth_store_recovery_runtime::PhysicalRecoveryLimitDimension, worth_store_recovery_runtime::PhysicalRecoveryOutcome, worth_store_recovery_runtime::PhysicalRecoverySourceDenial, worth_store_recovery_runtime::WorthStoreRecovery};

pub(crate) use {worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_proof::TransitionOutcome, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::open_serving_with_seal, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery::recover, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request, worth_store_testkit_recovery_runtime::production_entry::release_reopen, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request_with_memory};

pub(crate) fn selector(root: &Path, artifact: RecordArtifactFile) -> Option<Vec<u8>> {
    let path = root.join("families/records").join(artifact.file_name());
    match std::fs::read(path) {
        Ok(bytes) => Some(bytes),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => panic!("selected selector read failed: {error}"),
    }
}