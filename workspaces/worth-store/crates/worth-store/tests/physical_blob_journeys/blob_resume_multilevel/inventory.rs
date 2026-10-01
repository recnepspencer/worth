use worth_store::physical_runtime::{
    PhysicalRecordId, RecordByteLimit, RecordCountLimit, RecordReadLimits, RecordScanOutcome,
    RecordScanRequest, ServingPhysicalRuntime,
};
use worth_store_physical_format::{
    decode_blob_record, BlobGenerationPublicationV1, BlobRecordV1, BlobTreeNodeKind,
    BlobTreeNodeV1, PersistedRecordIdentity,
};

use super::child::{CHUNKS, CHUNK_BYTES, TOTAL_BYTES};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ChunkClaim {
    pub ordinal: u64,
    pub record: PhysicalRecordId,
    pub digest: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct NodeClaim {
    pub record: PhysicalRecordId,
    pub node: BlobTreeNodeV1,
}

#[derive(Debug)]
pub(super) struct SelectedInventory {
    pub chunks: Vec<ChunkClaim>,
    pub nodes: Vec<NodeClaim>,
    pub declaration_count: usize,
    pub frontier_ordinals: Vec<u64>,
    pub publications: Vec<BlobGenerationPublicationV1>,
}

/// Keeps only selected identities and three tree nodes. Each deferred C5
/// payload is read and decoded before the next selected row is visited. The
/// parent world is quiescent between C8 completion and resumed mutation.
pub(super) fn selected(serving: &ServingPhysicalRuntime, session: [u8; 16]) -> SelectedInventory {
    let mut inventory = SelectedInventory {
        chunks: Vec::with_capacity(CHUNKS),
        nodes: Vec::with_capacity(3),
        declaration_count: 0,
        frontier_ordinals: Vec::with_capacity(64),
        publications: Vec::with_capacity(1),
    };
    let mut scan = serving
        .records()
        .unwrap()
        .scan(
            RecordScanRequest::from_start()
                .with_batch_limit(RecordCountLimit::new(1).unwrap())
                .with_payload_limit(RecordByteLimit::new(236).unwrap()),
        )
        .unwrap();
    let mut scan_scratch = [0_u8; 4096];
    let mut frame = Vec::with_capacity(BlobTreeNodeV1::maximum_encoded_bytes());
    let mut visited = 0_usize;
    while let RecordScanOutcome::Batch(batch) = scan.read_next_into(&mut scan_scratch).unwrap() {
        for row in batch.records() {
            visited += 1;
            assert!(
                visited <= 8192,
                "selected inventory exceeded admitted scan bound"
            );
            let record = row.record_id();
            let length = row.declared_payload_bytes();
            assert!(
                length as usize <= BlobTreeNodeV1::maximum_encoded_bytes(),
                "selected frame exceeds bounded blob-node read"
            );
            frame.clear();
            frame.resize(length as usize, 0);
            let reader = serving.records().unwrap();
            let mut stream = reader
                .open(
                    record,
                    RecordReadLimits::new(
                        RecordByteLimit::new(u32::try_from(length).unwrap()).unwrap(),
                    ),
                )
                .unwrap();
            let mut used = 0;
            while used < frame.len() {
                let amount = stream.read_next(&mut frame[used..]).unwrap();
                assert!(amount > 0, "selected record ended before declared length");
                used += amount;
            }
            if !frame.starts_with(b"WRC11BLB") {
                continue; // Recovery bootstrap may have one ordinary C5 record.
            }
            match decode_blob_record(&frame).unwrap() {
                BlobRecordV1::SessionDeclared(value) if value.session() == session => {
                    inventory.declaration_count += 1;
                }
                BlobRecordV1::Chunk(value) if value.occurrence().session() == session => {
                    assert!(
                        inventory.chunks.len() < CHUNKS,
                        "extra selected chunk claim"
                    );
                    inventory.chunks.push(ChunkClaim {
                        ordinal: value.occurrence().ordinal(),
                        record,
                        digest: value.stored_digest(),
                    });
                }
                BlobRecordV1::TreeNode(value) if value.occurrence().session() == session => {
                    assert!(inventory.nodes.len() < 3, "extra selected tree node");
                    inventory.nodes.push(NodeClaim {
                        record,
                        node: value,
                    });
                }
                BlobRecordV1::SessionFrontier(value) if value.session() == session => {
                    assert!(inventory.frontier_ordinals.len() < 64, "extra frontier");
                    inventory.frontier_ordinals.push(value.next_chunk_ordinal());
                    assert_eq!(
                        value.durable_bytes(),
                        value.next_chunk_ordinal() * CHUNK_BYTES as u64
                    );
                }
                BlobRecordV1::GenerationPublished(value) if value.session() == session => {
                    assert!(inventory.publications.is_empty(), "duplicate generation");
                    inventory.publications.push(value);
                }
                _ => {}
            }
        }
        if batch.is_complete() {
            break;
        }
    }
    inventory.chunks.sort_unstable_by_key(|claim| claim.ordinal);
    inventory.nodes.sort_unstable_by_key(|claim| {
        (
            claim.node.occurrence().level(),
            claim.node.occurrence().index(),
        )
    });
    inventory.frontier_ordinals.sort_unstable();
    inventory
}

pub(super) fn assert_selected_tree(inventory: &SelectedInventory, session: [u8; 16]) {
    assert_eq!(inventory.declaration_count, 1);
    assert_eq!(inventory.chunks.len(), CHUNKS);
    for (ordinal, claim) in inventory.chunks.iter().enumerate() {
        assert_eq!(claim.ordinal, ordinal as u64);
    }
    assert_eq!(inventory.frontier_ordinals.len(), 64);
    for (index, ordinal) in inventory.frontier_ordinals.iter().enumerate() {
        assert_eq!(*ordinal, ((index + 1) * 64) as u64);
    }
    assert_eq!(inventory.nodes.len(), 3);
    let [full, partial, root] = inventory.nodes.as_slice() else {
        panic!("expected two selected leaves and one selected interior root")
    };
    for (node, index, kind, entries, covered) in [
        (full, 0, BlobTreeNodeKind::Leaf, 4096, 4096 * CHUNK_BYTES),
        (partial, 1, BlobTreeNodeKind::Leaf, 1, CHUNK_BYTES),
        (root, 0, BlobTreeNodeKind::Interior, 2, TOTAL_BYTES),
    ] {
        assert_eq!(node.node.occurrence().session(), session);
        assert_eq!(node.node.occurrence().kind(), kind);
        assert_eq!(
            node.node.occurrence().level(),
            if kind == BlobTreeNodeKind::Leaf { 0 } else { 1 }
        );
        assert_eq!(node.node.occurrence().index(), index);
        assert_eq!(node.node.entries().len(), entries);
        assert_eq!(node.node.covered_bytes(), covered as u64);
    }
    for (index, entry) in full.node.entries().iter().enumerate() {
        assert_record_binding(inventory.chunks[index].record, entry.record());
        assert_eq!(entry.digest(), inventory.chunks[index].digest);
        assert_eq!(entry.covered_bytes(), CHUNK_BYTES as u64);
    }
    assert_record_binding(
        inventory.chunks[4096].record,
        partial.node.entries()[0].record(),
    );
    assert_eq!(
        partial.node.entries()[0].digest(),
        inventory.chunks[4096].digest
    );
    assert_eq!(
        partial.node.entries()[0].covered_bytes(),
        CHUNK_BYTES as u64
    );
    for (node, edge) in [
        (full, &root.node.entries()[0]),
        (partial, &root.node.entries()[1]),
    ] {
        assert_record_binding(node.record, edge.record());
        assert_eq!(node.node.canonical_digest(), edge.digest());
        assert_eq!(node.node.covered_bytes(), edge.covered_bytes());
    }
}

pub(super) fn assert_record_binding(record: PhysicalRecordId, selected: PersistedRecordIdentity) {
    assert_eq!(record.allocation_epoch(), selected.allocation_epoch());
    assert_eq!(record.ordinal(), selected.ordinal());
}
