pub(crate) use {worth_proof::TransitionOutcome};
pub(crate) use {worth_signal::facade::TemporalDuration};
pub(crate) use {worth_store::physical_runtime::lower_physical_durability_performance_receipt, worth_store::physical_runtime::PageBasisPerformanceExpectation, worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_store::physical_runtime::PhysicalDurabilityCloseoutOutcome, worth_store::physical_runtime::PhysicalDurabilityPerformanceContract, worth_store::physical_runtime::PhysicalDurabilityPerformanceEvidenceDenial, worth_store::physical_runtime::PhysicalManifestCapacityTransition, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationPreparationSuccess, worth_store::physical_runtime::PhysicalMutationRequest, worth_store::physical_runtime::PhysicalRecordInitialization, worth_store::physical_runtime::PhysicalRecordOpen, worth_store::physical_runtime::PhysicalRecoveryAttemptBindingFact, worth_store::physical_runtime::PhysicalRecoveryOperationFate, worth_store::physical_runtime::RecordAppendBatch};

pub(crate) use {worth_store_testkit_store::physical_record_journeys::durability};
pub(crate) use {worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::physical_record_journeys::media, worth_store_testkit_store::physical_record_journeys::success};

pub(crate) fn initialized(root: &std::path::Path) -> worth_store::physical_runtime::ServingPhysicalRuntime {
    let media = media(root);
    let policy = durability(&media);
    let (format, placement, access) = configuration();
    success(
        media.initialize_record_store(worth_store::physical_runtime::PhysicalRecordInitialization::new(
            format, placement, access, policy,
        )),
    )
}