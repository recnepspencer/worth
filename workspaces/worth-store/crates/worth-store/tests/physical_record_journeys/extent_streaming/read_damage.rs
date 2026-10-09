use std::io::{Seek, SeekFrom, Write};

use worth_store::physical_runtime::{
    PhysicalRecordInitialization, RecordAppendBatch, RecordByteLimit, RecordReadLimits,
    RecordServingTerminalPosture, RecordStreamFailureKind,
};

use super::super::{
    durable_publication, media, scenario_configuration::dense_configuration,
    stream_fixture::PatternSource, success,
};

#[test]
fn streamed_read_damage_retains_the_completed_logical_range() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let (format, placement, access) = dense_configuration(4);
    let serving = success(initialize_record_store!(media(&root), |durability| {
        PhysicalRecordInitialization::new(format, placement, access, durability)
    }));
    let published = durable_publication::publish_single(
        &serving,
        placement,
        durable_publication::certification_material("extent-streamed-read-damage", 1),
        RecordAppendBatch::builder()
            .push_source(PatternSource::exact(40_000))
            .build()
            .unwrap(),
    );
    let path = root.join("families/records/arenas/arena-0000000000000001.data");
    let arena = std::fs::read(&path).unwrap();
    let second_chunk = super::super::durable_frame_oracle::first_arena_chunk_offset(&arena, 1);
    let mut file = std::fs::OpenOptions::new().write(true).open(path).unwrap();
    const EXTENT_METADATA_BYTES: usize = 64;
    const EXTENT_PAYLOAD_CAPACITY: usize =
        16_384 - super::super::durable_frame_oracle::HEADER_BYTES - EXTENT_METADATA_BYTES;
    file.seek(SeekFrom::Start(
        second_chunk
            + (super::super::durable_frame_oracle::HEADER_BYTES + EXTENT_METADATA_BYTES + 8) as u64,
    ))
    .unwrap();
    file.write_all(&[0xa5]).unwrap();
    file.sync_all().unwrap();
    assert!(
        serving
            .certification_physical_residency()
            .drain_unpinned_clean_frames()
            > 0
    );
    let mut session = serving
        .records()
        .expect("read protection admission")
        .open(
            published.settled_members()[0].record_id(0).unwrap(),
            RecordReadLimits::new(RecordByteLimit::new(40_000).unwrap()),
        )
        .unwrap();
    let mut first = vec![0_u8; EXTENT_PAYLOAD_CAPACITY];
    assert_eq!(
        session.read_next(&mut first).unwrap(),
        EXTENT_PAYLOAD_CAPACITY
    );
    let failure = session.read_next(&mut [0_u8; 1]).unwrap_err();
    // Data-frame checksum damage localizes to the dependent range; only
    // root/routing/manifest damage revokes global serving health (spec C.11
    // "Read and verification").
    assert_eq!(
        failure.kind(),
        RecordStreamFailureKind::SelectedDataFrameChecksumDamaged
    );
    assert_eq!(failure.completed_range(), 0..EXTENT_PAYLOAD_CAPACITY as u64);
    assert_eq!(session.observation().generation_checks(), 2);
    assert_eq!(session.observation().generation_rejections(), 0);
    drop(session);
    let mut reread = serving
        .records()
        .expect("local damage retains read admission")
        .open(
            published.settled_members()[0].record_id(0).unwrap(),
            RecordReadLimits::new(RecordByteLimit::new(40_000).unwrap()),
        )
        .unwrap();
    assert_eq!(
        reread.read_next(&mut first).unwrap(),
        EXTENT_PAYLOAD_CAPACITY
    );
    assert_eq!(
        reread.read_next(&mut [0_u8; 1]).unwrap_err().kind(),
        RecordStreamFailureKind::SelectedDataFrameChecksumDamaged,
        "the damaged range stays damaged; it is never served"
    );
    drop(reread);
    durable_publication::publish_single(
        &serving,
        placement,
        durable_publication::certification_material("extent-streamed-read-damage", 2),
        RecordAppendBatch::builder()
            .push_source(PatternSource::exact(5))
            .build()
            .unwrap(),
    );
    assert_eq!(
        serving.abort().records().posture(),
        RecordServingTerminalPosture::NoInspectionRequired
    );
}
