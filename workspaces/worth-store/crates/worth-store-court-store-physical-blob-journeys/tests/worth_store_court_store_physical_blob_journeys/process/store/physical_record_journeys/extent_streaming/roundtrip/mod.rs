#[allow(unused_imports)]
use crate::physical_record_journeys::extent_streaming::roundtrip::*;
use {
    worth_store::physical_runtime::ExternalPhysicalRecordLocator,
    worth_store::physical_runtime::PhysicalRecordInitialization,
    worth_store::physical_runtime::PhysicalWorkCounterSnapshot,
    worth_store::physical_runtime::PhysicalWorkCounterStage,
    worth_store::physical_runtime::PhysicalWorkEffectFate,
    worth_store::physical_runtime::PhysicalWorkOperationFamily,
    worth_store::physical_runtime::PhysicalWorkSignalFamily,
    worth_store::physical_runtime::PhysicalWritebackCounterSnapshot,
    worth_store::physical_runtime::RecordAppendBatch,
    worth_store::physical_runtime::ServingPhysicalRuntime,
};

use {
    worth_store_testkit_store::durable_publication::root::publish_single,
    worth_store_testkit_store::physical_record_journeys::media,
    worth_store_testkit_store::physical_record_journeys::scenario_configuration::dense_configuration,
    worth_store_testkit_store::physical_record_journeys::success,
    worth_store_testkit_store::stream_fixture::hex,
    worth_store_testkit_store::stream_fixture::pattern_digest,
    worth_store_testkit_store::stream_fixture::PatternSource,
};

#[test]
fn mixed_batch_streams_extent_and_a_fresh_process_reads_with_seventeen_widths() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let (format, placement, access) = dense_configuration(4);
    let serving = success(initialize_record_store!(media(&root), |durability| {
        worth_store::physical_runtime::PhysicalRecordInitialization::new(
            format, placement, access, durability,
        )
    }));
    let logical_bytes = 1_048_613_u64;
    let batch = worth_store::physical_runtime::RecordAppendBatch::builder()
        .push_bytes(b"inline")
        .push_source(worth_store_testkit_store::stream_fixture::PatternSource::exact(logical_bytes))
        .build()
        .unwrap();
    let writeback_baseline = crate::physical_record_journeys::extent_streaming::roundtrip::ExtentWritebackBaseline::capture(&serving);
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
    let locator = worth_store::physical_runtime::ExternalPhysicalRecordLocator::new(
        serving.store_identity(),
        published.settled_members()[0].record_id(1).unwrap(),
    );
    serving.close();
    assert_extent_artifact_and_fresh_read(&root, locator, logical_bytes, chunks);
}
