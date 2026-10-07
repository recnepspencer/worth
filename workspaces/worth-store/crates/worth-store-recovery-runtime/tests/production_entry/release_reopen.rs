//! Real Store release, checkpoint, close, and separate-process recovery.

use super::*;
use std::num::NonZeroU32;
use worth_store::physical_runtime::{
    BlobReclaimReceipt, BlobReclaimRetirement, BlobReclaimRetirementBudget, RecordByteLimit,
    RecordCountLimit, RecordScanOutcome, RecordScanRequest, ServingPhysicalRuntime,
};
use worth_store_physical_format::{
    checkpoint_certificate_frame_bytes, decode_blob_record, decode_checkpoint_certificate,
    BlobReclaimSourceKind, BlobRecordV1, CheckpointCertificateKind, CheckpointStreamFooter,
    PersistedRecordIdentity, ReleaseCheckpointAccumulatorV2, ReleaseCheckpointBatchV1,
    ReleaseCheckpointCertificateV1, CHECKPOINT_CERTIFICATE_PREFIX_BYTES,
    CHECKPOINT_CERTIFIED_FOOTER_RECORD_BYTES, CHECKPOINT_CERTIFIED_SCHEMA,
};

#[path = "release_reopen/candidate_crash.rs"]
pub(super) mod candidate_crash;
#[path = "release_reopen/mixed_failed_ingest.rs"]
pub(super) mod mixed_failed_ingest;
#[cfg(feature = "certification-test-authority")]
#[path = "release_reopen/per_object_pruned_successor.rs"]
mod per_object_pruned_successor;
#[path = "release_reopen/published_world.rs"]
pub(super) mod published_world;
#[path = "release_reopen/selected_head_oracle.rs"]
pub(super) mod selected_head_oracle;
#[cfg(feature = "certification-test-authority")]
#[path = "release_reopen/shared_reuse_custody.rs"]
mod shared_reuse_custody;
#[path = "release_reopen/two_batch.rs"]
pub(super) mod two_batch;
#[path = "release_reopen/two_generations.rs"]
pub(super) mod two_generations;

pub(super) fn run(maximum_dropped_records: u16, expect_terminal: bool) -> std::process::Output {
    let (world, mut first, record) = released_world(maximum_dropped_records);
    let mut selected_first_tip = None;
    if expect_terminal {
        let (batches, accumulator) = selected_release_certificates(&world);
        if let Some(batch) = batches.last() {
            assert_eq!(accumulator.base().tip(), batch.tip_provenance().unwrap());
        }
        assert!(accumulator.base().terminal());
        selected_first_tip = Some(accumulator.base().tip());
        assert!(first.dropped_records().len() > 1);
        assert_eq!(first.remaining_payload_records(), 0);
        assert_selected_terminal_descriptor(world.serving());
        let expected_retired = first
            .displaced_extents()
            .iter()
            .map(|extent| extent.range().length())
            .sum::<u64>();
        assert!(
            expected_retired > 0,
            "terminal drop must displace real extents"
        );
        let first_checkpoint = PhysicalCheckpointRequest::fuzzy(
            PhysicalCheckpointIdempotencyKey::new([0x73; 32]),
            PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
        );
        let TransitionOutcome::Success(handle) = world
            .serving()
            .checkpoints()
            .start(first_checkpoint)
            .into_raw()
        else {
            panic!("first release-custody checkpoint must admit")
        };
        assert!(matches!(
            handle.wait(),
            PhysicalCheckpointOutcome::Completed(_)
        ));
        let (batches, accumulator) = selected_release_certificates(&world);
        assert!(
            batches.is_empty(),
            "successor checkpoint must carry the earlier Batch"
        );
        assert_eq!(Some(accumulator.base().tip()), selected_first_tip);
        assert!(accumulator.base().prior_checkpoint_sequence() > 0);
        let budget = BlobReclaimRetirementBudget::new(
            NonZeroU32::new(128).unwrap(),
            PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
        );
        for _ in 0..64 {
            if first.retirement() == BlobReclaimRetirement::Completed {
                break;
            }
            world
                .serving()
                .blobs()
                .unwrap()
                .continue_reclaim_retirement(&mut first, budget)
                .expect("continue terminal retirement");
        }
        assert_eq!(first.retirement(), BlobReclaimRetirement::Completed);
        assert_eq!(first.bytes_released(), expected_retired);
    } else {
        assert_eq!(first.dropped_records(), &[record]);
        assert!(first.remaining_payload_records() > 0);
    }
    let checkpoint = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([0x72; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let TransitionOutcome::Success(handle) =
        world.serving().checkpoints().start(checkpoint).into_raw()
    else {
        panic!("checkpoint after release must admit")
    };
    assert!(matches!(
        handle.wait(),
        PhysicalCheckpointOutcome::Completed(_)
    ));
    if expect_terminal {
        assert_selected_terminal_descriptor(world.serving());
        let (batches, accumulator) = selected_release_certificates(&world);
        assert!(
            batches.is_empty(),
            "later checkpoint must carry the earlier release, not replay Batch"
        );
        assert_eq!(Some(accumulator.base().tip()), selected_first_tip);
        assert!(accumulator.base().prior_checkpoint_sequence() > 0);
        assert!(
            accumulator.base().terminal(),
            "selected release tip must be terminal"
        );
    }
    let (_, selected) = selected_release_certificates(&world);
    let _selected_heads =
        selected_head_oracle::selected_heads(world.retained_root().path(), selected);
    let retained = world.retained_root();
    let root = retained.path().to_path_buf();
    drop(world);
    // The selected head's bounded control/tree closure alone exceeds the
    // 512 KiB C8 phase-two profile. Use the existing finite C11 envelope;
    // the phase-two runner and production allocation guard stay unchanged.
    Command::new(env!("CARGO_BIN_EXE_physical_store_recover"))
        .arg(&root)
        .arg("--bounded-profile=c11-blob-crash-v1")
        .output()
        .expect("run C11 release checkpoint recovery entry")
}

pub(super) fn released_world(
    maximum_dropped_records: u16,
) -> (
    PhysicalResidencyStoreWorld,
    BlobReclaimReceipt,
    PersistedRecordIdentity,
) {
    let (world, receipt, record, _) = released_world_in_tier(maximum_dropped_records, false, None);
    (world, receipt, record)
}

#[cfg(feature = "certification-test-authority")]
fn released_world_with_wal_segment_bytes(
    maximum_dropped_records: u16,
    wal_segment_bytes: NonZeroU64,
) -> (
    PhysicalResidencyStoreWorld,
    BlobReclaimReceipt,
    PersistedRecordIdentity,
    AdmittedBlobReleaseProof,
) {
    released_world_in_tier(maximum_dropped_records, false, Some(wal_segment_bytes))
}

pub(super) fn released_tier_world(
    maximum_dropped_records: u16,
) -> (
    PhysicalResidencyStoreWorld,
    BlobReclaimReceipt,
    PersistedRecordIdentity,
) {
    let (world, receipt, record, _) = released_world_in_tier(maximum_dropped_records, true, None);
    (world, receipt, record)
}

fn released_world_in_tier(
    maximum_dropped_records: u16,
    activate_tier: bool,
    wal_segment_bytes: Option<NonZeroU64>,
) -> (
    PhysicalResidencyStoreWorld,
    BlobReclaimReceipt,
    PersistedRecordIdentity,
    AdmittedBlobReleaseProof,
) {
    let (world, proof, record) = published_world::create(activate_tier, wal_segment_bytes, None);
    let request = BlobReclaimRequest::released(
        reissue_proof(&proof),
        world.placement(),
        PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
        BlobReclaimLimits::new(
            NonZeroU64::new(1024).unwrap(),
            NonZeroU64::new(64 << 20).unwrap(),
            NonZeroU16::new(maximum_dropped_records).unwrap(),
        )
        .unwrap(),
    );
    let first = world
        .serving()
        .blobs()
        .unwrap()
        .reclaim(request)
        .expect("release batch")
        .wait()
        .expect("release completed");
    assert_eq!(first.disposition(), BlobReclaimDisposition::Dropped);
    assert!(first.dropped_records().contains(&record));
    (world, first, record, proof)
}

fn reissue_proof(proof: &AdmittedBlobReleaseProof) -> AdmittedBlobReleaseProof {
    AdmittedBlobReleaseProof::certification_admit(
        proof.store(),
        proof.object(),
        proof.generation(),
        proof.publication_allocation_epoch(),
        proof.publication_record_ordinal(),
        proof.publication_frame_sha256(),
        proof.issuer_evidence_sha256(),
    )
    .expect("same genuine selected release evidence")
}

fn selected_release_certificates(
    world: &PhysicalResidencyStoreWorld,
) -> (
    Vec<ReleaseCheckpointBatchV1>,
    ReleaseCheckpointAccumulatorV2,
) {
    let bytes = std::fs::read(
        world
            .retained_root()
            .path()
            .join("families/checkpoint.current"),
    )
    .expect("selected checkpoint bytes");
    selected_release_certificates_from_bytes(&bytes)
}

pub(super) fn selected_release_certificates_from_bytes(
    bytes: &[u8],
) -> (
    Vec<ReleaseCheckpointBatchV1>,
    ReleaseCheckpointAccumulatorV2,
) {
    assert_eq!(bytes[8], CHECKPOINT_CERTIFIED_SCHEMA);
    let footer_start = bytes.len() - CHECKPOINT_CERTIFIED_FOOTER_RECORD_BYTES;
    let footer = CheckpointStreamFooter::decode_record(&bytes[footer_start..])
        .expect("selected certified checkpoint footer");
    let mut offset = footer_start - footer.certificate_record_bytes() as usize;
    let mut batches = Vec::new();
    let mut accumulator = None;
    for _ in 0..footer.certificate_record_count() {
        let frame_bytes = checkpoint_certificate_frame_bytes(
            &bytes[offset..offset + CHECKPOINT_CERTIFICATE_PREFIX_BYTES],
        )
        .expect("selected certificate frame prefix");
        let (kind, payload) = decode_checkpoint_certificate(&bytes[offset..offset + frame_bytes])
            .expect("selected certificate frame");
        if kind == CheckpointCertificateKind::ReleasedDrop {
            match ReleaseCheckpointCertificateV1::decode(payload)
                .expect("selected tag-7 certificate")
            {
                ReleaseCheckpointCertificateV1::Batch(value) => batches.push(value),
                ReleaseCheckpointCertificateV1::AccumulatorV2(value) => {
                    assert!(accumulator.replace(value).is_none());
                }
                ReleaseCheckpointCertificateV1::Accumulator(_) => {
                    panic!("released checkpoint cannot use headless V1 accumulator")
                }
                ReleaseCheckpointCertificateV1::NoRelease(_) => {
                    panic!("released checkpoint cannot carry NoRelease custody")
                }
            }
        }
        offset += frame_bytes;
    }
    assert_eq!(offset, footer_start);
    (batches, accumulator.expect("selected release accumulator"))
}

fn assert_selected_terminal_descriptor(serving: &ServingPhysicalRuntime) {
    let mut scan = serving
        .records()
        .unwrap()
        .scan(
            RecordScanRequest::from_start()
                .with_batch_limit(RecordCountLimit::new(8).unwrap())
                .with_payload_limit(RecordByteLimit::new(1024).unwrap()),
        )
        .unwrap();
    let mut scratch = [0; 8192];
    let mut found = 0;
    while let RecordScanOutcome::Batch(batch) = scan.read_next_into(&mut scratch).unwrap() {
        for index in 0..batch.records().len() {
            let Some(bytes) = batch.payload(index) else {
                continue;
            };
            let released_terminal = match decode_blob_record(bytes) {
                Ok(BlobRecordV1::ReclaimDescriptorV2(value)) => {
                    value.source_kind() == BlobReclaimSourceKind::ReleasedGeneration
                        && value.terminal()
                }
                Ok(BlobRecordV1::ReclaimDescriptorV3(value)) => value.base().terminal(),
                _ => false,
            };
            found += usize::from(released_terminal);
        }
        if batch.is_complete() {
            break;
        }
    }
    assert_eq!(found, 1, "one terminal descriptor remains selected");
}
