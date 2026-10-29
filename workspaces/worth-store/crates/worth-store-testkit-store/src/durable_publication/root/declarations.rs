pub use {worth_store::physical_runtime::AdmittedRecordPlacementPolicy, worth_store::physical_runtime::CompletedPhysicalRootPublication, worth_store::physical_runtime::PhysicalManifestCapacityTransition, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::RecordAppendBatch, worth_store::physical_runtime::ServingPhysicalRuntime};

pub fn publish_single(
    serving: &ServingPhysicalRuntime,
    placement: AdmittedRecordPlacementPolicy,
    material: PhysicalMutationIdempotencyMaterial,
    batch: RecordAppendBatch,
) -> CompletedPhysicalRootPublication {
    publish_single_with_manifest_capacity_transition(
        serving,
        placement,
        worth_store::physical_runtime::PhysicalManifestCapacityTransition::PreserveCurrent,
        material,
        batch,
    )
}

pub fn publish_single_with_manifest_capacity_transition(
    serving: &ServingPhysicalRuntime,
    placement: AdmittedRecordPlacementPolicy,
    manifest_capacity_transition: PhysicalManifestCapacityTransition,
    material: PhysicalMutationIdempotencyMaterial,
    batch: RecordAppendBatch,
) -> CompletedPhysicalRootPublication {
    serving.certification_publish_single_durable_mutation(
        placement,
        manifest_capacity_transition,
        material,
        batch,
    )
}