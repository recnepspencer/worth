pub(crate) use {worth_store::physical_runtime::PhysicalRecordId, worth_store::physical_runtime::RecordByteLimit, worth_store::physical_runtime::RecordCountLimit, worth_store::physical_runtime::RecordReadLimits, worth_store::physical_runtime::RecordScanOutcome, worth_store::physical_runtime::RecordScanRequest, worth_store::physical_runtime::ServingPhysicalRuntime};
pub(crate) use {worth_store_physical_format::decode_blob_record, worth_store_physical_format::BlobGenerationPublicationV1, worth_store_physical_format::BlobRecordV1, worth_store_physical_format::BlobTreeNodeKind, worth_store_physical_format::BlobTreeNodeV1, worth_store_physical_format::PersistedRecordIdentity};

pub(crate) use {};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ChunkClaim {
    pub(crate) ordinal: u64,
    pub(crate) record: PhysicalRecordId,
    pub(crate) digest: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NodeClaim {
    pub(crate) record: PhysicalRecordId,
    pub(crate) node: BlobTreeNodeV1,
}

#[derive(Debug)]
pub(crate) struct SelectedInventory {
    pub(crate) chunks: Vec<ChunkClaim>,
    pub(crate) nodes: Vec<NodeClaim>,
    pub(crate) declaration_count: usize,
    pub(crate) frontier_ordinals: Vec<u64>,
    pub(crate) publications: Vec<BlobGenerationPublicationV1>,
}

/// Keeps only selected identities and three tree nodes. Each deferred C5
/// payload is read and decoded before the next selected row is visited. The
/// parent world is quiescent between C8 completion and resumed mutation.
pub(crate) fn selected(serving: &ServingPhysicalRuntime, session: [u8; 16]) -> SelectedInventory {
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
            worth_store::physical_runtime::RecordScanRequest::from_start()
                .with_batch_limit(worth_store::physical_runtime::RecordCountLimit::new(1).unwrap())
                .with_payload_limit(worth_store::physical_runtime::RecordByteLimit::new(236).unwrap()),
        )
        .unwrap();
    let mut scan_scratch = [0_u8; 4096];
    let mut frame = Vec::with_capacity(worth_store_physical_format::BlobTreeNodeV1::maximum_encoded_bytes());
    let mut visited = 0_usize;
    while let worth_store::physical_runtime::RecordScanOutcome::Batch(batch) = scan.read_next_into(&mut scan_scratch).unwrap() {
        for row in batch.records() {
            visited += 1;
            assert!(
                visited <= 8192,
                "selected inventory exceeded admitted scan bound"
            );
            let record = row.record_id();
            let length = row.declared_payload_bytes();
            assert!(
                length as usize <= worth_store_physical_format::BlobTreeNodeV1::maximum_encoded_bytes(),
                "selected frame exceeds bounded blob-node read"
            );
            frame.clear();
            frame.resize(length as usize, 0);
            let reader = serving.records().unwrap();
            let mut stream = reader
                .open(
                    record,
                    worth_store::physical_runtime::RecordReadLimits::new(
                        worth_store::physical_runtime::RecordByteLimit::new(u32::try_from(length).unwrap()).unwrap(),
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
                worth_store_physical_format::BlobRecordV1::SessionDeclared(value) if value.session() == session => {
                    inventory.declaration_count += 1;
                }
                worth_store_physical_format::BlobRecordV1::Chunk(value) if value.occurrence().session() == session => {
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
                worth_store_physical_format::BlobRecordV1::TreeNode(value) if value.occurrence().session() == session => {
                    assert!(inventory.nodes.len() < 3, "extra selected tree node");
                    inventory.nodes.push(NodeClaim {
                        record,
                        node: value,
                    });
                }
                worth_store_physical_format::BlobRecordV1::SessionFrontier(value) if value.session() == session => {
                    assert!(inventory.frontier_ordinals.len() < 64, "extra frontier");
                    inventory.frontier_ordinals.push(value.next_chunk_ordinal());
                    assert_eq!(
                        value.durable_bytes(),
                        value.next_chunk_ordinal() * CHUNK_BYTES as u64
                    );
                }
                worth_store_physical_format::BlobRecordV1::GenerationPublished(value) if value.session() == session => {
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

pub(crate) fn assert_selected_tree(inventory: &SelectedInventory, session: [u8; 16]) {
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
        (full, 0, worth_store_physical_format::BlobTreeNodeKind::Leaf, 4096, 4096 * CHUNK_BYTES),
        (partial, 1, worth_store_physical_format::BlobTreeNodeKind::Leaf, 1, CHUNK_BYTES),
        (root, 0, worth_store_physical_format::BlobTreeNodeKind::Interior, 2, TOTAL_BYTES),
    ] {
        assert_eq!(node.node.occurrence().session(), session);
        assert_eq!(node.node.occurrence().kind(), kind);
        assert_eq!(
            node.node.occurrence().level(),
            if kind == worth_store_physical_format::BlobTreeNodeKind::Leaf { 0 } else { 1 }
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

pub(crate) fn assert_record_binding(record: PhysicalRecordId, selected: PersistedRecordIdentity) {
    assert_eq!(record.allocation_epoch(), selected.allocation_epoch());
    assert_eq!(record.ordinal(), selected.ordinal());
}