use worth_store::physical_runtime::{RecordAppendBatch, RecordByteLimit, RecordReadLimits};

use super::super::durable_publication::publish_single;
use super::fixture;

#[derive(Default)]
struct CallerCopyEvidence {
    operations: u64,
    bytes: u64,
    maximum_width: u64,
}

impl CallerCopyEvidence {
    fn observe(&mut self, copied: usize) {
        self.operations += 1;
        self.bytes += copied as u64;
        self.maximum_width = self.maximum_width.max(copied as u64);
    }
}

#[test]
fn bounded_copy_streams_the_complete_larger_than_memory_record_with_exact_evidence() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("complete-bounded-copy");
    let (serving, placement) = fixture::initialize(&root);
    let expected = fixture::payload(7 * fixture::CHUNK_PAYLOAD_BYTES + 53);
    assert!(expected.len() as u64 > fixture::RESIDENT_BYTES);
    let published = publish_single(
        &serving,
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([170; 32]),
        RecordAppendBatch::try_from_iter([expected.as_slice()]).unwrap(),
    );
    let record = published.settled_members()[0].record_id(0).unwrap();
    let copies_before = serving.residency_observation().counters();
    let mut session = serving
        .records()
        .expect("read protection admission")
        .open(
            record,
            RecordReadLimits::new(RecordByteLimit::new(expected.len() as u32).unwrap()),
        )
        .unwrap();
    let mut reconstructed = Vec::with_capacity(expected.len());
    let mut caller = CallerCopyEvidence::default();

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
    assert!(copies_after.peak_resident_bytes() <= fixture::RESIDENT_BYTES);
    assert!(copies_after.evictions() > copies_before.evictions());
    fixture::assert_clean_close(serving);
}

#[test]
fn bounded_copies_and_views_share_one_cursor_with_exact_copy_evidence() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("interleaved-view-copy");
    let (serving, placement) = fixture::initialize(&root);
    let expected = fixture::payload(5 * fixture::CHUNK_PAYLOAD_BYTES + 37);
    assert!(expected.len() as u64 > fixture::RESIDENT_BYTES);
    let published = publish_single(
        &serving,
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([170; 32]),
        RecordAppendBatch::try_from_iter([expected.as_slice()]).unwrap(),
    );
    let record = published.settled_members()[0].record_id(0).unwrap();
    let copies_before = serving.residency_observation().counters();
    let mut session = serving
        .records()
        .expect("read protection admission")
        .open(
            record,
            RecordReadLimits::new(RecordByteLimit::new(expected.len() as u32).unwrap()),
        )
        .unwrap();
    let mut reconstructed = Vec::with_capacity(expected.len());
    let mut caller = CallerCopyEvidence::default();

    copy_once(&mut session, 73, &mut reconstructed, &mut caller);
    {
        let chunk = session.next_chunk().unwrap().unwrap();
        assert_eq!(
            chunk.logical_range(),
            73..fixture::CHUNK_PAYLOAD_BYTES as u64
        );
        reconstructed.extend_from_slice(chunk.bytes());
    }
    copy_once(&mut session, 101, &mut reconstructed, &mut caller);
    {
        let chunk = session.next_chunk().unwrap().unwrap();
        assert_eq!(
            chunk.logical_range(),
            fixture::CHUNK_PAYLOAD_BYTES as u64 + 101..2 * fixture::CHUNK_PAYLOAD_BYTES as u64
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
    assert_eq!(observation.peak_transfer_width(), fixture::FRAME_BYTES);
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
    assert!(copies_after.peak_resident_bytes() <= fixture::RESIDENT_BYTES);
    fixture::assert_clean_close(serving);
}

fn copy_once(
    session: &mut worth_store::physical_runtime::RecordReadSession,
    width: usize,
    reconstructed: &mut Vec<u8>,
    caller: &mut CallerCopyEvidence,
) {
    let mut target = vec![0_u8; width];
    let copied = session.read_next(&mut target).unwrap();
    assert_eq!(copied, width);
    reconstructed.extend_from_slice(&target[..copied]);
    caller.observe(copied);
}
