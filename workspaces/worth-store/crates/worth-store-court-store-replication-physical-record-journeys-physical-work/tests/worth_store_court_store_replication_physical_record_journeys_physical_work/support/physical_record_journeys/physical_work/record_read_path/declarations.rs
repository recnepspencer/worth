pub(crate) use worth_store_physical_backend::MediaOperationRole;
pub(crate) use {
    worth_store::physical_runtime::PhysicalWorkCounterSnapshot,
    worth_store::physical_runtime::PhysicalWorkCounterStage,
    worth_store::physical_runtime::PhysicalWorkOperationFamily,
    worth_store::physical_runtime::RecordAppendBatch,
    worth_store::physical_runtime::RecordByteLimit,
    worth_store::physical_runtime::RecordReadLimits,
    worth_store::physical_runtime::RecordReadObservation,
};

pub(crate) use worth_store_testkit_store::physical_work::record_read_signal_cleanup::await_read_signal_cleanup;
pub(crate) use {
    worth_store_testkit_store::durable_publication,
    worth_store_testkit_store::physical_record_journeys::read_record,
    worth_store_testkit_store::physical_record_journeys::serving_from_open,
};
pub(crate) use {
    worth_store_testkit_store::physical_record_journeys::configuration,
    worth_store_testkit_store::physical_record_journeys::serving_from_initialization,
};

pub(crate) const PAYLOAD: &[u8] = b"canonical cold and hot record read";

pub(crate) fn assert_same_semantic_observation(
    left: RecordReadObservation,
    right: RecordReadObservation,
) {
    assert_eq!(left.touched_segments(), right.touched_segments());
    assert_eq!(left.touched_pages(), right.touched_pages());
    assert_eq!(left.touched_extents(), right.touched_extents());
    assert_eq!(left.payload_bytes(), right.payload_bytes());
    assert_eq!(left.bytes_requested(), right.bytes_requested());
    assert_eq!(left.transfer_count(), right.transfer_count());
    assert_eq!(left.peak_transfer_width(), right.peak_transfer_width());
    assert_eq!(left.explicit_copy_count(), right.explicit_copy_count());
    assert_eq!(left.copied_bytes(), right.copied_bytes());
    assert_eq!(left.generation_checks(), right.generation_checks());
    assert_eq!(left.generation_rejections(), right.generation_rejections());
    assert_eq!(left.peak_scratch_bytes(), right.peak_scratch_bytes());
    assert_eq!(left.manifest_blocks(), right.manifest_blocks());
    assert_eq!(left.manifest_comparisons(), right.manifest_comparisons());
    assert_eq!(left.manifest_bytes(), right.manifest_bytes());
}

pub(crate) fn media_delta(
    before: worth_store_physical_backend::MediaCounterSnapshot,
    after: worth_store_physical_backend::MediaCounterSnapshot,
) -> u64 {
    media_role_delta(
        before,
        after,
        worth_store_physical_backend::MediaOperationRole::PositionedRead,
    )
}

pub(crate) fn media_role_delta(
    before: worth_store_physical_backend::MediaCounterSnapshot,
    after: worth_store_physical_backend::MediaCounterSnapshot,
    role: MediaOperationRole,
) -> u64 {
    after.attempts_for(role) - before.attempts_for(role)
}

pub(crate) fn work_delta(
    before: PhysicalWorkCounterSnapshot,
    after: PhysicalWorkCounterSnapshot,
    stage: PhysicalWorkCounterStage,
) -> u64 {
    [
        worth_store::physical_runtime::PhysicalWorkOperationFamily::ArtifactMetadataRead,
        worth_store::physical_runtime::PhysicalWorkOperationFamily::ArtifactRangeRead,
    ]
    .into_iter()
    .map(|family| after.count(family, stage) - before.count(family, stage))
    .sum()
}

pub(crate) fn family_work_delta(
    before: PhysicalWorkCounterSnapshot,
    after: PhysicalWorkCounterSnapshot,
    family: PhysicalWorkOperationFamily,
    stage: PhysicalWorkCounterStage,
) -> u64 {
    after.count(family, stage) - before.count(family, stage)
}
