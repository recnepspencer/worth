//! A selected V2 reuse leaf still names an actual source chunk and exact
//! source-tree edge. Its embedded publication only substitutes for a legally
//! un-routed publication frame, never for selected payload or tree custody.

use worth_store::physical_runtime::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    decode_blob_record, BlobChunkReuseClaimV2, BlobGenerationPublicationV1, BlobRecordKind,
    BlobRecordV1, BlobTreeNodeKind, BlobTreeNodeV1, CurrentPhysicalRecordPlacement,
    PersistedRecordIdentity, PhysicalRecordFormatDeclaration, SelectedRecordContentClass,
    BLOB_CHUNK_FRAME_MAX_BYTES, BLOB_TREE_NODE_FRAME_MAX_BYTES,
};

use super::super::super::record;
use crate::orchestration::planning::completion::historical_publication::HistoricalFailure;
use crate::{
    integrity_ingress::RecoveryIntegrityIngressTrace,
    orchestration::planning::manifest_entry_budget::ManifestEntryBudget,
};

const MAX_SOURCE_TREE_DEPTH: usize = 7;
const INVALID: HistoricalFailure = HistoricalFailure::Invalid;

pub(super) fn destination_matches(
    value: BlobChunkReuseClaimV2,
    destination: BlobGenerationPublicationV1,
    claim_record: PersistedRecordIdentity,
    start: u64,
    covered: u64,
    digest: [u8; 32],
) -> bool {
    let claim = value.claim();
    let chunk_size = u64::from(destination.chunk_size());
    claim_record != claim.selected_chunk()
        && claim_record != claim.source_publication()
        && claim.source_publication() != claim.selected_chunk()
        && claim.store() == destination.store()
        && claim.destination_session() == destination.session()
        && claim.scope() == destination.key_scope()
        && claim.chunk_size() == destination.chunk_size()
        && start % chunk_size == 0
        && claim.destination_ordinal() == start / chunk_size
        && claim.stored_digest() == digest
        && u64::from(claim.chunk_length()) == covered
        && covered
            == destination
                .total_bytes()
                .saturating_sub(start)
                .min(chunk_size)
}

/// A source read that ran out of its limits says so; it found no damage.
#[allow(clippy::too_many_arguments)]
pub(super) fn verify_selected_source(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    format: PhysicalRecordFormatDeclaration,
    routes: &[CurrentPhysicalRecordPlacement],
    value: BlobChunkReuseClaimV2,
    claim_record: PersistedRecordIdentity,
    budget: &mut ManifestEntryBudget,
    trace: &mut RecoveryIntegrityIngressTrace,
    scratch: &mut u64,
) -> Result<(), HistoricalFailure> {
    let claim = value.claim();
    let source = value.source_publication();
    if claim_record == claim.selected_chunk()
        || claim_record == claim.source_publication()
        || claim.selected_chunk() == claim.source_publication()
    {
        return Err(INVALID);
    }
    // A selected source publication, when present, must be the exact V2
    // witness. Absence is permitted only because this is an admitted V2 claim.
    if super::routed(routes, claim.source_publication()).is_some() {
        let bytes = read_selected(
            discovery,
            format,
            routes,
            claim.source_publication(),
            BlobRecordKind::GenerationPublished,
            BLOB_CHUNK_FRAME_MAX_BYTES,
            budget,
            trace,
            scratch,
        )?;
        if bytes != source.encode() {
            return Err(INVALID);
        }
    }
    let mut offset = claim
        .source_ordinal()
        .checked_mul(u64::from(claim.chunk_size()))
        .ok_or(INVALID)?;
    let mut record = source.root_record();
    let mut expected_digest = source.root_digest();
    let mut expected_bytes = source.total_bytes();
    let mut expected_level = None;
    let mut selected_edge = false;
    for depth in 0..MAX_SOURCE_TREE_DEPTH {
        let bytes = read_selected(
            discovery,
            format,
            routes,
            record,
            BlobRecordKind::TreeNode,
            BLOB_TREE_NODE_FRAME_MAX_BYTES,
            budget,
            trace,
            scratch,
        )?;
        let node = BlobTreeNodeV1::decode(&bytes).map_err(|_| INVALID)?;
        let digest = if depth == 0 {
            node.frame_digest()
        } else {
            node.canonical_digest()
        };
        if digest != expected_digest
            || node.covered_bytes() != expected_bytes
            || node.occurrence().store() != source.store()
            || node.occurrence().session() != source.session()
            || expected_level.is_some_and(|level| node.occurrence().level() != level)
        {
            return Err(INVALID);
        }
        let mut selected = None;
        for edge in node.entries() {
            if offset < edge.covered_bytes() {
                selected = Some(*edge);
                break;
            }
            offset -= edge.covered_bytes();
        }
        let edge = selected.ok_or(INVALID)?;
        if node.occurrence().kind() == BlobTreeNodeKind::Leaf {
            selected_edge = offset == 0
                && edge.record() == claim.selected_chunk()
                && edge.digest() == claim.stored_digest()
                && edge.covered_bytes() == u64::from(claim.chunk_length());
            break;
        }
        expected_level = Some(node.occurrence().level().checked_sub(1).ok_or(INVALID)?);
        record = edge.record();
        expected_digest = edge.digest();
        expected_bytes = edge.covered_bytes();
    }
    if !selected_edge {
        return Err(INVALID);
    }
    let bytes = read_selected(
        discovery,
        format,
        routes,
        claim.selected_chunk(),
        BlobRecordKind::Chunk,
        BLOB_CHUNK_FRAME_MAX_BYTES,
        budget,
        trace,
        scratch,
    )?;
    let Ok(BlobRecordV1::Chunk(chunk)) = decode_blob_record(&bytes) else {
        return Err(INVALID);
    };
    if chunk.occurrence().store() != source.store()
        || chunk.occurrence().session() != source.session()
        || chunk.occurrence().ordinal() != claim.source_ordinal()
        || chunk.chunk_size() != claim.chunk_size()
        || chunk.stored_digest() != claim.stored_digest()
        || chunk.bytes().len() != claim.chunk_length() as usize
    {
        return Err(INVALID);
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn read_selected(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    format: PhysicalRecordFormatDeclaration,
    routes: &[CurrentPhysicalRecordPlacement],
    record_id: PersistedRecordIdentity,
    kind: BlobRecordKind,
    maximum_bytes: usize,
    budget: &mut ManifestEntryBudget,
    trace: &mut RecoveryIntegrityIngressTrace,
    scratch: &mut u64,
) -> Result<Vec<u8>, HistoricalFailure> {
    let route = super::routed(routes, record_id).ok_or(INVALID)?;
    if !matches!(route, CurrentPhysicalRecordPlacement::Extent(_))
        || route.content_class() != SelectedRecordContentClass::Blob(kind)
    {
        return Err(INVALID);
    }
    record::read(
        discovery,
        format,
        Some(route),
        record_id,
        maximum_bytes as u64,
        budget,
        trace,
        scratch,
    )
}

#[cfg(test)]
mod tests {
    use sha2::{Digest, Sha256};
    use worth_store_physical_format::{
        BlobChunkReuseClaimV1, BlobChunkReuseClaimV2, BlobGenerationPublicationV1,
        PersistedRecordIdentity,
    };

    use super::destination_matches;

    fn record(ordinal: u64) -> PersistedRecordIdentity {
        PersistedRecordIdentity::new([9; 16], ordinal).unwrap()
    }

    #[test]
    fn v2_leaf_requires_destination_occurrence_scope_and_tree_edge_digest() {
        let destination = BlobGenerationPublicationV1::new(
            [1; 16],
            [2; 16],
            [3; 16],
            1,
            record(1),
            [4; 32],
            17,
            [5; 32],
            64 * 1024,
            [6; 32],
        )
        .unwrap();
        let source = BlobGenerationPublicationV1::new(
            [1; 16],
            [7; 16],
            [8; 16],
            1,
            record(2),
            [10; 32],
            17,
            [11; 32],
            64 * 1024,
            [6; 32],
        )
        .unwrap();
        let claim = BlobChunkReuseClaimV1::new(
            [1; 16],
            destination.session(),
            0,
            destination.key_scope(),
            destination.chunk_size(),
            17,
            [12; 32],
            record(3),
            record(4),
            0,
        )
        .unwrap();
        let source_sha: [u8; 32] = Sha256::digest(source.encode()).into();
        let value = BlobChunkReuseClaimV2::new(claim, source, source_sha).unwrap();
        assert!(destination_matches(
            value,
            destination,
            record(5),
            0,
            17,
            [12; 32]
        ));
        assert!(!destination_matches(
            value,
            destination,
            record(3),
            0,
            17,
            [12; 32]
        ));
        assert!(!destination_matches(
            value,
            destination,
            record(5),
            0,
            17,
            [13; 32]
        ));
        assert!(!destination_matches(
            value,
            destination,
            record(5),
            0,
            16,
            [12; 32]
        ));
        let other_session = BlobGenerationPublicationV1::new(
            [1; 16],
            [14; 16],
            [3; 16],
            1,
            record(1),
            [4; 32],
            17,
            [5; 32],
            64 * 1024,
            [6; 32],
        )
        .unwrap();
        assert!(!destination_matches(
            value,
            other_session,
            record(5),
            0,
            17,
            [12; 32]
        ));
    }
}
