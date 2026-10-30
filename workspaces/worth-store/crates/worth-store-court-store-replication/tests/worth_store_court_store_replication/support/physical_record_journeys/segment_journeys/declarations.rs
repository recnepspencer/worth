pub(crate) use std::path::Path;

pub(crate) use {
    worth_store::physical_runtime::ExternalPhysicalRecordLocator,
    worth_store::physical_runtime::FramePortCounterSnapshot,
    worth_store::physical_runtime::ManifestEntryCapacity,
    worth_store::physical_runtime::PageFillPercent,
    worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial,
    worth_store::physical_runtime::PhysicalRecordInitialization,
    worth_store::physical_runtime::PhysicalRecordPlacementPolicy,
    worth_store::physical_runtime::PhysicalWorkCounterSnapshot,
    worth_store::physical_runtime::PhysicalWorkCounterStage,
    worth_store::physical_runtime::PhysicalWorkEffectFate,
    worth_store::physical_runtime::PhysicalWorkOperationFamily,
    worth_store::physical_runtime::PhysicalWorkSignalFamily,
    worth_store::physical_runtime::PhysicalWritebackCounterSnapshot,
    worth_store::physical_runtime::RecordAppendBatch,
    worth_store::physical_runtime::RecordByteLimit,
    worth_store::physical_runtime::RecordReadLimits,
    worth_store::physical_runtime::RootPublicationPhysicalMutationMember,
    worth_store::physical_runtime::SegmentPageCount,
    worth_store::physical_runtime::ServingPhysicalRuntime,
};
pub(crate) use {
    worth_store_offline_verifier::OfflineAllocationClass,
    worth_store_offline_verifier::OfflineRecordPlacement,
};
pub(crate) use {
    worth_store_physical_backend::MediaCounterSnapshot,
    worth_store_physical_backend::MediaOperationRole,
};
pub(crate) use {
    worth_store_physical_format::store_namespace::StableStoreIdentity,
    worth_store_physical_format::PhysicalRecordFormatDeclaration,
};

pub(crate) use {
    worth_store_testkit_store::durable_publication::root::publish_single,
    worth_store_testkit_store::physical_record_journeys::media,
    worth_store_testkit_store::physical_record_journeys::read_record,
    worth_store_testkit_store::physical_record_journeys::scenario_configuration::dense_configuration,
    worth_store_testkit_store::physical_record_journeys::success,
    worth_store_testkit_store::stream_fixture::hex,
};

pub(crate) struct SegmentWritebackBaseline {
    pub(crate) media: MediaCounterSnapshot,
    pub(crate) writebacks: PhysicalWritebackCounterSnapshot,
    pub(crate) frames: FramePortCounterSnapshot,
    pub(crate) work: PhysicalWorkCounterSnapshot,
    pub(crate) wal_frames: u64,
    pub(crate) causal_records: usize,
    pub(crate) causal_overflow: u64,
}

impl SegmentWritebackBaseline {
    pub(crate) fn capture(serving: &ServingPhysicalRuntime) -> Self {
        Self {
            media: serving.media_counters(),
            writebacks: serving.residency_observation().writebacks(),
            frames: serving.certification_frame_port_observer().snapshot(),
            work: serving.physical_work_counters(),
            wal_frames: serving
                .record_submission()
                .wal_observation()
                .expect("the installed WAL owner must remain observable")
                .appended_frames(),
            causal_records: serving.physical_work_observer().causal().records().len(),
            causal_overflow: serving.physical_work_observer().causal().overflow(),
        }
    }
}

pub(crate) fn assert_segment_frame_and_media_evidence(
    serving: &ServingPhysicalRuntime,
    before: &SegmentWritebackBaseline,
    new_segment_candidates: u64,
    root_candidate_artifacts: u64,
) {
    let media = serving.media_counters();
    let residency = serving.residency_observation();
    let frames = serving.certification_frame_port_observer().snapshot();
    let candidate_frames = frames.candidate_frames() - before.frames.candidate_frames();
    let writeback_frames = residency.writebacks().attempts() - before.writebacks.attempts();
    let wal_frames = serving
        .record_submission()
        .wal_observation()
        .expect("the installed WAL owner must remain observable")
        .appended_frames()
        - before.wal_frames;
    assert_eq!(media.replacements(), before.media.replacements() + 3);
    assert_eq!(
        frames.declared_candidate_frames() - before.frames.declared_candidate_frames(),
        candidate_frames
    );
    assert_eq!(
        frames.candidate_publications() - before.frames.candidate_publications(),
        candidate_frames
    );
    assert_eq!(candidate_frames, 15);
    assert_eq!(wal_frames, 1);
    assert_eq!(new_segment_candidates, 4);
    assert_eq!(writeback_frames, 4);
    assert_eq!(root_candidate_artifacts, 7);
    assert_eq!(
        candidate_frames,
        new_segment_candidates + writeback_frames + root_candidate_artifacts
    );
    assert_eq!(frames.writebacks() - before.frames.writebacks(), 4);
    assert_eq!(
        residency.writebacks().exact_receipts() - before.writebacks.exact_receipts(),
        4
    );
    assert_eq!(
        residency.writebacks().retryable() - before.writebacks.retryable(),
        0
    );
    assert_eq!(
        residency.writebacks().inspection_required() - before.writebacks.inspection_required(),
        0
    );
    assert_eq!(
        media.attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite)
            - before
                .media
                .attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite),
        candidate_frames + wal_frames
    );
    assert_eq!(residency.counters().dirty_frames(), 0);
    assert_eq!(residency.counters().candidate_frames(), 0);
    assert_eq!(residency.counters().active_writeback_claims(), 0);
}

pub(crate) fn assert_segment_work_and_signal_evidence(
    serving: &ServingPhysicalRuntime,
    before: &SegmentWritebackBaseline,
) {
    assert_eq!(
        serving.physical_work_counters().count(
            worth_store::physical_runtime::PhysicalWorkOperationFamily::ArtifactRangeWrite,
            worth_store::physical_runtime::PhysicalWorkCounterStage::Terminal,
        ) - before.work.count(
            worth_store::physical_runtime::PhysicalWorkOperationFamily::ArtifactRangeWrite,
            worth_store::physical_runtime::PhysicalWorkCounterStage::Terminal,
        ),
        4
    );
    assert_eq!(
        serving.physical_work_observer().causal().overflow(),
        before.causal_overflow
    );
    let causal = serving.physical_work_observer().causal().records();
    let writebacks = causal[before.causal_records..]
        .iter()
        .filter(|record| {
            record.operation()
                == worth_store::physical_runtime::PhysicalWorkOperationFamily::ArtifactRangeWrite
        })
        .collect::<Vec<_>>();
    assert_eq!(writebacks.len(), 4);
    let signal_bindings = serving.physical_signal_aspect_binding_observations();
    for writeback in writebacks {
        assert_eq!(
            writeback.effect_fate(),
            worth_store::physical_runtime::PhysicalWorkEffectFate::WriteCompleted
        );
        assert!(writeback.backend_operation().is_some());
        assert!(signal_bindings.iter().any(|binding| {
            binding.digest() == writeback.signal_binding()
                && binding.families().contains(
                    worth_store::physical_runtime::PhysicalWorkSignalFamily::ExactWriteback,
                )
        }));
    }
}

pub(crate) fn assert_segment_artifact_lengths(root: &Path) {
    for segment in 1..=4_u64 {
        let path = root.join(format!(
            "families/records/segments/segment-{segment:016x}-0000000000000001.pages"
        ));
        assert_eq!(std::fs::metadata(path).unwrap().len(), 32_768);
    }
}

pub(crate) fn assert_inline_placement_truth(root: &Path, format: PhysicalRecordFormatDeclaration) {
    let root_manifest =
        worth_store_testkit_store::physical_record_journeys::manifest_fixture::decode_routing_tree(
            root, 2, format, 128,
        );
    for (index, entry) in root_manifest.placements().iter().enumerate() {
        let worth_store_offline_verifier::OfflineRecordPlacement::Inline {
            segment,
            page,
            slot,
            ..
        } = entry
        else {
            panic!("the rollover fixture contains only inline records")
        };
        assert_eq!(*segment, index as u64 / 4 + 1);
        assert_eq!(*page, index as u64 / 2 + 1);
        assert_eq!(*slot, index as u16 % 2 + 1);
    }
}

pub(crate) fn segment_reader_request(
    store_identity: StableStoreIdentity,
    member: &RootPublicationPhysicalMutationMember,
) -> String {
    let order = [14_usize, 0, 7, 3, 12, 1, 9, 5, 13, 2, 11, 6, 4, 10, 8];
    order
        .iter()
        .map(|index| {
            let locator = worth_store::physical_runtime::ExternalPhysicalRecordLocator::new(
                store_identity,
                member.record_id(*index).unwrap(),
            );
            format!("{index}:{}", hex(&locator.encode()))
        })
        .collect::<Vec<_>>()
        .join(";")
}

pub(crate) fn assert_fresh_process_records(root: &Path, request: &str) {
    let order = [14_usize, 0, 7, 3, 12, 1, 9, 5, 13, 2, 11, 6, 4, 10, 8];
    let output =
        worth_store_testkit_store::physical_record_journeys::child_process::invocation::run_child(
            "segment_reader",
            root,
            Some(request),
        );
    for index in order {
        assert!(
            output
                .lines()
                .any(|line| line == format!("C5_SEGMENT {index} {index} 3000 1 1")),
            "C5_PREDICATE:identity-placement-seam"
        );
    }
}
