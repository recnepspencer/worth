pub(crate) use {worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationOutcome, worth_store::physical_runtime::ServingPhysicalRuntime, worth_signal::facade::TemporalDuration, worth_proof::TransitionOutcome, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::completed, worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::extent_frame_fixture::extent_payload, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::prepare, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::read_record, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::request, worth_store_testkit_store::physical_record_journeys::serving_from_initialization};
pub(crate) use {worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_store::physical_runtime::PhysicalExtentCopyPhase, worth_store::physical_runtime::PhysicalExtentCopyResolutionProgress, worth_store::physical_runtime::PhysicalMutationIndeterminateStage};

pub(crate) fn checkpoint_copy_cancellation(
    serving: &ServingPhysicalRuntime,
    key: u8,
) -> worth_store::physical_runtime::CompletedPhysicalCheckpoint {
    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([key; 32]),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::at(
            worth_signal::facade::TemporalDuration::temporal_duration(30_000).unwrap(),
        ),
    );
    let worth_proof::TransitionOutcome::Success(handle) =
        serving.checkpoints().start(request).into_raw()
    else {
        panic!("copy cancellation checkpoint must admit");
    };
    let worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(checkpoint) =
        handle.wait()
    else {
        panic!("copy cancellation checkpoint must complete");
    };
    checkpoint
}

/// The newest generation `WritebackFenceDrain` admitted in the first arenas of
/// this residency incarnation; only copy cancellations drain in these journeys.
pub(crate) fn newest_arena_generation(
    serving: &ServingPhysicalRuntime,
) -> worth_store_physical_format::ExtentAllocationGeneration {
    let residency = serving.certification_physical_residency();
    (1..=8)
        .map(|arena| {
            let arena = worth_store_physical_format::ExtentArenaId::new(arena).unwrap();
            residency.active_arena_generation(
                worth_store_physical_format::ExtentArenaRange::new(arena, 0, u64::MAX / 2).unwrap(),
            )
        })
        .max()
        .unwrap()
}

/// Copies `persisted` to a candidate extent, durably cancels it, checkpoints
/// the cancellation and finalizes, publishing no root. Returns the generation
/// the cancelled destination re-entered the allocator at.
pub(crate) fn cancel_copy_without_root_publication(
    serving: &ServingPhysicalRuntime,
    persisted: worth_store_physical_format::PersistedRecordIdentity,
    material: u8,
) -> worth_store_physical_format::ExtentAllocationGeneration {
    let (_, placement, _) = configuration();
    let root_before = checkpoint_copy_cancellation(serving, material.wrapping_add(2))
        .basis()
        .source()
        .root()
        .generation();
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new([material; 32]))
        .unwrap();
    let Ok(mut progress) = submission
        .certification_begin_selected_extent_copy(placement, request(key), persisted)
        .unwrap()
    else {
        panic!("actual selected extent must admit a bounded copy");
    };
    for _ in 0..128 {
        if progress.phase == PhysicalExtentCopyPhase::ReadyForAdoption {
            break;
        }
        progress = submission.advance_extent_copy().unwrap();
    }
    assert_eq!(progress.phase, PhysicalExtentCopyPhase::ReadyForAdoption);
    let before = newest_arena_generation(serving);
    let PhysicalExtentCopyResolutionProgress::AwaitingCheckpoint { resolution_lsn } =
        submission.cancel_extent_copy().unwrap()
    else {
        panic!("a durable copy intent must await checkpoint coverage of its cancellation");
    };
    let checkpoint = checkpoint_copy_cancellation(serving, material.wrapping_add(1));
    let root = checkpoint.basis().source().root().generation();
    assert_eq!(
        root, root_before,
        "no root is published between copy and reuse"
    );
    assert_eq!(
        submission.finalize_extent_copy(&checkpoint).unwrap(),
        PhysicalExtentCopyResolutionProgress::Resolved
    );
    let reused = newest_arena_generation(serving);
    assert!(reused > before);
    assert_eq!(reused.root_publication_generation(), root);
    assert!(reused.extent_fence_seq() > resolution_lsn);
    reused
}
