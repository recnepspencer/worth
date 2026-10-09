use std::num::NonZeroU64;

use worth_store_contracts::DurableArtifactFamilyId;
use worth_store_physical_format::{
    BlobGenerationPublicationV1, BlobTreeEntryV1, BlobTreeNodeKind, BlobTreeNodeV1,
    PersistedRecordIdentity, BLOB_TREE_NODE_FRAME_MAX_BYTES,
};
use worth_store_physical_integrity::{PhysicalArtifactScope, PhysicalByteRange};

use crate::physical_runtime::{
    integrity::SelectedRecordScrubBasis,
    layout::{PhysicalIndexPointKey, PhysicalLayoutAccess},
    BlobPhysicalAllocation, PhysicalIntegrityScrubRequestDenial, PhysicalIntegrityScrubTarget,
    PhysicalRecordId, PhysicalRecordReader, RecordByteLimit, RecordReadLimits,
    ServingPhysicalRuntime,
};

use super::{AdmittedBlobScope, BlobReadFailure, BlobReadOpenFailure, PublishedBlobGeneration};

const MAX_TREE_DEPTH: u8 = 7;
const TREE_FANOUT: u64 = 4096;
const PUBLICATION_FRAME_BYTES: u32 = 48 + 188;
const SCRUB_SELECTION_CHARGE: u64 = 4096 + 7 * 512 * 1024 + 1024 * 1024;

#[derive(Debug)]
pub enum BlobScrubTargetFailure {
    Selection(BlobReadOpenFailure),
    InvalidDepth,
    TreeDamaged,
    Target(PhysicalIntegrityScrubRequestDenial),
}

impl From<BlobReadOpenFailure> for BlobScrubTargetFailure {
    fn from(value: BlobReadOpenFailure) -> Self {
        Self::Selection(value)
    }
}

struct SelectedPublication<'runtime> {
    reader: PhysicalRecordReader,
    record: PersistedRecordIdentity,
    _allocation: BlobPhysicalAllocation<'runtime>,
}

pub(super) fn publication_target(
    runtime: &ServingPhysicalRuntime,
    object: [u8; 16],
    generation: u64,
) -> Result<PhysicalIntegrityScrubTarget, BlobScrubTargetFailure> {
    let selected = selected_catalog_value(runtime, object, generation)?;
    let scope = PhysicalArtifactScope::blob_generation_publication(
        selected.reader.store_identity(),
        selected.record,
        PhysicalByteRange::new(0, u64::from(PUBLICATION_FRAME_BYTES))
            .expect("positive publication ceiling"),
    );
    PhysicalIntegrityScrubTarget::selected_record(
        scope,
        selected.reader.protected_root(),
        SelectedRecordScrubBasis::BlobGenerationPublication { object, generation },
    )
    .map_err(BlobScrubTargetFailure::Target)
}

pub(super) fn tree_node_target(
    runtime: &ServingPhysicalRuntime,
    published: PublishedBlobGeneration,
    scope: &AdmittedBlobScope,
    offset: u64,
    depth: u8,
) -> Result<PhysicalIntegrityScrubTarget, BlobScrubTargetFailure> {
    if depth >= MAX_TREE_DEPTH {
        return Err(BlobScrubTargetFailure::InvalidDepth);
    }
    if published.store() != runtime.store_identity() {
        return Err(BlobReadOpenFailure::ForeignStore.into());
    }
    let selected = selected_catalog_value(
        runtime,
        published.object().bytes(),
        published.generation().sequence(),
    )?;
    let frame = read_selected_record(&selected.reader, selected.record, PUBLICATION_FRAME_BYTES)?;
    let publication = BlobGenerationPublicationV1::decode(&frame)
        .map_err(BlobReadOpenFailure::PublicationDamaged)?;
    if publication.store() != runtime.store_identity().bytes()
        || publication.object() != published.object().bytes()
        || publication.generation() != published.generation().sequence()
        || publication.session() != published.session().bytes()
    {
        return Err(BlobReadOpenFailure::ConflictingPublication.into());
    }
    if publication.key_scope() != scope.fingerprint() {
        return Err(BlobReadOpenFailure::ScopeMismatch.into());
    }
    if offset >= publication.total_bytes() {
        return Err(BlobReadOpenFailure::RangeOutOfBounds.into());
    }
    let mut edge = NodeEdge {
        record: publication.root_record(),
        digest: publication.root_digest(),
        covered_bytes: publication.total_bytes(),
        level: root_level(publication.total_bytes(), publication.chunk_size())
            .ok_or(BlobScrubTargetFailure::TreeDamaged)?,
        index: 0,
        root: true,
    };
    let mut base = 0_u64;
    for _ in 0..depth {
        let frame = read_selected_record(
            &selected.reader,
            edge.record,
            BLOB_TREE_NODE_FRAME_MAX_BYTES as u32,
        )?;
        let node =
            BlobTreeNodeV1::decode(&frame).map_err(|_| BlobScrubTargetFailure::TreeDamaged)?;
        if !edge.matches_node(&node, publication.store(), publication.session())
            || node.occurrence().kind() != BlobTreeNodeKind::Interior
        {
            return Err(BlobScrubTargetFailure::TreeDamaged);
        }
        let (child, child_base, child_position) =
            selected_child(&node, base, offset).ok_or(BlobScrubTargetFailure::TreeDamaged)?;
        let level = node
            .occurrence()
            .level()
            .checked_sub(1)
            .ok_or(BlobScrubTargetFailure::TreeDamaged)?;
        edge = NodeEdge {
            record: child.record(),
            digest: child.digest(),
            covered_bytes: child.covered_bytes(),
            level,
            index: node
                .occurrence()
                .index()
                .checked_mul(TREE_FANOUT)
                .and_then(|base| base.checked_add(child_position))
                .ok_or(BlobScrubTargetFailure::TreeDamaged)?,
            root: false,
        };
        base = child_base;
    }
    let node_scope = PhysicalArtifactScope::blob_tree_node(
        selected.reader.store_identity(),
        edge.record,
        PhysicalByteRange::new(0, BLOB_TREE_NODE_FRAME_MAX_BYTES as u64)
            .expect("positive tree frame ceiling"),
    );
    PhysicalIntegrityScrubTarget::selected_record(
        node_scope,
        selected.reader.protected_root(),
        SelectedRecordScrubBasis::BlobTreeNode {
            session: publication.session(),
            level: edge.level,
            index: edge.index,
            digest: edge.digest,
            covered_bytes: edge.covered_bytes,
            root_digest: edge.root,
        },
    )
    .map_err(BlobScrubTargetFailure::Target)
}

struct NodeEdge {
    record: PersistedRecordIdentity,
    digest: [u8; 32],
    covered_bytes: u64,
    level: u8,
    index: u64,
    root: bool,
}

impl NodeEdge {
    fn matches_node(&self, node: &BlobTreeNodeV1, store: [u8; 16], session: [u8; 16]) -> bool {
        let occurrence = node.occurrence();
        occurrence.store() == store
            && occurrence.session() == session
            && occurrence.level() == self.level
            && occurrence.index() == self.index
            && node.covered_bytes() == self.covered_bytes
            && if self.root {
                node.frame_digest() == self.digest
            } else {
                node.canonical_digest() == self.digest
            }
    }
}

fn selected_child(
    node: &BlobTreeNodeV1,
    base: u64,
    offset: u64,
) -> Option<(BlobTreeEntryV1, u64, u64)> {
    let mut start = base;
    for (position, entry) in node.entries().iter().enumerate() {
        let end = start.checked_add(entry.covered_bytes())?;
        if (start..end).contains(&offset) {
            return Some((*entry, start, position as u64));
        }
        start = end;
    }
    None
}

fn root_level(total_bytes: u64, chunk_size: u32) -> Option<u8> {
    let chunks = total_bytes.checked_add(u64::from(chunk_size) - 1)? / u64::from(chunk_size);
    let mut capacity = TREE_FANOUT;
    for level in 0..MAX_TREE_DEPTH {
        if chunks <= capacity {
            return Some(level);
        }
        capacity = capacity.checked_mul(TREE_FANOUT)?;
    }
    None
}

fn selected_catalog_value(
    runtime: &ServingPhysicalRuntime,
    object: [u8; 16],
    generation: u64,
) -> Result<SelectedPublication<'_>, BlobScrubTargetFailure> {
    if object == [0; 16] || generation == 0 {
        return Err(BlobReadOpenFailure::InvalidRequestedIdentity.into());
    }
    let allocation = runtime
        .physical_allocations()
        .admit_blob(NonZeroU64::new(SCRUB_SELECTION_CHARGE).expect("positive scrub charge"))
        .map_err(BlobReadOpenFailure::Allocation)?;
    let reader = runtime
        .records()
        .map_err(BlobReadOpenFailure::RootProtection)?;
    let layouts =
        PhysicalLayoutAccess::from_reader(runtime, reader).map_err(BlobReadOpenFailure::Layout)?;
    let key =
        PhysicalIndexPointKey::selected_blob_catalog(runtime.store_identity(), object, generation)
            .map_err(BlobReadOpenFailure::InvalidPointKey)?;
    let record = layouts
        .btree(DurableArtifactFamilyId::BlobCatalog)
        .and_then(|tree| tree.point(key))
        .map_err(BlobReadOpenFailure::Layout)?
        .selected_record()
        .ok_or(BlobReadOpenFailure::PublicationNotFound)?;
    Ok(SelectedPublication {
        reader: layouts.into_reader(),
        record,
        _allocation: allocation,
    })
}

fn read_selected_record(
    reader: &PhysicalRecordReader,
    record: PersistedRecordIdentity,
    maximum_bytes: u32,
) -> Result<Vec<u8>, BlobScrubTargetFailure> {
    let mut read = reader
        .open(
            PhysicalRecordId::from_persisted(record),
            RecordReadLimits::new(RecordByteLimit::new(maximum_bytes).expect("positive bound")),
        )
        .map_err(|error| BlobReadOpenFailure::Read(BlobReadFailure::RecordRead(error)))?;
    let mut frame = Vec::new();
    frame
        .try_reserve_exact(maximum_bytes as usize)
        .map_err(|_| BlobReadOpenFailure::ScratchUnavailable)?;
    let mut scratch = [0_u8; 8192];
    loop {
        let count = read
            .read_next(&mut scratch)
            .map_err(|error| BlobReadOpenFailure::Read(BlobReadFailure::RecordStream(error)))?;
        if count == 0 {
            return Ok(frame);
        }
        if frame.len() + count > maximum_bytes as usize {
            return Err(BlobScrubTargetFailure::TreeDamaged);
        }
        frame.extend_from_slice(&scratch[..count]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_store_physical_format::BlobTreeOccurrenceV1;

    fn record(ordinal: u64) -> PersistedRecordIdentity {
        PersistedRecordIdentity::new([7; 16], ordinal).unwrap()
    }

    fn leaf(index: u64) -> BlobTreeNodeV1 {
        BlobTreeNodeV1::new(
            BlobTreeOccurrenceV1::new([1; 16], [2; 16], BlobTreeNodeKind::Leaf, 0, index).unwrap(),
            vec![BlobTreeEntryV1::new([3; 32], record(10 + index), 64 << 10).unwrap()],
        )
        .unwrap()
    }

    #[test]
    fn authenticated_parent_position_rejects_c11_valid_wrong_child_index() {
        let first = leaf(0);
        let second = leaf(1);
        let parent = BlobTreeNodeV1::new(
            BlobTreeOccurrenceV1::new([1; 16], [2; 16], BlobTreeNodeKind::Interior, 1, 0).unwrap(),
            vec![
                BlobTreeEntryV1::new(first.canonical_digest(), record(20), first.covered_bytes())
                    .unwrap(),
                BlobTreeEntryV1::new(
                    second.canonical_digest(),
                    record(21),
                    second.covered_bytes(),
                )
                .unwrap(),
            ],
        )
        .unwrap();
        let root = NodeEdge {
            record: record(22),
            digest: parent.frame_digest(),
            covered_bytes: parent.covered_bytes(),
            level: 1,
            index: 0,
            root: true,
        };
        assert!(root.matches_node(&parent, [1; 16], [2; 16]));
        let (selected, base, position) = selected_child(&parent, 0, 64 << 10).unwrap();
        assert_eq!((base, position), (64 << 10, 1));
        let child = NodeEdge {
            record: selected.record(),
            digest: selected.digest(),
            covered_bytes: selected.covered_bytes(),
            level: parent.occurrence().level() - 1,
            index: parent.occurrence().index() * TREE_FANOUT + position,
            root: false,
        };
        assert!(child.matches_node(&second, [1; 16], [2; 16]));
        let wrong = BlobTreeNodeV1::new(
            BlobTreeOccurrenceV1::new([1; 16], [2; 16], BlobTreeNodeKind::Leaf, 0, 8).unwrap(),
            second.entries().to_vec(),
        )
        .unwrap();
        let wrong = BlobTreeNodeV1::decode(&wrong.encode()).expect("wrong index is C.11-valid");
        assert_eq!(wrong.canonical_digest(), second.canonical_digest());
        assert!(!child.matches_node(&wrong, [1; 16], [2; 16]));
    }
}
