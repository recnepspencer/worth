//! A killed high-index anchor advance must replay from WAL, not its old root.
pub(crate) use crate::killed_child;
pub(crate) use worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint as Seam;
pub(crate) use {
    crate::durable_run::open, crate::incarnation::assert_contended, crate::workload,
    std::path::Path, worth_execution::ExecutionRequest,
};
pub(crate) use {std::path::PathBuf, std::thread, std::time::Instant};
pub(crate) use {
    worth_store::physical_runtime::AnchoredRecordName,
    worth_store::physical_runtime::AnchoredRecordPublication,
    worth_store::physical_runtime::PhysicalMutationDeadline,
};

pub(crate) fn publish(
    home: &worth_store_relational::RelationalStoreHome,
    text: &str,
    generation: u64,
    payload: Vec<u8>,
) {
    home.certification_serving()
        .anchored_records()
        .unwrap()
        .publish(
            worth_store::physical_runtime::AnchoredRecordPublication::new(
                worth_store::physical_runtime::AnchoredRecordName::new(text).unwrap(),
                generation,
                payload,
                placement(),
                worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000)
                    .unwrap(),
            ),
        )
        .unwrap();
}

pub(crate) fn placement() -> worth_store::physical_runtime::AdmittedRecordPlacementPolicy {
    use worth_store::physical_runtime::*;
    PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(ManifestEntryCapacity::new(64).unwrap())
        .admit(AdmittedPhysicalRecordFormat::admit(
            PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
        ))
        .unwrap()
}
