use sha2::{Digest, Sha256};
use worth_store_physical_format::{decode_blob_record, BlobRecordV1, BlobTreeNodeKind};

use crate::physical_runtime::PhysicalRecordReader;

use super::super::super::{scan, BlobReclaimFailure, BlobReclaimLimits};
use super::chain::ValidatedReleaseChain;
use super::graph_types::{leaf_occurrence_matches, ExpectedEdge, ExpectedKind};
use super::inventory::{SelectedBlobFact, SelectedReleaseInventory};
use super::transcript;

pub(super) const TRAVERSAL_STACK_ENTRY_BYTES: usize = std::mem::size_of::<ExpectedEdge>();

pub(super) fn authenticate_selected_closure(
    reader: &PhysicalRecordReader,
    inventory: &mut SelectedReleaseInventory,
    chain: &ValidatedReleaseChain,
    limits: BlobReclaimLimits,
    scratch: &mut [u8],
    work: &mut scan::ReclaimInspectionWork,
) -> Result<[u8; 32], BlobReclaimFailure> {
    let mut transcript_hash = Sha256::new();
    transcript_hash.update(transcript::CLOSURE_DOMAIN);
    let mut visited = 0_u64;
    let publication = inventory.basis.publication();
    let root = publication.root_record();
    if inventory.fact(root).is_none() {
        if chain.terminal && chain.settled_absence_authority {
            transcript_hash.update(0_u64.to_le_bytes());
            return Ok(transcript_hash.finalize().into());
        }
        return Err(BlobReclaimFailure::ConflictingSelectedFate);
    }
    let stack_capacity = usize::try_from(limits.maximum_selected_records())
        .ok()
        .and_then(|count| count.checked_mul(2))
        .ok_or(BlobReclaimFailure::ScratchUnavailable)?;
    let mut stack = Vec::new();
    stack
        .try_reserve_exact(stack_capacity)
        .map_err(|_| BlobReclaimFailure::ScratchUnavailable)?;
    if stack.capacity() != stack_capacity {
        return Err(BlobReclaimFailure::ScratchUnavailable);
    }
    stack.push(ExpectedEdge {
        parent: None,
        entry_index: 0,
        record: root,
        digest: publication.root_digest(),
        covered_bytes: publication.total_bytes(),
        start: 0,
        kind: ExpectedKind::RootTree,
    });
    while let Some(edge) = stack.pop() {
        let Some(fact) = inventory.fact(edge.record).copied() else {
            // The missing route is a fact of this exact owner-selected root,
            // not of a caller-supplied historical ID list. Only an attested
            // keyed head tied to this reader's root can settle the absence.
            if !matches!(edge.kind, ExpectedKind::RootTree) && chain.settled_absence_authority {
                continue;
            }
            return Err(BlobReclaimFailure::ConflictingSelectedFate);
        };
        let already_reached = fact.reachable;
        match (edge.kind, fact.blob) {
            (
                ExpectedKind::RootTree,
                SelectedBlobFact::Tree {
                    store,
                    session,
                    covered_bytes,
                    ..
                },
            ) if store == publication.store()
                && session == publication.session()
                && fact.frame_sha256 == edge.digest
                && covered_bytes == edge.covered_bytes => {}
            (
                ExpectedKind::Tree(expected_level),
                SelectedBlobFact::Tree {
                    store,
                    session,
                    level,
                    canonical_digest,
                    covered_bytes,
                    ..
                },
            ) if store == publication.store()
                && session == publication.session()
                && level == expected_level
                && canonical_digest == edge.digest
                && covered_bytes == edge.covered_bytes => {}
            (
                ExpectedKind::Chunk,
                SelectedBlobFact::Chunk {
                    store,
                    session,
                    ordinal,
                    content_digest,
                    covered_bytes,
                    chunk_size,
                    ..
                },
            ) if store == publication.store()
                && session == publication.session()
                && leaf_occurrence_matches(
                    publication.total_bytes(),
                    publication.chunk_size(),
                    edge.start,
                    edge.covered_bytes,
                    ordinal,
                )
                && content_digest == edge.digest
                && covered_bytes == edge.covered_bytes
                && chunk_size == publication.chunk_size() => {}
            (
                ExpectedKind::Chunk,
                SelectedBlobFact::ReuseClaim {
                    store,
                    session,
                    ordinal,
                    scope,
                    content_digest,
                    covered_bytes,
                    chunk_size,
                    ..
                },
            ) if store == publication.store()
                && session == publication.session()
                && leaf_occurrence_matches(
                    publication.total_bytes(),
                    publication.chunk_size(),
                    edge.start,
                    edge.covered_bytes,
                    ordinal,
                )
                && scope == publication.key_scope()
                && content_digest == edge.digest
                && covered_bytes == edge.covered_bytes
                && chunk_size == publication.chunk_size() => {}
            _ => return Err(BlobReclaimFailure::ConflictingSelectedFate),
        }
        transcript_hash.update([u8::from(edge.parent.is_some())]);
        if let Some(parent) = edge.parent {
            transcript::record_id(&mut transcript_hash, parent);
        } else {
            transcript_hash.update([0; 24]);
        }
        transcript_hash.update(edge.entry_index.to_le_bytes());
        transcript::record_id(&mut transcript_hash, edge.record);
        transcript_hash.update(edge.start.to_le_bytes());
        transcript_hash.update(edge.covered_bytes.to_le_bytes());
        transcript_hash.update(edge.digest);
        transcript_hash.update(fact.frame_sha256);
        visited = visited
            .checked_add(1)
            .ok_or(BlobReclaimFailure::ScanBoundExhausted)?;
        inventory
            .fact_mut(edge.record)
            .expect("selected fact exists")
            .reachable = true;
        if already_reached {
            continue;
        }
        let SelectedBlobFact::Tree { kind, level, .. } = fact.blob else {
            continue;
        };
        work.records = work
            .records
            .checked_add(1)
            .ok_or(BlobReclaimFailure::ScanBoundExhausted)?;
        let used = scan::read_selected(
            reader,
            fact.record,
            fact.payload_bytes,
            scratch,
            limits,
            work,
        )?;
        let bytes = &scratch[..used];
        let digest: [u8; 32] = Sha256::digest(bytes).into();
        if digest != fact.frame_sha256 {
            return Err(BlobReclaimFailure::ConflictingSelectedFate);
        }
        let BlobRecordV1::TreeNode(node) =
            decode_blob_record(bytes).map_err(BlobReclaimFailure::Format)?
        else {
            return Err(BlobReclaimFailure::ConflictingSelectedFate);
        };
        if node.occurrence().kind() != kind || node.occurrence().level() != level {
            return Err(BlobReclaimFailure::ConflictingSelectedFate);
        }
        let mut child_start = edge.start;
        for (entry_index, entry) in node.entries().iter().enumerate() {
            if stack.len() == stack_capacity {
                return Err(BlobReclaimFailure::ScanBoundExhausted);
            }
            let expected = match kind {
                BlobTreeNodeKind::Leaf => ExpectedKind::Chunk,
                BlobTreeNodeKind::Interior => ExpectedKind::Tree(level - 1),
            };
            stack.push(ExpectedEdge {
                parent: Some(edge.record),
                entry_index: u16::try_from(entry_index)
                    .map_err(|_| BlobReclaimFailure::ScanBoundExhausted)?,
                record: entry.record(),
                digest: entry.digest(),
                covered_bytes: entry.covered_bytes(),
                start: child_start,
                kind: expected,
            });
            child_start = child_start
                .checked_add(entry.covered_bytes())
                .ok_or(BlobReclaimFailure::ConflictingSelectedFate)?;
        }
        if child_start
            != edge
                .start
                .checked_add(edge.covered_bytes)
                .ok_or(BlobReclaimFailure::ConflictingSelectedFate)?
        {
            return Err(BlobReclaimFailure::ConflictingSelectedFate);
        }
    }
    transcript_hash.update(visited.to_le_bytes());
    Ok(transcript_hash.finalize().into())
}

pub(super) fn propagate_protected_subtrees(
    reader: &PhysicalRecordReader,
    inventory: &mut SelectedReleaseInventory,
    limits: BlobReclaimLimits,
    scratch: &mut [u8],
    work: &mut scan::ReclaimInspectionWork,
) -> Result<(), BlobReclaimFailure> {
    let capacity = usize::try_from(limits.maximum_selected_records())
        .map_err(|_| BlobReclaimFailure::ScratchUnavailable)?;
    let mut stack = Vec::new();
    stack
        .try_reserve_exact(capacity)
        .map_err(|_| BlobReclaimFailure::ScratchUnavailable)?;
    if stack.capacity() != capacity {
        return Err(BlobReclaimFailure::ScratchUnavailable);
    }
    for fact in &inventory.facts {
        if fact.protected && matches!(fact.blob, SelectedBlobFact::Tree { .. }) {
            stack.push(fact.record);
        }
    }
    let mut expanded = Vec::new();
    expanded
        .try_reserve_exact(capacity)
        .map_err(|_| BlobReclaimFailure::ScratchUnavailable)?;
    if expanded.capacity() != capacity {
        return Err(BlobReclaimFailure::ScratchUnavailable);
    }
    while let Some(record) = stack.pop() {
        if expanded.contains(&record) {
            continue;
        }
        expanded.push(record);
        let fact = inventory
            .fact(record)
            .copied()
            .ok_or(BlobReclaimFailure::ConflictingSelectedFate)?;
        if !matches!(fact.blob, SelectedBlobFact::Tree { .. }) {
            continue;
        }
        work.records = work
            .records
            .checked_add(1)
            .ok_or(BlobReclaimFailure::ScanBoundExhausted)?;
        let used = scan::read_selected(reader, record, fact.payload_bytes, scratch, limits, work)?;
        let BlobRecordV1::TreeNode(node) =
            decode_blob_record(&scratch[..used]).map_err(BlobReclaimFailure::Format)?
        else {
            return Err(BlobReclaimFailure::ConflictingSelectedFate);
        };
        for entry in node.entries() {
            if let Some(child) = inventory.fact_mut(entry.record()) {
                child.protected = true;
                if matches!(child.blob, SelectedBlobFact::Tree { .. }) {
                    if !stack.contains(&child.record) && !expanded.contains(&child.record) {
                        if stack.len() == capacity {
                            return Err(BlobReclaimFailure::ScanBoundExhausted);
                        }
                        stack.push(child.record);
                    }
                }
            }
        }
    }
    Ok(())
}
