//! A historical release above a HeadV2 checkpoint is anchored only by the
//! exact checkpoint-source head of its object. The head cases mirror the
//! Store owner table in the `pending_wal_release::lineage` tests.

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    BlobGenerationPublicationV1, BlobReclaimDescriptorV2, BlobReclaimSourceBasisV1,
    BlobReclaimSourceKind, CurrentPhysicalRecordPlacement, DurableExtentRecordPlacement,
    ExtentArenaId, ExtentArenaRange, PersistedRecordIdentity, PhysicalExtentId, PhysicalGeneration,
    PhysicalGenerationAuthority, ReleaseCustodyHeadEntryV1, ReleaseCustodyHeadKeyV1,
    ReleasedDropPredecessorV1, ReleasedGenerationReclaimBasisV1, SelectedRecordContentClass,
    SelectedRecordRouteMetadata,
};

use super::super::historical_publication::{HistoricalFailure, RootRouteInventory};
use super::closure_evidence::ReleasedClosureEvidence;
use super::continuation::VerifiedCheckpointSourceAbsence;
use super::head_predecessor::{checkpoint_head, extends_checkpoint_head, head_predecessor_matches};
use super::historical_predecessors::{authenticate_chain, HeadAnchoredBatch};
use crate::entry::PhysicalRecoveryLimitDimension::ManifestEntries;
use crate::orchestration::recovery_budget::{allowance_for_test, RecoveryAllowance};

const STORE: [u8; 16] = [1; 16];
const CHECKPOINT_GENERATION: u64 = 6;
/// Far more entries than any chain here retains.
const ENTRIES: RecoveryAllowance = allowance_for_test(ManifestEntries, 16);

fn record(ordinal: u64) -> PersistedRecordIdentity {
    PersistedRecordIdentity::new([4; 16], ordinal).unwrap()
}

fn source(object: u8) -> ReleasedGenerationReclaimBasisV1 {
    let publication = BlobGenerationPublicationV1::new(
        STORE,
        [2; 16],
        [object; 16],
        4,
        record(2),
        [5; 32],
        64 << 10,
        [6; 32],
        64 << 10,
        [7; 32],
    )
    .unwrap();
    ReleasedGenerationReclaimBasisV1::new(
        publication,
        record(1),
        Sha256::digest(publication.encode()).into(),
        [8; 32],
    )
    .unwrap()
}

fn digest(source: ReleasedGenerationReclaimBasisV1) -> [u8; 32] {
    BlobReclaimSourceBasisV1::ReleasedGeneration(source).digest(STORE)
}

fn head(
    source: ReleasedGenerationReclaimBasisV1,
    basis_digest: [u8; 32],
    cumulative: u64,
    terminal: bool,
) -> ReleaseCustodyHeadEntryV1 {
    ReleaseCustodyHeadEntryV1::new(
        ReleaseCustodyHeadKeyV1::new(source.object(), source.generation()).unwrap(),
        record(10),
        [0x10; 32],
        record(11),
        [0x11; 32],
        record(12),
        [0x12; 32],
        basis_digest,
        None,
        CHECKPOINT_GENERATION - 1,
        cumulative,
        terminal,
    )
    .unwrap()
}

/// A two-record batch built on root 9 that names `prior` as its predecessor.
fn successor(
    source: ReleasedGenerationReclaimBasisV1,
    prior: ReleaseCustodyHeadEntryV1,
    cumulative: u64,
) -> BlobReclaimDescriptorV2 {
    let predecessor =
        ReleasedDropPredecessorV1::new(prior.descriptor_record(), prior.descriptor_frame_sha256())
            .unwrap();
    BlobReclaimDescriptorV2::new(
        STORE,
        [9; 16],
        BlobReclaimSourceKind::ReleasedGeneration,
        digest(source),
        record(20),
        [0x20; 32],
        2,
        9,
        10,
        Some(predecessor),
        cumulative,
        false,
    )
    .unwrap()
}

fn route(ordinal: u64) -> CurrentPhysicalRecordPlacement {
    let cell = PhysicalGenerationAuthority::for_canonical_physical_format()
        .record_extent_cell(PhysicalExtentId::from_raw(ordinal).unwrap())
        .with_extent_generation(PhysicalGeneration::from_raw(1).unwrap());
    let range = ExtentArenaRange::new(ExtentArenaId::new(ordinal).unwrap(), 0, 64).unwrap();
    let metadata =
        SelectedRecordRouteMetadata::primary(SelectedRecordContentClass::Opaque).unwrap();
    CurrentPhysicalRecordPlacement::Extent(
        DurableExtentRecordPlacement::new_selected(record(ordinal), cell, 100, range, metadata)
            .unwrap(),
    )
}

#[test]
fn head_predecessor_denies_wrong_digest_terminal_and_wrong_cumulative() {
    let source = source(3);
    let exact = head(source, digest(source), 3, false);
    let current = successor(source, exact, 5);
    assert!(head_predecessor_matches(exact, current, 2));
    let wrong_digest = head(source, [0x77; 32], 3, false);
    assert!(!head_predecessor_matches(wrong_digest, current, 2));
    let terminal = head(source, digest(source), 3, true);
    assert!(!head_predecessor_matches(terminal, current, 2));
    let wrong_cumulative = head(source, digest(source), 4, false);
    assert!(!head_predecessor_matches(wrong_cumulative, current, 2));
    // The checkpoint heads another object only: this key has no anchor.
    assert_eq!(checkpoint_head(&[exact], self::source(9)), None);
    assert_eq!(checkpoint_head(&[exact], source), Some(exact));
}

#[test]
fn batch_built_on_the_checkpoint_source_root_itself_extends_its_head() {
    let source = source(3);
    let exact = head(source, digest(source), 3, false);
    // `successor` builds on root 9.
    let current = successor(source, exact, 5);
    assert_eq!(current.source_root_generation(), 9);
    // The checkpoint was taken at root 9 and the release leaves that root.
    assert!(extends_checkpoint_head(exact, current, 2, 9));
    // Or the release leaves a later root than the checkpoint source.
    assert!(extends_checkpoint_head(
        exact,
        current,
        2,
        CHECKPOINT_GENERATION
    ));
    // A batch built below the checkpoint source root never extends its head.
    assert!(!extends_checkpoint_head(exact, current, 2, 10));
    // The root bound never excuses the predecessor join itself.
    assert!(!extends_checkpoint_head(exact, current, 3, 9));
}

#[test]
fn chain_leaving_the_ordered_releases_needs_the_replayed_checkpoint_head() {
    let source = source(3);
    let exact = head(source, digest(source), 3, false);
    let current = successor(source, exact, 5);
    let anchored = authenticate_chain(current, source, 2, Some(exact), &[], &[exact], ENTRIES)
        .expect("exact checkpoint-source head anchors the chain");
    assert!(anchored.retained_same_key.is_empty());
    let batch = HeadAnchoredBatch {
        descriptor: current,
        count: 2,
    };
    assert_eq!(anchored.anchor, Some(batch));
    // No checkpoint head for the object, as under a NoRelease checkpoint.
    assert_eq!(
        authenticate_chain(current, source, 2, Some(exact), &[], &[], ENTRIES),
        Err(HistoricalFailure::Invalid)
    );
    // The release replayed no prior, or a different prior, for its head.
    assert_eq!(
        authenticate_chain(current, source, 2, None, &[], &[exact], ENTRIES),
        Err(HistoricalFailure::Invalid)
    );
    let other = head(source, digest(source), 4, false);
    assert_eq!(
        authenticate_chain(current, source, 2, Some(other), &[], &[exact], ENTRIES),
        Err(HistoricalFailure::Invalid)
    );
    for mismatch in [
        head(source, [0x77; 32], 3, false),
        head(source, digest(source), 3, true),
        head(source, digest(source), 4, false),
    ] {
        assert_eq!(
            authenticate_chain(
                current,
                source,
                2,
                Some(mismatch),
                &[],
                &[mismatch],
                ENTRIES
            ),
            Err(HistoricalFailure::Invalid)
        );
    }
}

#[test]
fn head_anchored_closure_denies_foreign_removal_and_unexplained_absence() {
    // Record 30 was still routed at the checkpoint source root.
    let residual = VerifiedCheckpointSourceAbsence::for_test(RootRouteInventory::for_test(vec![
        route(20),
        route(30),
    ]));
    let retained_same_key = [record(40)];
    let evidence = ReleasedClosureEvidence::CheckpointHeadAnchored {
        residual: &residual,
        retained_same_key: &retained_same_key,
    };
    assert!(evidence.has_predecessor());
    // Absent at the checkpoint source root: the head custody settles it.
    assert!(evidence.settles_absent_child(record(60)));
    // Dropped above the checkpoint by a retained batch of this same key.
    assert!(evidence.settles_absent_child(record(40)));
    // Routed at the checkpoint and gone at the release source without a
    // retained batch of this key: the edge of another key removed it, which
    // is never the custody of this head.
    assert!(!evidence.settles_absent_child(record(30)));
    // The graph root is settled by the retained drops of this key only.
    assert!(evidence.settles_absent_root(record(40)));
    assert!(!evidence.settles_absent_root(record(60)));
}
