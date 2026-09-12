use std::io::{Seek, SeekFrom, Write};

use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    PhysicalManifestCapacityTransition, PhysicalMutationIdempotencyMaterial,
    PhysicalMutationPreparationDenial, PhysicalRecordInitialization, RecordAppendBatch,
    RecordAppendDenial, RecordByteLimit, RecordReadLimits, RecordServingTerminalPosture,
    RecordStreamFailureKind, RecordWriteSource, RecordWriteSourceError,
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
    let path = root.join("families/records/extents/extent-0000000000000001-0000000000000001.data");
    let mut file = std::fs::OpenOptions::new().write(true).open(path).unwrap();
    const EXTENT_METADATA_BYTES: usize = 64;
    const EXTENT_PAYLOAD_CAPACITY: usize =
        16_384 - super::super::durable_frame_oracle::HEADER_BYTES - EXTENT_METADATA_BYTES;
    file.seek(SeekFrom::Start(
        (16_384 + super::super::durable_frame_oracle::HEADER_BYTES + EXTENT_METADATA_BYTES + 8)
            as u64,
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
    assert_eq!(failure.kind(), RecordStreamFailureKind::ArtifactDamaged);
    assert_eq!(failure.completed_range(), 0..EXTENT_PAYLOAD_CAPACITY as u64);
    assert_eq!(session.observation().generation_checks(), 2);
    assert_eq!(session.observation().generation_rejections(), 0);
    drop(session);
    let before_retry = serving.media_counters();
    assert!(matches!(
        durable_publication::prepare_single(
            &serving.record_submission(),
            placement,
            PhysicalManifestCapacityTransition::PreserveCurrent,
            PhysicalMutationIdempotencyMaterial::new([205; 32]),
            RecordAppendBatch::builder()
                .push_source(PanicOnReadSource)
                .build()
                .unwrap(),
        )
        .into_raw(),
        TransitionOutcome::Denied(PhysicalMutationPreparationDenial::RecordAppend(
            RecordAppendDenial::ServingRequiresInspection
        ))
    ));
    assert_eq!(serving.media_counters(), before_retry);
    assert_eq!(
        serving.abort().records().posture(),
        RecordServingTerminalPosture::InspectionRequired
    );
}

struct PanicOnReadSource;

impl RecordWriteSource for PanicOnReadSource {
    fn declared_length(&self) -> u64 {
        5
    }

    fn read_next(&mut self, _target: &mut [u8]) -> Result<usize, RecordWriteSourceError> {
        panic!("inspection-required preparation must not consume its payload")
    }
}
