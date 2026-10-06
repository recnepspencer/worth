//! Selection before the first checkpoint: only the whole WAL from the
//! canonical origin, under a root no checkpoint anchored, is a source.

use std::path::Path;

use worth_store_physical_format::{
    store_namespace::{ProposedStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion},
    DurablePhysicalRootManifest, DurableRootSelector, PhysicalRecordFormatDeclaration,
    RootSelectorIdentity, RootSelectorRole,
};
use worth_store_wal::{inspect_verified_wal_segment, prepare_wal_frame_append, WAL_ORIGIN};

use super::*;
use crate::{
    admit_physical_page_facts, admit_physical_wal_tail, observe_structured_physical_root_candidate,
    select_current_previous_root, PhysicalRootSlotObservation, PhysicalWalFrameFacts,
    PhysicalWalSegmentCandidate,
};

const ORIGIN: u64 = WAL_ORIGIN.lsn().get();

#[test]
fn generation_zero_admits_the_whole_wal_from_the_canonical_origin() {
    let selected = select(root(3, None), &[(1, 1, ORIGIN, 9), (2, 1, 9, 12)]).unwrap();
    assert!(selected.checkpoint().is_none());
    assert_eq!(selected.wal_tail().segments().len(), 2);
    assert_eq!(selected.covered_wal_end_exclusive(), ORIGIN);
    assert!(!selected.trace().checkpoint_selected());
}

#[test]
fn generation_zero_admits_an_empty_wal_under_the_first_root() {
    let selected = select(root(1, None), &[]).unwrap();
    assert!(selected.wal_tail().segments().is_empty());
}

#[test]
fn generation_zero_denies_an_empty_wal_under_a_published_root() {
    assert_eq!(
        select(root(2, None), &[]).err(),
        Some(PhysicalSourceSelectionDenial::WalOmitsCanonicalOrigin)
    );
}

#[test]
fn generation_zero_denies_a_wal_not_starting_at_the_origin_segment() {
    assert_eq!(
        select(root(3, None), &[(2, 1, ORIGIN, 9)]).err(),
        Some(PhysicalSourceSelectionDenial::WalOmitsCanonicalOrigin)
    );
}

#[test]
fn generation_zero_denies_a_wal_not_of_the_origin_generation() {
    assert_eq!(
        select(root(3, None), &[(1, 2, ORIGIN, 9)]).err(),
        Some(PhysicalSourceSelectionDenial::WalOmitsCanonicalOrigin)
    );
}

#[test]
fn generation_zero_denies_a_wal_whose_segment_ends_before_the_origin() {
    assert_eq!(
        select(root(1, None), &[(1, 1, 0, ORIGIN)]).err(),
        Some(PhysicalSourceSelectionDenial::WalOmitsCanonicalOrigin)
    );
}

#[test]
fn generation_zero_denies_a_root_that_anchors_a_tier_epoch() {
    assert_eq!(
        select(root(3, Some([9; 32])), &[(1, 1, ORIGIN, 9)]).err(),
        Some(PhysicalSourceSelectionDenial::TierAnchorRequiresCheckpoint)
    );
}

#[test]
fn generation_zero_denies_a_tail_admitted_over_another_basis() {
    let root = root(3, None);
    let pages = admit_physical_page_facts(root.selected(), Vec::new(), 1, 1).unwrap();
    let tail = admit_physical_wal_tail(
        ORIGIN,
        Some(ORIGIN),
        vec![candidate(1, 1, ORIGIN, 9)],
        Vec::with_capacity(1),
        Vec::with_capacity(1),
    )
    .unwrap();
    assert_eq!(
        select_physical_recovery_sources(root, pages, None, None, tail, None, Vec::new()).err(),
        Some(PhysicalSourceSelectionDenial::WalCheckpointBasisMismatch)
    );
}

#[test]
fn generation_zero_denies_a_compaction_product() {
    let root = root(1, None);
    let pages = admit_physical_page_facts(root.selected(), Vec::new(), 1, 1).unwrap();
    let tail = admit_physical_wal_tail(ORIGIN, None, Vec::new(), Vec::new(), Vec::new()).unwrap();
    let compaction = super::super::release_custody::admitted_compaction_product();
    assert_eq!(
        select_physical_recovery_sources(
            root,
            pages,
            None,
            None,
            tail,
            Some(compaction),
            Vec::new()
        )
        .err(),
        Some(PhysicalSourceSelectionDenial::CompactionRequiresCheckpoint)
    );
}

#[test]
fn the_generation_zero_basis_is_the_canonical_origin_without_a_cutoff() {
    assert_eq!(checkpoint_wal_basis(None), (ORIGIN, None));
}

/// Selects over `segments`, each `(segment, generation, start, end)`, with no
/// checkpoint.
fn select(
    root: SelectedPhysicalRoot,
    segments: &[(u64, u64, u64, u64)],
) -> Result<PhysicalSourceSelection, PhysicalSourceSelectionDenial> {
    let pages = admit_physical_page_facts(root.selected(), Vec::new(), 1, 1).unwrap();
    let candidates = segments
        .iter()
        .map(|&(segment, generation, start, end)| candidate(segment, generation, start, end))
        .collect::<Vec<_>>();
    let (frontier, cutoff) = checkpoint_wal_basis(None);
    let tail = admit_physical_wal_tail(
        frontier,
        cutoff,
        candidates,
        Vec::with_capacity(segments.len()),
        Vec::with_capacity(segments.len()),
    )
    .unwrap();
    select_physical_recovery_sources(root, pages, None, None, tail, None, Vec::new())
}

fn candidate(segment: u64, generation: u64, start: u64, end: u64) -> PhysicalWalSegmentCandidate {
    let frame = prepare_wal_frame_append(
        Path::new("generation-zero-selection"),
        segment,
        generation,
        start,
        end,
        "generation-zero-frame",
        b"frame",
    )
    .unwrap();
    let identity = worth_store_wal::WalSegmentArtifactIdentity::new(
        worth_store_wal::WalSegmentId::new(segment).unwrap(),
        worth_store_wal::WalSegmentGeneration::new(generation).unwrap(),
    );
    let inspection = inspect_verified_wal_segment(identity, frame.encoded_frame())
        .unwrap()
        .inspection();
    PhysicalWalSegmentCandidate::from_frame_facts(
        inspection,
        None,
        vec![PhysicalWalFrameFacts::new(inspection.lsn_range(), inspection.byte_count()).unwrap()],
    )
    .unwrap()
}

fn root(generation: u64, tier_epoch_anchor: Option<[u8; 32]>) -> SelectedPhysicalRoot {
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let manifest = DurablePhysicalRootManifest::builder(generation, 7, 4, 19)
        .tier_epoch_anchor(tier_epoch_anchor)
        .admit()
        .unwrap();
    let store = StoreNamespaceIdentityRecord::new(
        StoreNamespaceVersion::CURRENT,
        ProposedStoreIdentity::from_nonzero_bytes([7; 16]).unwrap(),
    )
    .published_identity();
    let selector = DurableRootSelector::new(
        store,
        format,
        RootSelectorIdentity::new(1).unwrap(),
        RootSelectorRole::Current,
        generation,
        None,
        None,
    )
    .unwrap();
    select_current_previous_root(
        observe_structured_physical_root_candidate(selector, manifest, format),
        PhysicalRootSlotObservation::Absent,
        None,
    )
    .unwrap()
}
