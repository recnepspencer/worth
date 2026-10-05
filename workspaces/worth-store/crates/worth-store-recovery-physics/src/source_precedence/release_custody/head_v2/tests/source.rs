use std::num::NonZeroU64;
use std::path::Path;

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    release_checkpoint_batch_records_digest_v1,
    store_namespace::{ProposedStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion},
    CheckpointBindingCompactionHeader, CheckpointCertificateKind, CheckpointRootBasis,
    CheckpointStreamEncoder, CheckpointWalSourceRange, DurablePhysicalRootManifest,
    DurableRootSelector, OriginalDropReservationRequestV1, PersistedRecordIdentity,
    PhysicalCheckpointIdentity, PhysicalCheckpointSource, PhysicalRecordFormatDeclaration,
    ReleaseCheckpointAccumulatorV1, ReleaseCheckpointAccumulatorV2, ReleaseCheckpointBatchV1,
    ReleaseCheckpointCertificateV1, ReleaseCustodyHeadBlockV1, ReleaseCustodyHeadEntryV1,
    ReleaseCustodyHeadKeyV1, ReleaseCustodyHeadRosterDigestV1, ReleasedDropCumulativeEvidenceV1,
    ReleasedDropWalFateWitnessV1, RootSelectorIdentity, RootSelectorRole,
};
use worth_store_physical_integrity::{
    validate_checkpoint_binding_compaction, validate_checkpoint_footer,
    validate_checkpoint_stream_header, validate_root_manifest,
    CheckpointBindingCompactionIntegrityValidation, CheckpointFooterIntegrityValidation,
    CheckpointFooterValidationBasis, CheckpointStreamHeaderIntegrityValidation,
    CheckpointStreamHeaderScopeIdentity, PhysicalArtifactScope, PhysicalByteRange,
    RootManifestIntegrityValidation, UntrustedPhysicalArtifact, VerifiedCheckpointStream,
};
use worth_store_wal::{
    inspect_verified_wal_segment, prepare_wal_frame_append, WalSegmentArtifactIdentity,
    WalSegmentGeneration, WalSegmentId,
};

use super::super::*;
use crate::{
    admit_physical_page_facts, admit_physical_wal_tail, observe_structured_physical_root_candidate,
    select_current_previous_root, select_physical_recovery_sources, PhysicalCheckpointBase,
    PhysicalRootSlotObservation, PhysicalWalFrameFacts, PhysicalWalSegmentCandidate,
};

pub(super) struct AdmittedSource {
    pub(super) selected: PhysicalSourceSelection,
    pub(super) checkpoint: VerifiedCheckpointStream,
    pub(super) root: DurablePhysicalRootManifest,
    pub(super) head: ReleaseCustodyHeadBlockV1,
    /// Every block of the head tree, the leaf first, each at its block number less one.
    pub(super) blocks: Vec<ReleaseCustodyHeadBlockV1>,
    pub(super) entry: ReleaseCustodyHeadEntryV1,
    pub(super) digest: [u8; 32],
}

pub(super) fn admitted_source() -> AdmittedSource {
    admitted_source_with_cutoff(20)
}

pub(super) fn admitted_source_with_cutoff(cutoff: u64) -> AdmittedSource {
    admitted_source_under(cutoff, 0)
}

/// The one head's leaf under `levels` single-child branches: a sound tree of
/// `levels + 1` blocks that holds one head.
pub(super) fn admitted_source_under(cutoff: u64, levels: u16) -> AdmittedSource {
    let format = record_format();
    let entry = head_entry(1);
    let head = ReleaseCustodyHeadBlockV1::leaf(7, 1, 1, vec![entry], format).unwrap();
    let mut blocks = vec![head];
    for level in 1..=levels {
        let child = blocks.last().unwrap().reference(format);
        let block = u64::from(level) + 1;
        blocks.push(
            ReleaseCustodyHeadBlockV1::branch(7, 1, block, level, vec![child], format).unwrap(),
        );
    }
    admitted_source_of(cutoff, entry, blocks)
}

/// A roster of one head whose tree is a level-one root over two leaves, the
/// second holding a head the roster does not count.
pub(super) fn admitted_source_over_two_leaves() -> AdmittedSource {
    let format = record_format();
    let entry = head_entry(1);
    let left = ReleaseCustodyHeadBlockV1::leaf(7, 1, 1, vec![entry], format).unwrap();
    let right = ReleaseCustodyHeadBlockV1::leaf(7, 1, 2, vec![head_entry(2)], format).unwrap();
    let children = vec![left.reference(format), right.reference(format)];
    let root = ReleaseCustodyHeadBlockV1::branch(7, 1, 3, 1, children, format).unwrap();
    admitted_source_of(20, entry, vec![left, right, root])
}

fn record(ordinal: u64) -> PersistedRecordIdentity {
    PersistedRecordIdentity::new([9; 16], ordinal).unwrap()
}

fn head_entry(key: u64) -> ReleaseCustodyHeadEntryV1 {
    ReleaseCustodyHeadEntryV1::new(
        ReleaseCustodyHeadKeyV1::new([9; 16], key).unwrap(),
        record(1),
        [1; 32],
        record(3),
        [3; 32],
        record(2),
        [2; 32],
        [4; 32],
        None,
        1,
        1,
        false,
    )
    .unwrap()
}

/// The source whose head tree is `blocks`, each at its block number less
/// one, the root last; the roster counts `entry` alone.
fn admitted_source_of(
    cutoff: u64,
    entry: ReleaseCustodyHeadEntryV1,
    blocks: Vec<ReleaseCustodyHeadBlockV1>,
) -> AdmittedSource {
    let format = record_format();
    let identity = PhysicalCheckpointIdentity::new(store(), NonZeroU64::new(1).unwrap());
    let head = blocks[0].clone();
    let root = DurablePhysicalRootManifest::builder(1, 7, 4, 19)
        .release_custody_head_root(Some(blocks.last().unwrap().reference(format)))
        .next_release_custody_head_block(blocks.len() as u64 + 1)
        .admit()
        .unwrap();
    let root_bytes = root.encode(format);
    let root_sha: [u8; 32] = Sha256::digest(&root_bytes).into();
    let mut roster = ReleaseCustodyHeadRosterDigestV1::new(root.release_custody_head_root(), 8);
    roster.push(entry).unwrap();
    let (count, digest) = roster.finish();
    let request = OriginalDropReservationRequestV1::new([5; 32], [6; 32], 1, 3).unwrap();
    let fate = ReleasedDropWalFateWitnessV1::new(10, 11, [7; 32], [8; 32]).unwrap();
    let step = ReleasedDropCumulativeEvidenceV1::new(
        record(1),
        [1; 32],
        [10; 32],
        record(2),
        [2; 32],
        fate,
        1,
        root_sha,
        None,
        1,
        false,
    )
    .unwrap();
    let (cumulative, cumulative_digest) = step.advance(0, [0; 32]).unwrap();
    let batch = ReleaseCheckpointBatchV1::new(
        identity,
        1,
        root_sha,
        0,
        record(1),
        [1; 32],
        [10; 32],
        record(2),
        [2; 32],
        request,
        fate,
        1,
        root_sha,
        None,
        cumulative,
        cumulative_digest,
        false,
    )
    .unwrap();
    let base = ReleaseCheckpointAccumulatorV1::new(
        identity,
        1,
        root_sha,
        0,
        [0; 32],
        [0; 32],
        0,
        [0; 32],
        1,
        release_checkpoint_batch_records_digest_v1(&[batch]).unwrap(),
        batch.tip_provenance().unwrap(),
        cumulative,
        cumulative_digest,
        false,
    )
    .unwrap();
    let accumulator = ReleaseCheckpointAccumulatorV2::new(base, count, digest, 0, [0; 32]).unwrap();
    let checkpoint_source = PhysicalCheckpointSource::concurrent(
        identity,
        CheckpointWalSourceRange::new(10, 20).unwrap(),
        CheckpointRootBasis::new(1, 7),
        1,
    )
    .with_maintenance_protocol();
    let verified = verified_checkpoint(checkpoint_source, batch, accumulator, cutoff);
    let selector = DurableRootSelector::new(
        store(),
        format,
        RootSelectorIdentity::new(1).unwrap(),
        RootSelectorRole::Current,
        1,
        None,
        None,
    )
    .unwrap();
    let selected_root = select_current_previous_root(
        observe_structured_physical_root_candidate(selector, root.clone(), format),
        PhysicalRootSlotObservation::Absent,
        None,
    )
    .unwrap();
    let scope = PhysicalArtifactScope::root_manifest(
        store(),
        format,
        1,
        PhysicalByteRange::new(0, root_bytes.len() as u64).unwrap(),
    )
    .unwrap();
    let RootManifestIntegrityValidation::Intact(validated_root) = validate_root_manifest(
        UntrustedPhysicalArtifact::from_bounded_bytes(&root_bytes),
        scope,
    )
    .0
    else {
        panic!("source root must validate")
    };
    let checkpoint =
        PhysicalCheckpointBase::admit(&selected_root, &verified, &validated_root).unwrap();
    let pages = admit_physical_page_facts(selected_root.selected(), Vec::new(), 1, 1).unwrap();
    let covered = prepare_wal_frame_append(
        Path::new("head-v2-checkpoint-covered"),
        1,
        1,
        10,
        20,
        "head-v2-checkpoint-cover",
        b"covered",
    )
    .unwrap();
    let segment = WalSegmentArtifactIdentity::new(
        WalSegmentId::new(1).unwrap(),
        WalSegmentGeneration::new(1).unwrap(),
    );
    let inspection = inspect_verified_wal_segment(segment, covered.encoded_frame())
        .unwrap()
        .inspection();
    let covered = PhysicalWalSegmentCandidate::from_frame_facts(
        inspection,
        None,
        vec![PhysicalWalFrameFacts::new(inspection.lsn_range(), inspection.byte_count()).unwrap()],
    )
    .unwrap();
    let tail = admit_physical_wal_tail(
        20,
        Some(cutoff),
        vec![covered],
        Vec::with_capacity(1),
        Vec::with_capacity(1),
    )
    .unwrap();
    let selected = select_physical_recovery_sources(
        selected_root,
        pages,
        None,
        Some(checkpoint),
        tail,
        None,
        Vec::new(),
    )
    .unwrap();
    AdmittedSource {
        selected,
        checkpoint: verified,
        root,
        head,
        blocks,
        entry,
        digest,
    }
}

fn verified_checkpoint(
    source: PhysicalCheckpointSource,
    batch: ReleaseCheckpointBatchV1,
    accumulator: ReleaseCheckpointAccumulatorV2,
    cutoff: u64,
) -> VerifiedCheckpointStream {
    let (encoder, header) = CheckpointStreamEncoder::begin_certified(source);
    let (mut encoder, compaction) = encoder
        .begin_binding_compaction(CheckpointBindingCompactionHeader::new(1, cutoff).unwrap());
    let batch = encoder
        .encode_certificate_record(
            CheckpointCertificateKind::ReleasedDrop,
            &ReleaseCheckpointCertificateV1::Batch(batch).encode(),
        )
        .unwrap();
    let accumulator = encoder
        .encode_certificate_record(
            CheckpointCertificateKind::ReleasedDrop,
            &ReleaseCheckpointCertificateV1::AccumulatorV2(accumulator).encode(),
        )
        .unwrap();
    let (_, footer) = encoder.finish();
    let mut bytes = Vec::new();
    let mut append = |record: &[u8]| {
        let range = PhysicalByteRange::new(bytes.len() as u64, record.len() as u64).unwrap();
        bytes.extend_from_slice(record);
        range
    };
    let header_range = append(&header);
    let compaction_range = append(&compaction);
    let batch_range = append(&batch);
    let accumulator_range = append(&accumulator);
    let footer_range = append(&footer);
    let record = |range: PhysicalByteRange| {
        UntrustedPhysicalArtifact::from_bounded_bytes(
            &bytes[range.offset() as usize..range.end_exclusive() as usize],
        )
    };
    let header_scope = PhysicalArtifactScope::checkpoint_stream_header(
        CheckpointStreamHeaderScopeIdentity::staged(store()),
        header_range,
    );
    let compaction_scope =
        PhysicalArtifactScope::checkpoint_binding_compaction(source.identity(), compaction_range);
    let footer_scope = PhysicalArtifactScope::checkpoint_footer(source.identity(), footer_range);
    let CheckpointStreamHeaderIntegrityValidation::Intact(header) =
        validate_checkpoint_stream_header(record(header_range), header_scope).0
    else {
        panic!("checkpoint header must validate")
    };
    let CheckpointBindingCompactionIntegrityValidation::Intact(compaction) =
        validate_checkpoint_binding_compaction(record(compaction_range), compaction_scope).0
    else {
        panic!("checkpoint compaction must validate")
    };
    let certificates = [
        (
            batch_range,
            &bytes[batch_range.offset() as usize..batch_range.end_exclusive() as usize],
        ),
        (
            accumulator_range,
            &bytes[accumulator_range.offset() as usize..accumulator_range.end_exclusive() as usize],
        ),
    ];
    let CheckpointFooterIntegrityValidation::Intact(footer) = validate_checkpoint_footer(
        record(footer_range),
        footer_scope,
        CheckpointFooterValidationBasis::new(&header, &[], &compaction, &[])
            .with_certificates(&certificates),
    )
    .0
    else {
        panic!("certified footer must validate")
    };
    VerifiedCheckpointStream::assemble_from_validated_records_with_certificates(
        UntrustedPhysicalArtifact::from_bounded_bytes(&bytes),
        &header,
        &[],
        &compaction,
        &[],
        &certificates,
        &footer,
    )
    .unwrap()
}

pub(super) fn record_format() -> PhysicalRecordFormatDeclaration {
    PhysicalRecordFormatDeclaration::builder().admit().unwrap()
}

fn store() -> worth_store_physical_format::store_namespace::StableStoreIdentity {
    StoreNamespaceIdentityRecord::new(
        StoreNamespaceVersion::CURRENT,
        ProposedStoreIdentity::from_nonzero_bytes([7; 16]).unwrap(),
    )
    .published_identity()
}
