use std::io::{Seek, SeekFrom, Write};

use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    PhysicalManifestCapacityTransition, PhysicalMutationIdempotencyMaterial,
    PhysicalMutationPreparationDenial, RecordAppendBatch, RecordAppendDenial, RecordByteLimit,
    RecordReadLimits, RecordServingTerminalPosture, RecordStreamFailureKind,
};

use super::super::durable_publication::{self, publish_single};
use super::fixture;

#[test]
fn later_extent_damage_through_a_view_revokes_health_and_releases_read_authority() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("damaged-view");
    let (serving, placement) = fixture::initialize(&root);
    let expected = fixture::payload(40_000);
    let publication = publish_single(
        &serving,
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([173; 32]),
        RecordAppendBatch::try_from_iter([expected.as_slice()]).unwrap(),
    );
    let record = publication.settled_members()[0].record_id(0).unwrap();
    let extent =
        root.join("families/records/extents/extent-0000000000000001-0000000000000001.data");
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .open(extent)
        .unwrap();
    file.seek(SeekFrom::Start(fixture::FRAME_BYTES + 120))
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
        .open(
            record,
            RecordReadLimits::new(RecordByteLimit::new(expected.len() as u32).unwrap()),
        )
        .unwrap();

    {
        let first = session.next_chunk().unwrap().unwrap();
        assert_eq!(
            first.logical_range(),
            0..fixture::CHUNK_PAYLOAD_BYTES as u64
        );
        assert_eq!(first.bytes(), &expected[..fixture::CHUNK_PAYLOAD_BYTES]);
    }
    let failure = match session.next_chunk() {
        Err(failure) => failure,
        Ok(_) => panic!("the damaged second extent frame must fail borrowed iteration"),
    };
    assert_eq!(failure.kind(), RecordStreamFailureKind::ArtifactDamaged);
    assert_eq!(
        failure.completed_range(),
        0..fixture::CHUNK_PAYLOAD_BYTES as u64
    );
    assert_eq!(session.observation().generation_checks(), 2);
    assert_eq!(session.observation().generation_rejections(), 0);
    drop(session);

    assert_eq!(serving.observer().record_counters().read_sessions_live(), 0);
    let residency = serving.residency_observation().counters();
    assert_eq!(residency.pin_leases(), 0);
    assert_eq!(residency.pinned_frames(), 0);
    assert_eq!(residency.active_operation_bytes(), 0);
    assert!(matches!(
        durable_publication::prepare_single(
            &serving.record_submission(),
            placement,
            PhysicalManifestCapacityTransition::PreserveCurrent,
            PhysicalMutationIdempotencyMaterial::new([213; 32]),
            RecordAppendBatch::try_from_iter([b"denied after damage".as_slice()]).unwrap(),
        )
        .into_raw(),
        TransitionOutcome::Denied(PhysicalMutationPreparationDenial::RecordAppend(
            RecordAppendDenial::ServingRequiresInspection
        ))
    ));
    assert_eq!(
        serving.abort().records().posture(),
        RecordServingTerminalPosture::InspectionRequired
    );
}

#[test]
fn cancelling_after_a_view_reports_unread_bytes_and_releases_the_held_frame() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("cancelled-view");
    let (serving, placement) = fixture::initialize(&root);
    let expected = fixture::payload(3 * fixture::CHUNK_PAYLOAD_BYTES + 19);
    let publication = publish_single(
        &serving,
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([173; 32]),
        RecordAppendBatch::try_from_iter([expected.as_slice()]).unwrap(),
    );
    let record = publication.settled_members()[0].record_id(0).unwrap();
    let mut session = serving
        .records()
        .open(
            record,
            RecordReadLimits::new(RecordByteLimit::new(expected.len() as u32).unwrap()),
        )
        .unwrap();

    {
        let first = session.next_chunk().unwrap().unwrap();
        assert_eq!(first.bytes(), &expected[..fixture::CHUNK_PAYLOAD_BYTES]);
    }
    assert_eq!(serving.observer().record_counters().read_sessions_live(), 1);
    let residency = serving.residency_observation().counters();
    assert_eq!(residency.pin_leases(), 1);
    assert_eq!(residency.pinned_frames(), 1);
    assert!(residency.active_operation_bytes() > 0);
    let media_before_cancel = serving.media_counters();

    let cancellation = session.cancel();

    assert_eq!(
        cancellation.observation().bytes_completed(),
        fixture::CHUNK_PAYLOAD_BYTES as u64
    );
    assert_eq!(
        cancellation.unread_payload_bytes(),
        (expected.len() - fixture::CHUNK_PAYLOAD_BYTES) as u64
    );
    assert!(!cancellation.delivery_was_complete());
    assert_eq!(serving.media_counters(), media_before_cancel);
    assert_eq!(serving.observer().record_counters().read_sessions_live(), 0);
    let residency = serving.residency_observation().counters();
    assert_eq!(residency.pin_leases(), 0);
    assert_eq!(residency.pinned_frames(), 0);
    assert_eq!(residency.active_operation_bytes(), 0);
    fixture::assert_clean_close(serving);
}

#[test]
fn abort_during_extent_read_reports_runtime_release_without_accusing_bytes() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("aborted-view");
    let (serving, placement) = fixture::initialize(&root);
    let expected = fixture::payload(3 * fixture::CHUNK_PAYLOAD_BYTES + 19);
    let publication = publish_single(
        &serving,
        placement,
        PhysicalMutationIdempotencyMaterial::new([174; 32]),
        RecordAppendBatch::try_from_iter([expected.as_slice()]).unwrap(),
    );
    let record = publication.settled_members()[0].record_id(0).unwrap();
    let mut session = serving
        .records()
        .open(
            record,
            RecordReadLimits::new(RecordByteLimit::new(expected.len() as u32).unwrap()),
        )
        .unwrap();

    let first = session.next_chunk().unwrap().unwrap();
    assert_eq!(first.bytes(), &expected[..fixture::CHUNK_PAYLOAD_BYTES]);
    drop(first);

    let _shutdown = serving.abort();
    let failure = match session.next_chunk() {
        Err(failure) => failure,
        Ok(_) => panic!("an aborted runtime must stop the next extent read"),
    };

    assert_eq!(failure.kind(), RecordStreamFailureKind::RuntimeReleased);
    assert_eq!(
        failure.completed_range(),
        0..fixture::CHUNK_PAYLOAD_BYTES as u64
    );
}
