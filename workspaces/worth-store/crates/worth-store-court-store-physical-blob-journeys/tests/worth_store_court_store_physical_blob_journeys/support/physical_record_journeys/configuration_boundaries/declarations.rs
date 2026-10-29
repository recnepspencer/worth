pub(crate) use {worth_proof::TransitionOutcome};
pub(crate) use {worth_store::physical_runtime::AdmittedPhysicalRecordFormat, worth_store::physical_runtime::ManifestEntryCapacity, worth_store::physical_runtime::PhysicalManifestCapacityTransition, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationPreparationDenial, worth_store::physical_runtime::PhysicalPageSizeClass, worth_store::physical_runtime::PhysicalRecordAccessPolicy, worth_store::physical_runtime::PhysicalRecordFormatDeclaration, worth_store::physical_runtime::PhysicalRecordInitialization, worth_store::physical_runtime::PhysicalRecordOpen, worth_store::physical_runtime::PhysicalRecordPlacementPolicy, worth_store::physical_runtime::RecordAppendBatch, worth_store::physical_runtime::RecordAppendDenial, worth_store::physical_runtime::RecordBootstrapDenial, worth_store::physical_runtime::RecordByteLimit, worth_store::physical_runtime::RecordReadLimits};
pub(crate) use {worth_store_physical_backend::MediaOperationRole};
pub(crate) use {worth_store_physical_format::DurableExtentManifest};

pub(crate) use {worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::durable_publication, worth_store_testkit_store::physical_record_journeys::media, worth_store_testkit_store::physical_record_journeys::read_record, worth_store_testkit_store::physical_record_journeys::scenario_configuration::dense_configuration, worth_store_testkit_store::physical_record_journeys::serving_from_initialization, worth_store_testkit_store::physical_record_journeys::success};

pub(crate) fn format_64k() -> AdmittedPhysicalRecordFormat {
    worth_store::physical_runtime::AdmittedPhysicalRecordFormat::admit(
        worth_store::physical_runtime::PhysicalRecordFormatDeclaration::builder()
            .page_size(worth_store::physical_runtime::PhysicalPageSizeClass::KiB64)
            .admit()
            .unwrap(),
    )
}