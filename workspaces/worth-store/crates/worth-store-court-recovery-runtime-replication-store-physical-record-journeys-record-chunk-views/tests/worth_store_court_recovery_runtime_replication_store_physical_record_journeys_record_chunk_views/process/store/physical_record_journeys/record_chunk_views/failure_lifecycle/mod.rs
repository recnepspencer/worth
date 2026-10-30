use {std::io::Seek, std::io::Write};

use worth_store_testkit_store::durable_publication::root::publish_single;

#[test]
fn later_extent_damage_through_a_view_stays_local_and_releases_read_authority() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("damaged-view");
    let (serving, placement) = worth_store_testkit_store::physical_record_journeys::record_chunk_views::fixture::initialize(&root);
    let expected =
        worth_store_testkit_store::physical_record_journeys::record_chunk_views::fixture::payload(
            40_000,
        );
    let publication = publish_single(
        &serving,
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([173; 32]),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter([expected.as_slice()])
            .unwrap(),
    );
    let operation_bytes_before_read = serving
        .residency_observation()
        .counters()
        .active_operation_bytes();
    let record = publication.settled_members()[0].record_id(0).unwrap();
    let extent = root.join("families/records/arenas/arena-0000000000000001.data");
    let arena = std::fs::read(&extent).unwrap();
    let second_chunk =
        worth_store_testkit_store::durable_frame_oracle::first_arena_chunk_offset(&arena, 1);
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .open(extent)
        .unwrap();
    file.seek(std::io::SeekFrom::Start(second_chunk + 120))
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
            record,
            worth_store::physical_runtime::RecordReadLimits::new(
                worth_store::physical_runtime::RecordByteLimit::new(expected.len() as u32).unwrap(),
            ),
        )
        .unwrap();

    {
        let first = session.next_chunk().unwrap().unwrap();
        assert_eq!(
            first.logical_range(),
            0..worth_store_testkit_store::physical_record_journeys::record_chunk_views::fixture::CHUNK_PAYLOAD_BYTES as u64
        );
        assert_eq!(first.bytes(), &expected[..worth_store_testkit_store::physical_record_journeys::record_chunk_views::fixture::CHUNK_PAYLOAD_BYTES]);
    }
    let failure = match session.next_chunk() {
        Err(failure) => failure,
        Ok(_) => panic!("the damaged second extent frame must fail borrowed iteration"),
    };
    // Data-frame checksum damage localizes to the dependent range; only
    // root/routing/manifest damage revokes global serving health (spec C.11
    // "Read and verification").
    assert_eq!(
        failure.kind(),
        worth_store::physical_runtime::RecordStreamFailureKind::SelectedDataFrameChecksumDamaged
    );
    assert_eq!(
        failure.completed_range(),
        0..worth_store_testkit_store::physical_record_journeys::record_chunk_views::fixture::CHUNK_PAYLOAD_BYTES as u64
    );
    assert_eq!(session.observation().generation_checks(), 2);
    assert_eq!(session.observation().generation_rejections(), 0);
    drop(session);

    assert_eq!(serving.observer().record_counters().read_sessions_live(), 0);
    let residency = serving.residency_observation().counters();
    assert_eq!(residency.pin_leases(), 0);
    assert_eq!(residency.pinned_frames(), 0);
    assert_eq!(
        residency.active_operation_bytes(),
        operation_bytes_before_read
    );
    publish_single(
        &serving,
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([213; 32]),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter([
            b"served after local damage".as_slice(),
        ])
        .unwrap(),
    );
    assert_eq!(
        serving.abort().records().posture(),
        worth_store::physical_runtime::RecordServingTerminalPosture::NoInspectionRequired
    );
}

#[test]
fn cancelling_after_a_view_reports_unread_bytes_and_releases_the_held_frame() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("cancelled-view");
    let (serving, placement) = worth_store_testkit_store::physical_record_journeys::record_chunk_views::fixture::initialize(&root);
    let expected = worth_store_testkit_store::physical_record_journeys::record_chunk_views::fixture::payload(3 * worth_store_testkit_store::physical_record_journeys::record_chunk_views::fixture::CHUNK_PAYLOAD_BYTES + 19);
    let publication = publish_single(
        &serving,
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([173; 32]),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter([expected.as_slice()])
            .unwrap(),
    );
    let operation_bytes_before_read = serving
        .residency_observation()
        .counters()
        .active_operation_bytes();
    let record = publication.settled_members()[0].record_id(0).unwrap();
    let mut session = serving
        .records()
        .expect("read protection admission")
        .open(
            record,
            worth_store::physical_runtime::RecordReadLimits::new(
                worth_store::physical_runtime::RecordByteLimit::new(expected.len() as u32).unwrap(),
            ),
        )
        .unwrap();

    {
        let first = session.next_chunk().unwrap().unwrap();
        assert_eq!(first.bytes(), &expected[..worth_store_testkit_store::physical_record_journeys::record_chunk_views::fixture::CHUNK_PAYLOAD_BYTES]);
    }
    assert_eq!(serving.observer().record_counters().read_sessions_live(), 1);
    let residency = serving.residency_observation().counters();
    assert_eq!(residency.pin_leases(), 1);
    assert_eq!(residency.pinned_frames(), 1);
    assert!(residency.active_operation_bytes() > operation_bytes_before_read);
    let media_before_cancel = serving.media_counters();

    let cancellation = session.cancel();

    assert_eq!(
        cancellation.observation().bytes_completed(),
        worth_store_testkit_store::physical_record_journeys::record_chunk_views::fixture::CHUNK_PAYLOAD_BYTES as u64
    );
    assert_eq!(
        cancellation.unread_payload_bytes(),
        (expected.len() - worth_store_testkit_store::physical_record_journeys::record_chunk_views::fixture::CHUNK_PAYLOAD_BYTES) as u64
    );
    assert!(!cancellation.delivery_was_complete());
    assert_eq!(serving.media_counters(), media_before_cancel);
    assert_eq!(serving.observer().record_counters().read_sessions_live(), 0);
    let residency = serving.residency_observation().counters();
    assert_eq!(residency.pin_leases(), 0);
    assert_eq!(residency.pinned_frames(), 0);
    assert_eq!(
        residency.active_operation_bytes(),
        operation_bytes_before_read
    );
    worth_store_testkit_store::physical_record_journeys::record_chunk_views::fixture::assert_clean_close(serving);
}

#[test]
fn abort_during_extent_read_reports_runtime_release_without_accusing_bytes() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("aborted-view");
    let (serving, placement) = worth_store_testkit_store::physical_record_journeys::record_chunk_views::fixture::initialize(&root);
    let expected = worth_store_testkit_store::physical_record_journeys::record_chunk_views::fixture::payload(3 * worth_store_testkit_store::physical_record_journeys::record_chunk_views::fixture::CHUNK_PAYLOAD_BYTES + 19);
    let publication = publish_single(
        &serving,
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([174; 32]),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter([expected.as_slice()])
            .unwrap(),
    );
    let record = publication.settled_members()[0].record_id(0).unwrap();
    let mut session = serving
        .records()
        .expect("read protection admission")
        .open(
            record,
            worth_store::physical_runtime::RecordReadLimits::new(
                worth_store::physical_runtime::RecordByteLimit::new(expected.len() as u32).unwrap(),
            ),
        )
        .unwrap();

    let first = session.next_chunk().unwrap().unwrap();
    assert_eq!(first.bytes(), &expected[..worth_store_testkit_store::physical_record_journeys::record_chunk_views::fixture::CHUNK_PAYLOAD_BYTES]);

    let _shutdown = serving.abort();
    let failure = match session.next_chunk() {
        Err(failure) => failure,
        Ok(_) => panic!("an aborted runtime must stop the next extent read"),
    };

    assert_eq!(
        failure.kind(),
        worth_store::physical_runtime::RecordStreamFailureKind::RuntimeReleased
    );
    assert_eq!(
        failure.completed_range(),
        0..worth_store_testkit_store::physical_record_journeys::record_chunk_views::fixture::CHUNK_PAYLOAD_BYTES as u64
    );
}
