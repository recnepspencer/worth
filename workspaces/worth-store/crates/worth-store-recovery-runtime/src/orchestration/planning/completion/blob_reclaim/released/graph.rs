//! Independently authenticate the entire pre-drop publication closure.
//! A publication frame or its digest alone is not graph custody evidence.

use std::collections::BTreeSet;

use sha2::{Digest, Sha256};
use worth_store::physical_runtime::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    decode_blob_record, BlobRecordKind, BlobRecordV1, BlobTreeNodeKind,
    CurrentPhysicalRecordPlacement, PersistedRecordIdentity, PhysicalRecordFormatDeclaration,
    ReleasedGenerationReclaimBasisV1, SelectedRecordContentClass, BLOB_CHUNK_FRAME_MAX_BYTES,
};

use super::super::record;
use super::closure_evidence::ReleasedClosureEvidence;
use crate::orchestration::planning::completion::historical_publication::HistoricalFailure;
use crate::{
    integrity_ingress::RecoveryIntegrityIngressTrace,
    orchestration::planning::manifest_entry_budget::ManifestEntryBudget,
};

#[cfg(test)]
#[path = "graph/child_tests.rs"]
mod child_tests;
#[path = "graph/reuse_source.rs"]
mod reuse_source;

const INVALID: HistoricalFailure = HistoricalFailure::Invalid;

#[derive(Clone, Copy)]
enum ExpectedKind {
    Root,
    Tree(u8),
    Chunk,
}

#[derive(Clone, Copy)]
struct ExpectedEdge {
    parent: Option<PersistedRecordIdentity>,
    entry_index: u16,
    record: PersistedRecordIdentity,
    digest: [u8; 32],
    start: u64,
    covered: u64,
    kind: ExpectedKind,
}

pub(super) fn authenticate(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    format: PhysicalRecordFormatDeclaration,
    routes: &[CurrentPhysicalRecordPlacement],
    source: ReleasedGenerationReclaimBasisV1,
    closure_evidence: ReleasedClosureEvidence<'_>,
    terminal: bool,
    budget: &mut ManifestEntryBudget,
    trace: &mut RecoveryIntegrityIngressTrace,
    scratch: &mut u64,
    reached: &mut BTreeSet<PersistedRecordIdentity>,
) -> Result<[u8; 32], HistoricalFailure> {
    let publication = source.publication();
    let mut transcript = Sha256::new();
    transcript.update(b"store.physical.released-drop-closure.v1");
    let mut edge_count = 0_u64;
    let mut pending = vec![ExpectedEdge {
        parent: None,
        entry_index: 0,
        record: publication.root_record(),
        digest: publication.root_digest(),
        start: 0,
        covered: publication.total_bytes(),
        kind: ExpectedKind::Root,
    }];
    if terminal
        && routed(routes, publication.root_record()).is_none()
        && closure_evidence.settles_absent_root(publication.root_record())
    {
        transcript.update(0_u64.to_le_bytes());
        return Ok(transcript.finalize().into());
    }
    while let Some(edge) = pending.pop() {
        let Some(route) = routed(routes, edge.record) else {
            // Only the graph root is queued without a route of its own.
            return Err(INVALID);
        };
        edge_count = edge_count.checked_add(1).ok_or(INVALID)?;
        let admitted_class = match edge.kind {
            ExpectedKind::Root | ExpectedKind::Tree(_) => {
                route.content_class() == SelectedRecordContentClass::Blob(BlobRecordKind::TreeNode)
            }
            ExpectedKind::Chunk => matches!(
                route.content_class(),
                SelectedRecordContentClass::Blob(
                    BlobRecordKind::Chunk | BlobRecordKind::ChunkReuseClaimV2
                )
            ),
        };
        if !matches!(route, CurrentPhysicalRecordPlacement::Extent(_)) || !admitted_class {
            return Err(INVALID);
        }
        // A read that ran out of its limits says so; it found no damage.
        let bytes = record::read(
            discovery,
            format,
            Some(route),
            edge.record,
            BLOB_CHUNK_FRAME_MAX_BYTES as u64,
            budget,
            trace,
            scratch,
        )?;
        let Ok(fact) = decode_blob_record(&bytes) else {
            return Err(INVALID);
        };
        transcript.update([u8::from(edge.parent.is_some())]);
        update_record(&mut transcript, edge.parent);
        transcript.update(edge.entry_index.to_le_bytes());
        update_record(&mut transcript, Some(edge.record));
        transcript.update(edge.start.to_le_bytes());
        transcript.update(edge.covered.to_le_bytes());
        transcript.update(edge.digest);
        transcript.update(Sha256::digest(&bytes));
        match (edge.kind, fact) {
            (ExpectedKind::Root, BlobRecordV1::TreeNode(node)) => {
                if node.occurrence().store() != publication.store()
                    || node.occurrence().session() != publication.session()
                    || node.covered_bytes() != edge.covered
                    || <[u8; 32]>::from(Sha256::digest(&bytes)) != edge.digest
                {
                    return Err(INVALID);
                }
                if reached.insert(edge.record)
                    && !push_children(
                        &mut pending,
                        &node,
                        edge.record,
                        edge.start,
                        routes,
                        closure_evidence,
                    )
                {
                    return Err(INVALID);
                }
            }
            (ExpectedKind::Tree(level), BlobRecordV1::TreeNode(node)) => {
                if node.occurrence().store() != publication.store()
                    || node.occurrence().session() != publication.session()
                    || node.occurrence().level() != level
                    || node.covered_bytes() != edge.covered
                    || node.canonical_digest() != edge.digest
                {
                    return Err(INVALID);
                }
                if reached.insert(edge.record)
                    && !push_children(
                        &mut pending,
                        &node,
                        edge.record,
                        edge.start,
                        routes,
                        closure_evidence,
                    )
                {
                    return Err(INVALID);
                }
            }
            (ExpectedKind::Chunk, BlobRecordV1::Chunk(chunk)) => {
                if chunk.occurrence().store() != publication.store()
                    || chunk.occurrence().session() != publication.session()
                    || chunk.stored_digest() != edge.digest
                    || chunk.bytes().len() as u64 != edge.covered
                    || chunk.chunk_size() != publication.chunk_size()
                    || edge.start % u64::from(publication.chunk_size()) != 0
                    || chunk.occurrence().ordinal()
                        != edge.start / u64::from(publication.chunk_size())
                    || edge.covered
                        != publication
                            .total_bytes()
                            .saturating_sub(edge.start)
                            .min(u64::from(publication.chunk_size()))
                {
                    return Err(INVALID);
                }
                reached.insert(edge.record);
            }
            (ExpectedKind::Chunk, BlobRecordV1::ChunkReuseClaimV2(claim)) => {
                if !reuse_source::destination_matches(
                    claim,
                    publication,
                    edge.record,
                    edge.start,
                    edge.covered,
                    edge.digest,
                ) {
                    return Err(INVALID);
                }
                reuse_source::verify_selected_source(
                    discovery,
                    format,
                    routes,
                    claim,
                    edge.record,
                    budget,
                    trace,
                    scratch,
                )?;
                reached.insert(edge.record);
            }
            _ => return Err(INVALID),
        }
        if reached.len() > routes.len() {
            return Err(INVALID);
        }
    }
    transcript.update(edge_count.to_le_bytes());
    Ok(transcript.finalize().into())
}

fn update_record(transcript: &mut Sha256, record: Option<PersistedRecordIdentity>) {
    let (epoch, ordinal) = record
        .map(|record| (record.allocation_epoch(), record.ordinal()))
        .unwrap_or(([0; 16], 0));
    transcript.update(epoch);
    transcript.update(ordinal.to_le_bytes());
}

/// The route the source root holds for `record`. The selected inventory
/// admits `routes` only in ascending record order.
pub(super) fn routed(
    routes: &[CurrentPhysicalRecordPlacement],
    record: PersistedRecordIdentity,
) -> Option<CurrentPhysicalRecordPlacement> {
    routes
        .binary_search_by_key(&record, |route| route.record())
        .ok()
        .map(|index| routes[index])
}

/// Queues the children of `node` that the source root still routes. A child
/// it no longer routes must be settled by the closure evidence: an earlier
/// batch of this release dropped it. A tree queues each routed record once,
/// so a queue as long as `routes` already holds a record named more than
/// once: the closure is denied there, and the queue never outgrows `routes`.
fn push_children(
    pending: &mut Vec<ExpectedEdge>,
    node: &worth_store_physical_format::BlobTreeNodeV1,
    parent: PersistedRecordIdentity,
    start: u64,
    routes: &[CurrentPhysicalRecordPlacement],
    closure_evidence: ReleasedClosureEvidence<'_>,
) -> bool {
    let mut child_start = start;
    for (index, entry) in node.entries().iter().enumerate() {
        let kind = match node.occurrence().kind() {
            BlobTreeNodeKind::Leaf => ExpectedKind::Chunk,
            BlobTreeNodeKind::Interior => {
                let Some(level) = node.occurrence().level().checked_sub(1) else {
                    return false;
                };
                ExpectedKind::Tree(level)
            }
        };
        if routed(routes, entry.record()).is_some() {
            if pending.len() >= routes.len() {
                return false;
            }
            pending.push(ExpectedEdge {
                parent: Some(parent),
                entry_index: index as u16,
                record: entry.record(),
                digest: entry.digest(),
                start: child_start,
                covered: entry.covered_bytes(),
                kind,
            });
        } else if !closure_evidence.settles_absent_child(entry.record()) {
            return false;
        }
        let Some(next_start) = child_start.checked_add(entry.covered_bytes()) else {
            return false;
        };
        child_start = next_start;
    }
    start
        .checked_add(node.covered_bytes())
        .is_some_and(|end| child_start == end)
}
