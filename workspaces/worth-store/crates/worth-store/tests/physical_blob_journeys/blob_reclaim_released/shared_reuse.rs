use std::num::{NonZeroU16, NonZeroU64};

use worth_proof::AdmittedBlobReleaseProof;
use worth_store::physical_runtime::{
    AdmittedBlobScope, BlobReadFailure, BlobReadLimits, BlobReadOpenFailure,
    BlobReclaimDisposition, BlobReclaimLimits, BlobReclaimReceipt, BlobReclaimRequest,
    BlobReclaimRetirement, PhysicalMutationDeadline, PublishedBlobGeneration, RecordReadDenial,
    ServingPhysicalRuntime,
};
use worth_store_physical_format::{
    decode_blob_record, BlobRecordV1, IndexedThroughBlobPublication,
};

#[path = "shared_reuse/denied_reopen.rs"]
mod denied_reopen;
#[path = "shared_reuse/fresh_process.rs"]
mod fresh_process;
#[path = "shared_reuse/stale_new_reuse.rs"]
mod stale_new_reuse;

use super::{publish_one, shared_format, shared_placement, SHARED_CHUNK as CHUNK};
use crate::fixture::{admitted_blob_scope, serving_from_initialization_with_format_and_placement};

#[test]
fn retired_tail_slot_does_not_block_selected_successor_reuse() {
    let directory = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization_with_format_and_placement(
        directory.path(),
        shared_format(),
        shared_placement(),
    );
    let scope = admitted_blob_scope("c11.blob.retired.tail.successor");
    let payload = multi_chunk_payload();
    let source = publish_one(&serving, &scope, &payload);
    crate::blob_expiry::completed_checkpoint(&serving, 0xcf);
    let successor = publish_one(&serving, &scope, &payload);
    assert_selected_reuse_claim(&serving);
    assert_bytes_exact(&serving, &scope, source, &payload);
    assert_bytes_exact(&serving, &scope, successor, &payload);
    serving.close();
}

pub(super) fn source_first() {
    let directory = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization_with_format_and_placement(
        directory.path(),
        shared_format(),
        shared_placement(),
    );
    let scope = admitted_blob_scope("c11.blob.release.shared.source.first");
    let payload = multi_chunk_payload();
    let original = publish_one(&serving, &scope, &payload);
    let original_marker = marker(&serving);
    crate::blob_expiry::completed_checkpoint(&serving, 0xd1);
    let reused = publish_one(&serving, &scope, &payload);
    assert_selected_reuse_claim(&serving);
    for index in 0..4_u8 {
        let unrelated = vec![0x20 + index; CHUNK];
        publish_one(&serving, &scope, &unrelated);
        crate::blob_expiry::completed_checkpoint(&serving, 0xd2 + index);
    }

    let source_first = release(&serving, original, original_marker);
    assert_retired(&source_first);
    assert!(source_first
        .dropped_records()
        .contains(&original_marker.record()));
    assert!(source_first.remaining_payload_records() > 0);
    assert_invisible(&serving, &scope, original);
    assert_eq!(assert_bytes_exact(&serving, &scope, reused, &payload), 4);
    stale_new_reuse::assert_fresh_ingest_replaces_stale_cells(&serving, &scope, &payload);
    crate::blob_expiry::completed_checkpoint(&serving, 0xd6);
    serving.close();

    fresh_process::assert_open_denied(directory.path());
    denied_reopen::assert_open_requires_c8_custody(directory.path());
}

#[test]
fn released_reuse_child() {
    fresh_process::run_child();
}

#[test]
fn destination_first_release_drops_claim_then_source_without_stranded_custody() {
    let directory = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization_with_format_and_placement(
        directory.path(),
        shared_format(),
        shared_placement(),
    );
    let scope = admitted_blob_scope("c11.blob.release.shared.destination.first");
    let payload = multi_chunk_payload();
    let original = publish_one(&serving, &scope, &payload);
    let original_marker = marker(&serving);
    crate::blob_expiry::completed_checkpoint(&serving, 0xe1);
    let reused = publish_one(&serving, &scope, &payload);
    let reused_marker = marker(&serving);
    assert_selected_reuse_claim(&serving);
    assert_bytes_exact(&serving, &scope, original, &payload);

    let destination = release(&serving, reused, reused_marker);
    assert_retired(&destination);
    assert_eq!(destination.remaining_payload_records(), 0);
    assert_invisible(&serving, &scope, reused);
    assert!(
        crate::blob_frontier::selected_blob_records(&serving)
            .iter()
            .any(|(record, _)| record.allocation_epoch()
                == original_marker.record().allocation_epoch()
                && record.ordinal() == original_marker.record().ordinal()),
        "destination release must preserve the original publication's authoritative route",
    );
    assert_bytes_exact(&serving, &scope, original, &payload);
    let source = release(&serving, original, original_marker);
    assert_retired(&source);
    assert_eq!(source.remaining_payload_records(), 0);
    assert_invisible(&serving, &scope, original);
    serving.close();

    fresh_process::assert_open_denied(directory.path());
    denied_reopen::assert_open_requires_c8_custody(directory.path());
}

fn marker(serving: &ServingPhysicalRuntime) -> IndexedThroughBlobPublication {
    serving
        .certification_selected_latest_blob_publication()
        .unwrap()
        .expect("published generation selected")
}

fn request(
    serving: &ServingPhysicalRuntime,
    published: PublishedBlobGeneration,
    marker: IndexedThroughBlobPublication,
) -> BlobReclaimRequest<'_> {
    let proof = AdmittedBlobReleaseProof::certification_admit(
        serving.store_identity().bytes(),
        published.object().bytes(),
        published.generation().sequence(),
        marker.record().allocation_epoch(),
        marker.record().ordinal(),
        marker.encoded_digest(),
        [0x73; 32],
    )
    .unwrap();
    BlobReclaimRequest::released(
        proof,
        shared_placement(),
        PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
        BlobReclaimLimits::new(
            NonZeroU64::new(512).unwrap(),
            NonZeroU64::new(64 << 20).unwrap(),
            NonZeroU16::new(16).unwrap(),
        )
        .unwrap(),
    )
}

fn release(
    serving: &ServingPhysicalRuntime,
    published: PublishedBlobGeneration,
    marker: IndexedThroughBlobPublication,
) -> BlobReclaimReceipt {
    let receipt = serving
        .blobs()
        .unwrap()
        .reclaim(request(serving, published, marker))
        .unwrap()
        .wait()
        .unwrap();
    assert_eq!(receipt.disposition(), BlobReclaimDisposition::Dropped);
    receipt
}

fn assert_retired(receipt: &BlobReclaimReceipt) {
    assert_eq!(receipt.retirement(), BlobReclaimRetirement::Completed);
    assert!(receipt.bytes_released() > 0);
    assert_eq!(
        receipt.bytes_released(),
        receipt
            .displaced_extents()
            .iter()
            .map(|extent| extent.range().length())
            .sum::<u64>()
    );
}

fn assert_invisible(
    serving: &ServingPhysicalRuntime,
    scope: &AdmittedBlobScope,
    published: PublishedBlobGeneration,
) {
    let result = serving.blobs().unwrap().resolve_publication(
        published.object().bytes(),
        published.generation().sequence(),
        scope,
        BlobReadLimits::new(NonZeroU64::new(1).unwrap()),
    );
    match result {
        Err(BlobReadOpenFailure::PublicationNotFound) => {}
        Err(BlobReadOpenFailure::Read(BlobReadFailure::RecordRead(error)))
            if error.denial() == RecordReadDenial::RecordNotFound => {}
        other => panic!("released publication has wrong visibility: {other:?}"),
    }
}

fn assert_bytes_exact(
    serving: &ServingPhysicalRuntime,
    scope: &AdmittedBlobScope,
    published: PublishedBlobGeneration,
    expected: &[u8],
) -> u64 {
    let blobs = serving.blobs().unwrap();
    let mut read = blobs
        .read(
            published,
            scope,
            0,
            expected.len() as u64,
            BlobReadLimits::new(NonZeroU64::new(1).unwrap()),
        )
        .unwrap();
    let mut bytes = vec![0_u8; expected.len()];
    let mut used = 0;
    while used < bytes.len() {
        let count = read.read_next(&mut bytes[used..]).unwrap();
        assert!(count > 0);
        used += count;
    }
    assert_eq!(read.read_next(&mut bytes).unwrap(), 0);
    assert_eq!(&bytes, expected);
    assert_eq!(read.observation().touched_chunks(), 2);
    read.observation().reuse_source_selected_reads()
}

fn multi_chunk_payload() -> Vec<u8> {
    let mut bytes = vec![0x5a; CHUNK + 17];
    bytes[CHUNK..].fill(0x7c);
    bytes
}

fn assert_selected_reuse_claim(serving: &ServingPhysicalRuntime) {
    assert_eq!(
        super::super::blob_frontier::selected_blob_records(serving)
            .iter()
            .filter(|(_, bytes)| matches!(
                decode_blob_record(bytes),
                Ok(BlobRecordV1::ChunkReuseClaimV2(_))
            ))
            .count(),
        2
    );
}
