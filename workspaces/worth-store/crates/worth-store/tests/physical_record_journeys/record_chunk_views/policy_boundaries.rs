use worth_store::physical_runtime::{
    PhysicalOperationAllocationScope, PhysicalResidencyDimension, PhysicalResidencyRetryPosture,
    RecordAppendBatch, RecordByteLimit, RecordReadDenial, RecordReadLimits,
    RecordStreamFailureKind,
};

use super::super::durable_publication::publish_single;
use super::fixture;

#[test]
fn caller_maximum_payload_denies_before_session_delivery_and_releases_allocation() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("caller-limit");
    let (serving, placement) = fixture::initialize(&root);
    let expected = fixture::payload(2 * fixture::CHUNK_PAYLOAD_BYTES + 17);
    let publication = publish_single(
        &serving,
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([171; 32]),
        RecordAppendBatch::try_from_iter([expected.as_slice()]).unwrap(),
    );
    let record = publication.settled_members()[0].record_id(0).unwrap();

    let denial = match serving.records().expect("read protection admission").open(
        record,
        RecordReadLimits::new(RecordByteLimit::new(expected.len() as u32 - 1).unwrap()),
    ) {
        Err(error) => error,
        Ok(_) => panic!("a caller limit below the logical payload must deny the read session"),
    };

    assert_eq!(denial.denial(), RecordReadDenial::CallerLimitExceeded);
    assert_eq!(
        denial.observation().bytes_requested(),
        expected.len() as u64
    );
    assert_eq!(denial.observation().bytes_completed(), 0);
    assert_eq!(serving.observer().record_counters().read_sessions_live(), 0);
    let residency = serving.residency_observation().counters();
    assert_eq!(residency.pin_leases(), 0);
    assert_eq!(residency.pinned_frames(), 0);
    assert_eq!(residency.active_operation_bytes(), 0);

    let mut admitted = serving
        .records()
        .expect("read protection admission")
        .open(
            record,
            RecordReadLimits::new(RecordByteLimit::new(expected.len() as u32).unwrap()),
        )
        .unwrap();
    {
        let first = admitted.next_chunk().unwrap().unwrap();
        assert_eq!(first.bytes(), &expected[..fixture::CHUNK_PAYLOAD_BYTES]);
    }
    let cancellation = admitted.cancel();
    assert_eq!(
        cancellation.observation().bytes_requested(),
        expected.len() as u64
    );
    assert_eq!(
        cancellation.observation().bytes_completed(),
        fixture::CHUNK_PAYLOAD_BYTES as u64
    );
    assert_eq!(
        cancellation.unread_payload_bytes(),
        (expected.len() - fixture::CHUNK_PAYLOAD_BYTES) as u64
    );
    fixture::assert_clean_close(serving);
}

#[test]
fn public_extent_views_preserve_exact_over_pin_pressure_without_revoking_store_health() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("public-over-pin");
    let (serving, placement) = fixture::initialize(&root);
    let expected = fixture::payload(2 * fixture::CHUNK_PAYLOAD_BYTES + 17);
    let publication = publish_single(
        &serving,
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([171; 32]),
        RecordAppendBatch::try_from_iter([expected.as_slice()]).unwrap(),
    );
    let record = publication.settled_members()[0].record_id(0).unwrap();
    let limits = RecordReadLimits::new(RecordByteLimit::new(expected.len() as u32).unwrap());
    let reader = serving.records().expect("read protection admission");
    {
        let mut first = reader.open(record, limits).unwrap();
        let mut second = reader.open(record, limits).unwrap();
        let mut denied = reader.open(record, limits).unwrap();
        let first_view = first.next_chunk().unwrap().unwrap();
        let second_view = second.next_chunk().unwrap().unwrap();
        assert_eq!(
            first_view.basis().frame_coordinate(),
            second_view.basis().frame_coordinate()
        );
        let failure = match denied.next_chunk() {
            Err(failure) => failure,
            Ok(_) => panic!("the third public view must exceed the two-lease policy"),
        };
        assert_eq!(failure.kind(), RecordStreamFailureKind::PhysicalPressure);
        assert_eq!(failure.completed_range(), 0..0);
        let pressure = failure
            .pressure()
            .expect("stream pressure must carry exact Store-facing evidence");
        assert_eq!(pressure.dimension(), PhysicalResidencyDimension::PinLeases);
        assert_eq!(
            pressure.scope(),
            PhysicalOperationAllocationScope::ForegroundRead
        );
        assert_eq!(pressure.requested(), 1);
        assert_eq!(pressure.admitted(), 2);
        assert_eq!(pressure.limit(), 2);
        assert_eq!(
            pressure.retry_posture(),
            PhysicalResidencyRetryPosture::AfterLeaseRelease
        );
        assert!(!pressure.effect_may_have_started());
        assert_eq!(pressure.basis().store_identity(), serving.store_identity());
        assert_eq!(pressure.basis().record(), Some(record));
        assert_eq!(
            pressure.basis().frame_coordinate(),
            Some(first_view.basis().frame_coordinate())
        );
        assert_eq!(denied.observation().bytes_completed(), 0);
    }
    let mut retry = reader
        .open(record, limits)
        .expect("pre-effect pressure must not revoke Store health");
    assert_eq!(
        retry.next_chunk().unwrap().unwrap().bytes(),
        &expected[..fixture::CHUNK_PAYLOAD_BYTES]
    );
    drop(retry);
    drop(reader);
    fixture::assert_clean_close(serving);
}
