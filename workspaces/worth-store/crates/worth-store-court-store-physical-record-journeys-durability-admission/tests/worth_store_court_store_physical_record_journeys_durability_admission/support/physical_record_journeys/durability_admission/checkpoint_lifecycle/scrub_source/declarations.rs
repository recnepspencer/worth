pub(crate) use {std::time::Duration, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::ServingPhysicalRuntime, worth_store_testkit_store::physical_record_journeys::durability_admission::checkpoint_lifecycle::checkpoint_request, worth_store_testkit_store::physical_record_journeys::durability_admission::checkpoint_lifecycle::serving_with_durable_wal, worth_store_testkit_store::physical_record_journeys::durability_admission::checkpoint_lifecycle::start};
pub(crate) use {worth_store::physical_runtime::ManagedPhysicalIntegrityScrubProgress as Progress, worth_store::physical_runtime::ManagedPhysicalIntegrityScrubRequest as Request, worth_store::physical_runtime::PhysicalIntegrityScrubTarget as Target};
pub(crate) use {worth_store_physical_format::PhysicalArtifactReadTarget, worth_store_physical_format::PhysicalCheckpointIdentity};
pub(crate) use {worth_store_physical_integrity::CheckpointStreamHeaderScopeIdentity, worth_store_physical_integrity::IndeterminatePhysicalIntegrityCause, worth_store_physical_integrity::PhysicalArtifactScope, worth_store_physical_integrity::PhysicalByteRange, worth_store_physical_integrity::PhysicalIntegrityObservationOutcome as Observation, worth_store_physical_integrity::PhysicalIntegrityRejection as Rejection};

pub(crate) fn publish(serving: &ServingPhysicalRuntime, key: u8) -> PhysicalCheckpointIdentity {
    match start(serving, checkpoint_request(key)).wait() {
        worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(completed) => completed.basis().identity(),
        outcome => panic!("real checkpoint publication: {outcome:?}"),
    }
}

pub(crate) fn header(identity: PhysicalCheckpointIdentity) -> Target {
    worth_store::physical_runtime::PhysicalIntegrityScrubTarget::new(
        worth_store_physical_format::PhysicalArtifactReadTarget::Checkpoint(identity),
        worth_store_physical_integrity::PhysicalArtifactScope::checkpoint_stream_header(
            worth_store_physical_integrity::CheckpointStreamHeaderScopeIdentity::known(identity),
            worth_store_physical_integrity::PhysicalByteRange::new(0, 164).unwrap(),
        ),
    )
    .unwrap()
}