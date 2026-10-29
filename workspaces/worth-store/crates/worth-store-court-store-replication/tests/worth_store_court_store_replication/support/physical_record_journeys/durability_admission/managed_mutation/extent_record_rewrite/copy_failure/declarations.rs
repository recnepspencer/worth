pub(crate) use {worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationOutcome, worth_store::physical_runtime::ServingPhysicalRuntime, worth_signal::facade::TemporalDuration, worth_proof::TransitionOutcome, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::completed, worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::extent_frame_fixture::extent_payload, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::prepare, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::read_record, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::request, worth_store_testkit_store::physical_record_journeys::serving_from_initialization};
pub(crate) use {worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_store::physical_runtime::PhysicalExtentCopyPhase, worth_store::physical_runtime::PhysicalExtentCopyResolutionProgress, worth_store::physical_runtime::PhysicalMutationIndeterminateStage};

pub(crate) fn checkpoint_copy_cancellation(
    serving: &ServingPhysicalRuntime,
) -> worth_store::physical_runtime::CompletedPhysicalCheckpoint {
    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([234; 32]),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::at(worth_signal::facade::TemporalDuration::temporal_duration(30_000).unwrap()),
    );
    let worth_proof::TransitionOutcome::Success(handle) = serving.checkpoints().start(request).into_raw() else {
        panic!("copy cancellation checkpoint must admit");
    };
    let worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(checkpoint) = handle.wait() else {
        panic!("copy cancellation checkpoint must complete");
    };
    checkpoint
}