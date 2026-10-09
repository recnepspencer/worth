use worth_store::physical_runtime::{
    ExternalPhysicalRecordLocator, PhysicalRecordInitialization, PhysicalWorkCounterSnapshot,
    PhysicalWorkCounterStage, PhysicalWorkEffectFate, PhysicalWorkOperationFamily,
    PhysicalWorkSignalFamily, PhysicalWritebackCounterSnapshot, RecordAppendBatch,
    ServingPhysicalRuntime,
};

use super::super::{
    durable_publication::publish_single,
    media,
    scenario_configuration::dense_configuration,
    stream_fixture::{hex, pattern_digest, PatternSource},
    success,
};

const EXTENT_METADATA_BYTES: u64 = 64;
const EXTENT_FRAME_OVERHEAD: u64 =
    super::super::durable_frame_oracle::HEADER_BYTES as u64 + EXTENT_METADATA_BYTES;

#[test]
fn mixed_batch_streams_extent_and_a_fresh_process_reads_with_seventeen_widths() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let (format, placement, access) = dense_configuration(4);
    let serving = success(initialize_record_store!(media(&root), |durability| {
        PhysicalRecordInitialization::new(format, placement, access, durability)
    }));
    let logical_bytes = 1_048_613_u64;
    let batch = RecordAppendBatch::builder()
        .push_bytes(b"inline")
        .push_source(PatternSource::exact(logical_bytes))
        .build()
        .unwrap();
    let writeback_baseline = ExtentWritebackBaseline::capture(&serving);
    let published = publish_single(
        &serving,
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([180; 32]),
        batch,
    );
    let observation = published.settled_members()[0].observation();
    let chunks = logical_bytes.div_ceil(16_384 - EXTENT_FRAME_OVERHEAD);
    assert_eq!(observation.extent_artifacts(), 1);
    assert_eq!(observation.bytes_requested(), logical_bytes + 6);
    assert_eq!(observation.bytes_completed(), logical_bytes + 6);
    assert_eq!(observation.transfer_count(), chunks + 1);
    assert_eq!(observation.peak_transfer_width(), 16_384);
    assert_eq!(observation.peak_scratch_bytes(), 16_384);
    assert_eq!(observation.explicit_copy_count(), chunks + 2);
    assert_eq!(observation.copied_bytes(), logical_bytes * 2 + 6);
    assert_extent_writeback_evidence(&serving, writeback_baseline, chunks);
    let locator = ExternalPhysicalRecordLocator::new(
        serving.store_identity(),
        published.settled_members()[0].record_id(1).unwrap(),
    );
    serving.close();
    assert_extent_artifact_and_fresh_read(&root, locator, logical_bytes, chunks);
}

fn assert_extent_artifact_and_fresh_read(
    root: &std::path::Path,
    locator: ExternalPhysicalRecordLocator,
    logical_bytes: u64,
    chunks: u64,
) {
    let extent = root.join("families/records/arenas/arena-0000000000000001.data");
    let arena = std::fs::read(&extent).unwrap();
    let last_chunk =
        super::super::durable_frame_oracle::first_arena_chunk_offset(&arena, chunks - 1);
    let last_payload = logical_bytes - (chunks - 1) * (16_384 - EXTENT_FRAME_OVERHEAD);
    assert_eq!(
        std::fs::metadata(extent).unwrap().len(),
        last_chunk + EXTENT_FRAME_OVERHEAD + last_payload
    );
    let output = super::super::child_process::run_child(
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

struct ExtentWritebackBaseline {
    writebacks: PhysicalWritebackCounterSnapshot,
    work: PhysicalWorkCounterSnapshot,
    causal_records: usize,
    causal_overflow: u64,
}

impl ExtentWritebackBaseline {
    fn capture(serving: &ServingPhysicalRuntime) -> Self {
        Self {
            writebacks: serving.residency_observation().writebacks(),
            work: serving.physical_work_counters(),
            causal_records: serving.physical_work_observer().causal().records().len(),
            causal_overflow: serving.physical_work_observer().causal().overflow(),
        }
    }
}

fn assert_extent_writeback_evidence(
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
            PhysicalWorkOperationFamily::ArtifactPublication,
            PhysicalWorkCounterStage::Terminal
        ) - before.work.count(
            PhysicalWorkOperationFamily::ArtifactPublication,
            PhysicalWorkCounterStage::Terminal
        ) >= expected_frames
    );
    assert_eq!(
        work_after.count(
            PhysicalWorkOperationFamily::ArtifactRangeWrite,
            PhysicalWorkCounterStage::Terminal,
        ) - before.work.count(
            PhysicalWorkOperationFamily::ArtifactRangeWrite,
            PhysicalWorkCounterStage::Terminal,
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
        .filter(|record| record.operation() == PhysicalWorkOperationFamily::ArtifactRangeWrite)
        .collect::<Vec<_>>();
    assert_eq!(writeback_records.len() as u64, expected_writebacks);
    let signal_bindings = serving.physical_signal_aspect_binding_observations();
    for writeback in writeback_records {
        assert_eq!(
            writeback.effect_fate(),
            PhysicalWorkEffectFate::WriteCompleted
        );
        assert!(writeback.backend_operation().is_some());
        assert!(signal_bindings.iter().any(|binding| {
            binding.digest() == writeback.signal_binding()
                && binding
                    .families()
                    .contains(PhysicalWorkSignalFamily::ExactWriteback)
        }));
    }
}
