#[allow(unused_imports)]
use crate::physical_record_journeys::record_chunk_views::bounded_copy::*;
use {
    worth_store::physical_runtime::RecordAppendBatch,
    worth_store::physical_runtime::RecordByteLimit,
    worth_store::physical_runtime::RecordReadLimits,
};

use worth_store_testkit_store::durable_publication::root::publish_single;
use worth_store_testkit_store::physical_record_journeys::record_chunk_views::fixture;

#[test]
fn bounded_copy_streams_the_complete_larger_than_memory_record_with_exact_evidence() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("complete-bounded-copy");
    let (serving, placement) = worth_store_testkit_store::physical_record_journeys::record_chunk_views::fixture::initialize(&root);
    let expected = worth_store_testkit_store::physical_record_journeys::record_chunk_views::fixture::payload(7 * worth_store_testkit_store::physical_record_journeys::record_chunk_views::fixture::CHUNK_PAYLOAD_BYTES + 53);
    assert!(expected.len() as u64 > worth_store_testkit_store::physical_record_journeys::record_chunk_views::fixture::RESIDENT_BYTES);
    let published = publish_single(
        &serving,
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([170; 32]),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter([expected.as_slice()])
            .unwrap(),
    );
    let record = published.settled_members()[0].record_id(0).unwrap();
    let copies_before = serving.residency_observation().counters();
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
    let mut reconstructed = Vec::with_capacity(expected.len());
    let mut caller = crate::physical_record_journeys::record_chunk_views::bounded_copy::CallerCopyEvidence::default();

    loop {
        let mut target = [0_u8; 997];
        let copied = session.read_next(&mut target).unwrap();
        if copied == 0 {
            break;
        }
        reconstructed.extend_from_slice(&target[..copied]);
        caller.observe(copied);
    }

    assert_eq!(reconstructed, expected);
    let observation = session.observation();
    assert_eq!(observation.payload_bytes(), expected.len() as u64);
    assert_eq!(observation.explicit_copy_count(), caller.operations);
    assert_eq!(observation.copied_bytes(), caller.bytes);
    assert_eq!(caller.bytes, expected.len() as u64);
    assert_eq!(caller.maximum_width, 997);
    drop(session);
    let copies_after = serving.residency_observation().counters();
    assert_eq!(
        copies_after.copy_operations() - copies_before.copy_operations(),
        caller.operations
    );
    assert_eq!(
        copies_after.copied_bytes() - copies_before.copied_bytes(),
        caller.bytes
    );
    assert_eq!(
        copies_after.maximum_copy_width(),
        copies_before.maximum_copy_width().max(caller.maximum_width)
    );
    assert!(copies_after.peak_resident_bytes() <= worth_store_testkit_store::physical_record_journeys::record_chunk_views::fixture::RESIDENT_BYTES);
    assert!(copies_after.evictions() > copies_before.evictions());
    worth_store_testkit_store::physical_record_journeys::record_chunk_views::fixture::assert_clean_close(serving);
}

#[test]
fn bounded_copies_and_views_share_one_cursor_with_exact_copy_evidence() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("interleaved-view-copy");
    let (serving, placement) = worth_store_testkit_store::physical_record_journeys::record_chunk_views::fixture::initialize(&root);
    let expected = worth_store_testkit_store::physical_record_journeys::record_chunk_views::fixture::payload(5 * worth_store_testkit_store::physical_record_journeys::record_chunk_views::fixture::CHUNK_PAYLOAD_BYTES + 37);
    assert!(expected.len() as u64 > worth_store_testkit_store::physical_record_journeys::record_chunk_views::fixture::RESIDENT_BYTES);
    let published = publish_single(
        &serving,
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([170; 32]),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter([expected.as_slice()])
            .unwrap(),
    );
    let record = published.settled_members()[0].record_id(0).unwrap();
    let copies_before = serving.residency_observation().counters();
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
    let mut reconstructed = Vec::with_capacity(expected.len());
    let mut caller = crate::physical_record_journeys::record_chunk_views::bounded_copy::CallerCopyEvidence::default();

    copy_once(&mut session, 73, &mut reconstructed, &mut caller);
    {
        let chunk = session.next_chunk().unwrap().unwrap();
        assert_eq!(
            chunk.logical_range(),
            73..worth_store_testkit_store::physical_record_journeys::record_chunk_views::fixture::CHUNK_PAYLOAD_BYTES as u64
        );
        reconstructed.extend_from_slice(chunk.bytes());
    }
    copy_once(&mut session, 101, &mut reconstructed, &mut caller);
    {
        let chunk = session.next_chunk().unwrap().unwrap();
        assert_eq!(
            chunk.logical_range(),
            worth_store_testkit_store::physical_record_journeys::record_chunk_views::fixture::CHUNK_PAYLOAD_BYTES as u64 + 101..2 * worth_store_testkit_store::physical_record_journeys::record_chunk_views::fixture::CHUNK_PAYLOAD_BYTES as u64
        );
        reconstructed.extend_from_slice(chunk.bytes());
    }
    while let Some(chunk) = session.next_chunk().unwrap() {
        reconstructed.extend_from_slice(chunk.bytes());
    }

    assert_eq!(reconstructed, expected);
    assert!(session.next_chunk().unwrap().is_none());
    assert_eq!(session.read_next(&mut [0_u8; 1]).unwrap(), 0);
    let observation = session.observation();
    assert_eq!(observation.payload_bytes(), expected.len() as u64);
    assert_eq!(observation.explicit_copy_count(), caller.operations);
    assert_eq!(observation.copied_bytes(), caller.bytes);
    assert_eq!(observation.peak_transfer_width(), worth_store_testkit_store::physical_record_journeys::record_chunk_views::fixture::FRAME_BYTES);
    drop(session);

    let copies_after = serving.residency_observation().counters();
    assert_eq!(
        copies_after.copy_operations() - copies_before.copy_operations(),
        caller.operations
    );
    assert_eq!(
        copies_after.copied_bytes() - copies_before.copied_bytes(),
        caller.bytes
    );
    assert_eq!(
        copies_after.maximum_copy_width(),
        copies_before.maximum_copy_width().max(caller.maximum_width)
    );
    assert!(copies_after.peak_resident_bytes() <= worth_store_testkit_store::physical_record_journeys::record_chunk_views::fixture::RESIDENT_BYTES);
    worth_store_testkit_store::physical_record_journeys::record_chunk_views::fixture::assert_clean_close(serving);
}
