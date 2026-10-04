//! A retirement edge is pinned to the exact root bytes its intent names, both
//! when the edge is admitted and when the history advances over it.

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    durable_artifact_checksum, CurrentPhysicalRecordPlacement, DurableExtentRecordPlacement,
    DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest, ExtentArenaId, ExtentArenaRange,
    PersistedRecordIdentity, PhysicalExtentId, PhysicalFreeSpaceMembershipBlock,
    PhysicalGeneration, PhysicalGenerationAuthority, PhysicalPageId,
    PhysicalRecordFormatDeclaration, PhysicalSegmentId, RecordFreeSpaceManifestEntry,
    RecordSegmentPageManifestEntry, SelectedRecordContentClass, SelectedRecordRouteMetadata,
};
use worth_store_wal::{LogSequenceNumber, WalLsnRange};

use super::super::OrderedRootHistoryBuilder;
use super::{
    decide_ordered_root_step_basis as decide, CheckpointRetiredReleaseIntent, OrderedRootStepBasis,
    RetirementReleaseIntent, VerifiedRetirementRootEdge,
};
use crate::source_precedence::{ordinary_root_step::transcript, ReleasedInventoryView};
use crate::OrderedRootHistoryDenial as Denial;

const CUTOFF: u64 = 16;
const CHECKPOINT: u64 = 14;
const LIMIT: u64 = 16;

struct Root {
    root: DurablePhysicalRootManifest,
    free: DurableFreeSpaceManifestHeader,
    entries: Vec<RecordFreeSpaceManifestEntry>,
}

impl Root {
    fn view(&self) -> ReleasedInventoryView<'_> {
        ReleasedInventoryView::new(&self.root, &self.free, &[], &[], &self.entries)
    }
    fn sha256(&self) -> [u8; 32] {
        Sha256::digest(self.root.encode(format())).into()
    }
}

fn format() -> PhysicalRecordFormatDeclaration {
    PhysicalRecordFormatDeclaration::builder().admit().unwrap()
}

fn free_entry(offset: u64, length: u64, generation: u64) -> RecordFreeSpaceManifestEntry {
    let range = ExtentArenaRange::new(ExtentArenaId::new(1).unwrap(), offset, length).unwrap();
    RecordFreeSpaceManifestEntry::arena_range(range, generation).unwrap()
}

/// Each release frees one more 4 KiB range; `next_block` changes only root bytes.
fn root(generation: u64, next_block: u64) -> Root {
    let format = format();
    let freed = generation - CHECKPOINT;
    let entries = vec![free_entry(8192, 4096 * (freed + 1), CHECKPOINT)];
    let block =
        PhysicalFreeSpaceMembershipBlock::leaf(7, generation, 1, entries.clone(), 4).unwrap();
    let free = DurableFreeSpaceManifestHeader::new(
        generation,
        7,
        4,
        4,
        1,
        1,
        1,
        3,
        2,
        20480,
        4096,
        2,
        Some(block.reference(durable_artifact_checksum(&block.encode(format)))),
    )
    .unwrap();
    let root = DurablePhysicalRootManifest::builder(
        generation,
        7,
        4,
        durable_artifact_checksum(&free.encode(format)),
    )
    .free_space_root(free.root())
    .next_block(next_block)
    .admit()
    .unwrap()
    .with_maintenance_protocol();
    Root {
        root,
        free,
        entries,
    }
}

fn basis(source: u64, start: u64, digest: [u8; 32]) -> CheckpointRetiredReleaseIntent {
    let lsn = WalLsnRange::new(
        LogSequenceNumber::new(start),
        LogSequenceNumber::new(start + 1),
    )
    .unwrap();
    let intent = RetirementReleaseIntent::new(lsn, source, source + 1, digest);
    match decide(source, true, CUTOFF, false, &[intent]) {
        Ok(OrderedRootStepBasis::RetirementIntent(basis)) => basis,
        other => panic!("expected a retirement basis, got {other:?}"),
    }
}

fn edge(source: &Root, result: &Root, start: u64) -> VerifiedRetirementRootEdge {
    let basis = basis(source.root.generation(), start, result.sha256());
    VerifiedRetirementRootEdge::admit(source.view(), result.view(), basis, format(), LIMIT).unwrap()
}

fn builder(checkpoint: &Root, cutoff: u64) -> OrderedRootHistoryBuilder {
    let topology = transcript(checkpoint.view(), format(), LIMIT).unwrap();
    OrderedRootHistoryBuilder::begin(
        &checkpoint.root,
        &checkpoint.free,
        checkpoint.sha256(),
        cutoff,
        topology,
        format(),
        4,
        u64::MAX,
    )
    .unwrap()
}

fn advance(
    history: &mut OrderedRootHistoryBuilder,
    edge: VerifiedRetirementRootEdge,
    result: &Root,
) -> Result<(), Denial> {
    history.advance_retirement(edge, &result.root, &result.free, format())
}

#[test]
fn admit_requires_the_exact_candidate_root_and_unchanged_membership() {
    let (source, result) = (root(CHECKPOINT, 2), root(CHECKPOINT + 1, 2));
    let admit = |source: ReleasedInventoryView<'_>, result: ReleasedInventoryView<'_>, basis| {
        VerifiedRetirementRootEdge::admit(source, result, basis, format(), LIMIT)
    };
    let exact = basis(CHECKPOINT, 15, result.sha256());
    assert!(admit(source.view(), result.view(), exact).is_ok());

    let other_bytes = root(CHECKPOINT + 1, 3);
    assert_eq!(
        admit(source.view(), other_bytes.view(), exact),
        Err(Denial::Source)
    );
    let wrong_digest = basis(CHECKPOINT, 15, [7; 32]);
    assert_eq!(
        admit(source.view(), result.view(), wrong_digest),
        Err(Denial::Source)
    );
    // The intent names generation 15; a byte-exact root at 16 is not it.
    let skipped = root(CHECKPOINT + 2, 2);
    let named_skip = basis(CHECKPOINT, 15, skipped.sha256());
    assert_eq!(
        admit(source.view(), skipped.view(), named_skip),
        Err(Denial::Source)
    );
    let later_source = root(CHECKPOINT + 1, 2);
    assert_eq!(
        admit(later_source.view(), result.view(), exact),
        Err(Denial::Source)
    );

    let routes = [route()];
    let routed = ReleasedInventoryView {
        routes: &routes,
        ..result.view()
    };
    assert_eq!(admit(source.view(), routed, exact), Err(Denial::Effect));
    let segments = [segment()];
    let segmented = ReleasedInventoryView {
        segments: &segments,
        ..result.view()
    };
    assert_eq!(admit(source.view(), segmented, exact), Err(Denial::Effect));
}

#[test]
fn history_hashes_each_retirement_result_root() {
    let checkpoint = root(CHECKPOINT, 2);
    let (result, other_bytes) = (root(CHECKPOINT + 1, 2), root(CHECKPOINT + 1, 3));
    let admitted = edge(&checkpoint, &result, 15);
    // The edge was admitted for `result`; a root with other bytes at the same
    // generation and inventory may not stand in for it.
    let mut history = builder(&checkpoint, CUTOFF);
    assert_eq!(
        advance(&mut history, admitted, &other_bytes),
        Err(Denial::Source)
    );
    advance(&mut history, admitted, &result).unwrap();
    assert!(history.finish(&result.root, &result.free, format()).is_ok());
}

#[test]
fn history_admits_retirements_only_at_the_checkpoint_cutoff() {
    let checkpoint = root(CHECKPOINT, 2);
    let result = root(CHECKPOINT + 1, 2);
    let admitted = edge(&checkpoint, &result, 15);
    let mut later_cutoff = builder(&checkpoint, CUTOFF + 1);
    assert_eq!(
        advance(&mut later_cutoff, admitted, &result),
        Err(Denial::Source)
    );
    // A member edge moves the WAL position past the cutoff; no retirement
    // edge may follow it.
    let mut after_member = builder(&checkpoint, CUTOFF);
    after_member.previous_lsn_end = CUTOFF + 2;
    assert_eq!(
        advance(&mut after_member, admitted, &result),
        Err(Denial::Source)
    );
}

#[test]
fn consecutive_retirements_chain_in_generation_and_wal_order() {
    let checkpoint = root(CHECKPOINT, 2);
    let (first_root, second_root) = (root(CHECKPOINT + 1, 2), root(CHECKPOINT + 2, 2));
    let first = edge(&checkpoint, &first_root, 14);
    let second = edge(&first_root, &second_root, 15);
    let mut history = builder(&checkpoint, CUTOFF);
    advance(&mut history, first, &first_root).unwrap();
    assert!(history.retirement_prefix());
    advance(&mut history, second, &second_root).unwrap();
    let verified = history
        .finish(&second_root.root, &second_root.free, format())
        .unwrap();
    assert_eq!(verified.edges().len(), 2);

    // The second intent must follow the first in WAL order.
    let early_first = edge(&checkpoint, &first_root, 15);
    let early_second = edge(&first_root, &second_root, 14);
    let mut reordered = builder(&checkpoint, CUTOFF);
    advance(&mut reordered, early_first, &first_root).unwrap();
    assert_eq!(
        advance(&mut reordered, early_second, &second_root),
        Err(Denial::WalOrder)
    );
    // An edge may not skip its predecessor's result.
    let mut skipping = builder(&checkpoint, CUTOFF);
    assert_eq!(
        advance(&mut skipping, second, &second_root),
        Err(Denial::Source)
    );
}

fn route() -> CurrentPhysicalRecordPlacement {
    let record = PersistedRecordIdentity::new([4; 16], 1).unwrap();
    let cell = PhysicalGenerationAuthority::for_canonical_physical_format()
        .record_extent_cell(PhysicalExtentId::from_raw(1).unwrap())
        .with_extent_generation(PhysicalGeneration::from_raw(1).unwrap());
    let range = ExtentArenaRange::new(ExtentArenaId::new(1).unwrap(), 0, 4096).unwrap();
    let metadata =
        SelectedRecordRouteMetadata::primary(SelectedRecordContentClass::Opaque).unwrap();
    CurrentPhysicalRecordPlacement::Extent(
        DurableExtentRecordPlacement::new_selected(record, cell, 100, range, metadata).unwrap(),
    )
}

fn segment() -> RecordSegmentPageManifestEntry {
    let authority = PhysicalGenerationAuthority::for_canonical_physical_format();
    let id = PhysicalSegmentId::from_raw(1).unwrap();
    let generation = PhysicalGeneration::from_raw(1).unwrap();
    let cell = authority
        .segment_cell(id)
        .with_segment_generation(generation);
    let page = authority
        .page_cell(id, PhysicalPageId::from_raw(2).unwrap())
        .with_page_generation(generation);
    RecordSegmentPageManifestEntry::new(page, cell, 1, 0).unwrap()
}
