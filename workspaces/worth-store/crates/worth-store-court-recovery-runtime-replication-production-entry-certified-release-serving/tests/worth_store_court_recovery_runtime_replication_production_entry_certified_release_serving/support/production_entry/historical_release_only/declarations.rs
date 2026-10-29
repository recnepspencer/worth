//! A completed release history followed by an ordinary selected root has no
//! pending V3 descriptor; it still owes an independently rejoined Store seal.

pub(crate) use {worth_store::physical_runtime::PhysicalMutationDeadline, worth_proof::TransitionOutcome, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving, worth_store_testkit_recovery_runtime::production_entry::pending_wal_world};
pub(crate) use {std::fs, std::path::Path};
pub(crate) use {worth_store::physical_runtime::AdmittedPhysicalRecordFormat, worth_store::physical_runtime::ManifestEntryCapacity, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationOutcome, worth_store::physical_runtime::PhysicalMutationPreparationSuccess, worth_store::physical_runtime::PhysicalMutationRequest, worth_store::physical_runtime::PhysicalRecordFormatDeclaration, worth_store::physical_runtime::PhysicalRecordPlacementPolicy, worth_store::physical_runtime::RecordAppendBatch};
pub(crate) use {worth_store_physical_format::DurableRootSelector, worth_store_physical_format::RecordArtifactFile};
pub(crate) use {worth_store_recovery_runtime::PhysicalRecoveryOutcome, worth_store_recovery_runtime::WorthStoreRecovery};

pub(crate) fn selected_generation(root: &Path) -> u64 {
    let bytes = std::fs::read(
        root.join("families/records")
            .join(worth_store_physical_format::RecordArtifactFile::CurrentRootSelector.file_name()),
    )
    .unwrap();
    worth_store_physical_format::DurableRootSelector::decode(&bytes)
        .unwrap()
        .root_generation()
}