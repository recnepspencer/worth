pub(crate) use {worth_store::physical_runtime::ExternalPhysicalRecordLocator, worth_store::physical_runtime::PhysicalRecordInitialization, worth_store::physical_runtime::PhysicalWorkCounterSnapshot, worth_store::physical_runtime::PhysicalWorkCounterStage, worth_store::physical_runtime::PhysicalWorkEffectFate, worth_store::physical_runtime::PhysicalWorkOperationFamily, worth_store::physical_runtime::PhysicalWorkSignalFamily, worth_store::physical_runtime::PhysicalWritebackCounterSnapshot, worth_store::physical_runtime::RecordAppendBatch, worth_store::physical_runtime::ServingPhysicalRuntime};

pub(crate) use {worth_store_testkit_store::durable_publication::root::publish_single, worth_store_testkit_store::physical_record_journeys::media, worth_store_testkit_store::physical_record_journeys::scenario_configuration::dense_configuration, worth_store_testkit_store::stream_fixture::hex, worth_store_testkit_store::stream_fixture::pattern_digest, worth_store_testkit_store::stream_fixture::PatternSource, worth_store_testkit_store::physical_record_journeys::success};

pub(crate) const EXTENT_METADATA_BYTES: u64 = 64;
pub(crate) const EXTENT_FRAME_OVERHEAD: u64 =
    worth_store_testkit_store::durable_frame_oracle::HEADER_BYTES as u64 + EXTENT_METADATA_BYTES;

pub(crate) fn assert_extent_artifact_and_fresh_read(
    root: &std::path::Path,
    locator: ExternalPhysicalRecordLocator,
    logical_bytes: u64,
    chunks: u64,
) {
    let extent = root.join("families/records/arenas/arena-0000000000000001.data");
    let arena = std::fs::read(&extent).unwrap();
    let last_chunk =
        worth_store_testkit_store::durable_frame_oracle::first_arena_chunk_offset(&arena, chunks - 1);
    let last_payload = logical_bytes - (chunks - 1) * (16_384 - EXTENT_FRAME_OVERHEAD);
    assert_eq!(
        std::fs::metadata(extent).unwrap().len(),
        last_chunk + EXTENT_FRAME_OVERHEAD + last_payload
    );
    let output = worth_store_testkit_store::physical_record_journeys::child_process::invocation::run_child(
        "extent_reader",
        root,
        Some(&hex(&locator.encode())),
    );
    assert!(output.contains(&format!(
        "C5_EXTENT {logical_bytes} {}",
        pattern_digest(logical_bytes)
    )));
    assert!(output.contains(&format!(
        "C5_EXTENT_OBS {logical_bytes} {logical_bytes} {} 16384 {logical_bytes} {} 0",
        chunks + 1,
        chunks + 1,
    )));
}

pub(crate) struct ExtentWritebackBaseline {
    pub(crate) writebacks: PhysicalWritebackCounterSnapshot,
    pub(crate) work: PhysicalWorkCounterSnapshot,
    pub(crate) causal_records: usize,
    pub(crate) causal_overflow: u64,
}

impl ExtentWritebackBaseline {
    pub(crate) fn capture(serving: &ServingPhysicalRuntime) -> Self {
        Self {
            writebacks: serving.residency_observation().writebacks(),
            work: serving.physical_work_counters(),
            causal_records: serving.physical_work_observer().causal().records().len(),
            causal_overflow: serving.physical_work_observer().causal().overflow(),
        }
    }
}

pub(crate) fn assert_extent_writeback_evidence(
    serving: &ServingPhysicalRuntime,
    before: ExtentWritebackBaseline,
    expected_frames: u64,
) {
    // Arena allocation candidates use admitted publication-range effects;
    // nonzero offsets no longer imply an existing dirty-frame writeback.
    let expected_writebacks = 0;
    let writebacks_after = serving.residency_observation().writebacks();
    assert_eq!(
        writebacks_after.attempts() - before.writebacks.attempts(),
        expected_writebacks
    );
    assert_eq!(
        writebacks_after.exact_receipts() - before.writebacks.exact_receipts(),
        expected_writebacks
    );
    assert_eq!(
        writebacks_after.retryable() - before.writebacks.retryable(),
        0
    );
    assert_eq!(
        writebacks_after.inspection_required() - before.writebacks.inspection_required(),
        0
    );
    let work_after = serving.physical_work_counters();
    assert!(
        work_after.count(
            worth_store::physical_runtime::PhysicalWorkOperationFamily::ArtifactPublication,
            worth_store::physical_runtime::PhysicalWorkCounterStage::Terminal
        ) - before.work.count(
            worth_store::physical_runtime::PhysicalWorkOperationFamily::ArtifactPublication,
            worth_store::physical_runtime::PhysicalWorkCounterStage::Terminal
        ) >= expected_frames
    );
    assert_eq!(
        work_after.count(
            worth_store::physical_runtime::PhysicalWorkOperationFamily::ArtifactRangeWrite,
            worth_store::physical_runtime::PhysicalWorkCounterStage::Terminal,
        ) - before.work.count(
            worth_store::physical_runtime::PhysicalWorkOperationFamily::ArtifactRangeWrite,
            worth_store::physical_runtime::PhysicalWorkCounterStage::Terminal,
        ),
        expected_writebacks
    );
    assert_eq!(
        serving.physical_work_observer().causal().overflow(),
        before.causal_overflow
    );
    let causal = serving.physical_work_observer().causal().records();
    let writeback_records = causal[before.causal_records..]
        .iter()
        .filter(|record| record.operation() == worth_store::physical_runtime::PhysicalWorkOperationFamily::ArtifactRangeWrite)
        .collect::<Vec<_>>();
    assert_eq!(writeback_records.len() as u64, expected_writebacks);
    let signal_bindings = serving.physical_signal_aspect_binding_observations();
    for writeback in writeback_records {
        assert_eq!(
            writeback.effect_fate(),
            worth_store::physical_runtime::PhysicalWorkEffectFate::WriteCompleted
        );
        assert!(writeback.backend_operation().is_some());
        assert!(signal_bindings.iter().any(|binding| {
            binding.digest() == writeback.signal_binding()
                && binding
                    .families()
                    .contains(worth_store::physical_runtime::PhysicalWorkSignalFamily::ExactWriteback)
        }));
    }
}