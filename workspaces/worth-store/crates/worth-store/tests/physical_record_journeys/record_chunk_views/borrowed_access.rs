use worth_store::physical_runtime::{
    ExternalPhysicalRecordLocator, RecordAppendBatch, RecordByteLimit, RecordReadLimits,
};
use worth_store_physical_format::{RecordArtifactFile, RecordFrameCoordinate};

use super::super::durable_publication::publish_single;
use super::fixture;

#[test]
fn inline_view_exposes_only_the_record_payload_and_observational_basis() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("inline-view");
    let (serving, placement) = fixture::initialize(&root);
    let expected = b"lease-scoped inline payload";
    let published = publish_single(
        &serving,
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([172; 32]),
        RecordAppendBatch::try_from_iter([expected.as_slice()]).unwrap(),
    );
    let record = published.settled_members()[0].record_id(0).unwrap();
    let store = serving.store_identity();
    let generation = serving.residency_observation().store_generation();
    let copies_before = serving.residency_observation().counters();
    let mut session = serving
        .records()
        .expect("read protection admission")
        .open(
            record,
            RecordReadLimits::new(RecordByteLimit::new(expected.len() as u32).unwrap()),
        )
        .unwrap();

    {
        let chunk = session.next_chunk().unwrap().unwrap();
        assert_eq!(chunk.bytes(), expected);
        assert_eq!(chunk.logical_range(), 0..expected.len() as u64);
        assert_eq!(chunk.basis().store_identity(), store);
        assert_eq!(chunk.basis().store_generation(), generation);
        assert_eq!(chunk.basis().record(), record);
        assert_eq!(
            chunk.basis().frame_coordinate().length(),
            fixture::FRAME_BYTES as u32
        );
    }
    assert!(session.next_chunk().unwrap().is_none());
    let observation = session.observation();
    assert_eq!(observation.payload_bytes(), expected.len() as u64);
    assert_eq!(observation.explicit_copy_count(), 0);
    assert_eq!(observation.copied_bytes(), 0);
    drop(session);
    let copies_after = serving.residency_observation().counters();
    assert_eq!(
        copies_after.copy_operations(),
        copies_before.copy_operations()
    );
    assert_eq!(copies_after.copied_bytes(), copies_before.copied_bytes());
    fixture::assert_clean_close(serving);
}

#[test]
fn external_locator_view_retains_the_readmitted_record_basis_without_copying() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("external-locator-view");
    let (serving, placement) = fixture::initialize(&root);
    let expected = b"externally located lease-scoped payload";
    let published = publish_single(
        &serving,
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([172; 32]),
        RecordAppendBatch::try_from_iter([expected.as_slice()]).unwrap(),
    );
    let record = published.settled_members()[0].record_id(0).unwrap();
    let store = serving.store_identity();
    let generation = serving.residency_observation().store_generation();
    let locator = ExternalPhysicalRecordLocator::new(store, record);
    let copies_before = serving.residency_observation().counters();
    let mut session = serving
        .records()
        .expect("read protection admission")
        .open_external(
            locator,
            RecordReadLimits::new(RecordByteLimit::new(expected.len() as u32).unwrap()),
        )
        .unwrap();

    {
        let chunk = session.next_chunk().unwrap().unwrap();
        assert_eq!(chunk.bytes(), expected);
        assert_eq!(chunk.logical_range(), 0..expected.len() as u64);
        assert_eq!(chunk.basis().store_identity(), store);
        assert_eq!(chunk.basis().store_generation(), generation);
        assert_eq!(chunk.basis().record(), record);
        assert_eq!(
            chunk.basis().frame_coordinate().length(),
            fixture::FRAME_BYTES as u32
        );
    }
    assert!(session.next_chunk().unwrap().is_none());
    let observation = session.observation();
    assert_eq!(observation.payload_bytes(), expected.len() as u64);
    assert_eq!(observation.explicit_copy_count(), 0);
    assert_eq!(observation.copied_bytes(), 0);
    drop(session);
    let copies_after = serving.residency_observation().counters();
    assert_eq!(
        copies_after.copy_operations(),
        copies_before.copy_operations()
    );
    assert_eq!(copies_after.copied_bytes(), copies_before.copied_bytes());
    fixture::assert_clean_close(serving);
}

#[test]
fn extent_views_stream_one_resident_frame_at_a_time_without_pool_copies() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("extent-views");
    let (serving, placement) = fixture::initialize(&root);
    let expected = fixture::payload(5 * fixture::CHUNK_PAYLOAD_BYTES + 37);
    assert!(expected.len() as u64 > fixture::RESIDENT_BYTES);
    let published = publish_single(
        &serving,
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([172; 32]),
        RecordAppendBatch::try_from_iter([expected.as_slice()]).unwrap(),
    );
    let record = published.settled_members()[0].record_id(0).unwrap();
    let generation = serving.residency_observation().store_generation();
    let copies_before = serving.residency_observation().counters();
    let mut session = serving
        .records()
        .expect("read protection admission")
        .open(
            record,
            RecordReadLimits::new(RecordByteLimit::new(expected.len() as u32).unwrap()),
        )
        .unwrap();
    let mut logical_offset = 0_u64;
    let mut frame_index = 0_u64;
    let artifact = RecordArtifactFile::Extent {
        extent: 1,
        generation: 1,
    };
    assert!(root
        .join("families/records/extents")
        .join(artifact.file_name())
        .is_file());
    let residency = serving.certification_physical_residency();

    while let Some(chunk) = session.next_chunk().unwrap() {
        let range = chunk.logical_range();
        let decoded_payload_offset = fixture::FRAME_BYTES as usize - fixture::CHUNK_PAYLOAD_BYTES;
        let expected_payload_start = frame_index as usize * fixture::CHUNK_PAYLOAD_BYTES;
        let expected_payload_bytes =
            (expected.len() - expected_payload_start).min(fixture::CHUNK_PAYLOAD_BYTES);
        let coordinate = RecordFrameCoordinate::new(
            artifact,
            frame_index * fixture::FRAME_BYTES,
            (decoded_payload_offset + expected_payload_bytes) as u32,
        )
        .unwrap();
        assert_eq!(range.start, logical_offset);
        assert_eq!(
            chunk.bytes(),
            &expected[range.start as usize..range.end as usize]
        );
        assert_eq!(chunk.basis().store_identity(), serving.store_identity());
        assert_eq!(chunk.basis().store_generation(), generation);
        assert_eq!(chunk.basis().record(), record);
        assert_eq!(chunk.basis().frame_coordinate(), coordinate);
        let resident = residency.pin_exact(coordinate).unwrap();
        assert_eq!(
            chunk.bytes().as_ptr(),
            resident.bytes()[decoded_payload_offset..].as_ptr()
        );
        logical_offset = range.end;
        frame_index += 1;
    }

    assert_eq!(logical_offset, expected.len() as u64);
    assert_eq!(frame_index, 6);
    let observation = session.observation();
    assert_eq!(observation.payload_bytes(), expected.len() as u64);
    assert_eq!(observation.explicit_copy_count(), 0);
    assert_eq!(observation.copied_bytes(), 0);
    drop(session);
    let copies_after = serving.residency_observation().counters();
    assert_eq!(
        copies_after.copy_operations(),
        copies_before.copy_operations()
    );
    assert_eq!(copies_after.copied_bytes(), copies_before.copied_bytes());
    assert!(copies_after.peak_resident_bytes() <= fixture::RESIDENT_BYTES);
    assert!(copies_after.evictions() > 0);
    fixture::assert_clean_close(serving);
}

#[test]
fn dropping_a_partially_consumed_extent_releases_its_session_frame_and_allocation() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("partial-extent-drop");
    let (serving, placement) = fixture::initialize(&root);
    let expected = fixture::payload(3 * fixture::CHUNK_PAYLOAD_BYTES + 19);
    let published = publish_single(
        &serving,
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([172; 32]),
        RecordAppendBatch::try_from_iter([expected.as_slice()]).unwrap(),
    );
    let mut session = serving
        .records()
        .expect("read protection admission")
        .open(
            published.settled_members()[0].record_id(0).unwrap(),
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
    assert_eq!(serving.observer().record_counters().read_sessions_live(), 1);
    let residency = serving.residency_observation().counters();
    assert_eq!(residency.pin_leases(), 1);
    assert_eq!(residency.pinned_frames(), 1);
    assert!(residency.active_operation_bytes() > 0);

    drop(session);

    assert_eq!(serving.observer().record_counters().read_sessions_live(), 0);
    let residency = serving.residency_observation().counters();
    assert_eq!(residency.pin_leases(), 0);
    assert_eq!(residency.pinned_frames(), 0);
    assert_eq!(residency.active_operation_bytes(), 0);
    fixture::assert_clean_close(serving);
}
