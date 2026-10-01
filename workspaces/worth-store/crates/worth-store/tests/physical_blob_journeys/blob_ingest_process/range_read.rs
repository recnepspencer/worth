use sha2::{Digest, Sha256};
use worth_store::physical_runtime::{BlobReadSession, ServingPhysicalRuntime};
use worth_store_physical_backend::{MediaCounterSnapshot, MediaOperationRole};

use super::{
    admitted_blob_scope, expected_byte, hex, serving_from_open, unhex_16, BlobReadLimits,
    NonZeroU64, Path, OBJECT_BYTES, PROCESS_PREFIX, PUBLICATION_ENV, READ_END, READ_PREFIX,
    READ_START, SCOPE_KEY, SELECTED_SCAN_LIMIT,
};

pub(crate) fn reader(root: &Path) {
    println!("{PROCESS_PREFIX}{}", std::process::id());
    let declaration = std::env::var(PUBLICATION_ENV).unwrap();
    let (object_hex, generation) = declaration.split_once(':').unwrap();
    let object = unhex_16(object_hex);
    let generation: u64 = generation.parse().unwrap();
    let serving = serving_from_open(root);
    let scope = admitted_blob_scope(SCOPE_KEY);
    let limits = BlobReadLimits::new(NonZeroU64::new(SELECTED_SCAN_LIMIT).unwrap());
    let blobs = serving.blobs().unwrap();
    let published = blobs
        .resolve_publication(object, generation, &scope, limits)
        .unwrap();
    let mut read = blobs
        .read(
            published,
            &scope,
            READ_START as u64,
            (READ_END - READ_START) as u64,
            limits,
        )
        .unwrap();
    assert_eq!(read.published_bytes(), OBJECT_BYTES as u64);
    assert_eq!(read.observation().touched_chunks(), 0);
    assert_eq!(read.observation().tree_nodes_loaded(), 1);
    // Catalog selection and root validation finished before this matched
    // snapshot. The delta measures streaming traversal alone.
    let before_work = read.observation().physical_work_count();
    let before_media = serving.media_counters();
    let digest = stream_expected_range(&serving, &mut read);
    let observation = read.observation();
    assert_eq!(observation.touched_chunks(), 5);
    assert_eq!(observation.tree_nodes_loaded(), 1);
    assert_eq!(observation.returned_bytes(), (READ_END - READ_START) as u64);
    let after_media = serving.media_counters();
    assert_eq!(
        observation.physical_work_count() - before_work,
        traversal_io_attempts(after_media) - traversal_io_attempts(before_media),
        "C5 traversal work must match independent backend read/metadata attempts"
    );
    let read_attempts =
        after_media.positioned_read_attempts() - before_media.positioned_read_attempts();
    let media_bytes = after_media.completed_bytes_for(MediaOperationRole::PositionedRead)
        - before_media.completed_bytes_for(MediaOperationRole::PositionedRead);
    assert!(
        media_bytes < OBJECT_BYTES as u64,
        "partial read loaded whole blob"
    );
    assert!(
        read_attempts > 0,
        "fresh reader must exercise physical chunk faults"
    );
    println!(
        "C11_BLOB_IO touched_chunks={} returned_bytes={} physical_work={} attempts={read_attempts} media_bytes={media_bytes}",
        observation.touched_chunks(), observation.returned_bytes(),
        observation.physical_work_count() - before_work,
    );
    drop(read);
    println!("{READ_PREFIX}{digest}");
    serving.close();
}

fn stream_expected_range(
    serving: &ServingPhysicalRuntime,
    read: &mut BlobReadSession<'_>,
) -> String {
    let mut buffer = [0_u8; 64 * 1024];
    let mut read_bytes = 0_usize;
    let mut digest = Sha256::new();
    let mut retained_chunk_reuses = 0;
    while read.remaining_bytes() != 0 {
        let before = read.observation();
        let before_media = serving.media_counters();
        let count = read.read_next(&mut buffer).unwrap();
        assert!(count > 0);
        for (offset, observed) in buffer[..count].iter().enumerate() {
            assert_eq!(*observed, expected_byte(READ_START + read_bytes + offset));
        }
        if read.observation().touched_chunks() == before.touched_chunks() {
            retained_chunk_reuses += 1;
            assert_eq!(
                read.observation().physical_work_count(),
                before.physical_work_count()
            );
            assert_eq!(
                traversal_io_attempts(serving.media_counters()),
                traversal_io_attempts(before_media)
            );
        }
        digest.update(&buffer[..count]);
        read_bytes += count;
    }
    assert!(
        retained_chunk_reuses > 0,
        "test must exercise retained chunk reuse"
    );
    assert_eq!(read_bytes, READ_END - READ_START);
    assert_eq!(read.read_next(&mut buffer).unwrap(), 0);
    hex(&digest.finalize())
}

fn traversal_io_attempts(counters: MediaCounterSnapshot) -> u64 {
    counters.identified_operation_attempts_for(MediaOperationRole::PositionedRead)
        + counters.identified_operation_attempts_for(MediaOperationRole::ReadMetadata)
}
