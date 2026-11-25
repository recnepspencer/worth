//! Fixture-managed multi-leaf `BTreeNodeV1` codec, child/sibling link routing,
//! and bounded backup verifier oracle.
//!
//! Splits and merges are maintained by this fixture to exercise `BTreeNodeV1`
//! encoding/decoding, `first_child()` / `cell.child()` routing links,
//! `previous_sibling()` / `next_sibling()` leaf links, and
//! `verify_bounded_layout_index_artifact_from_reader` against a `BTreeMap`.
use std::collections::BTreeMap;
use std::io::Cursor;
use worth_store_layout_indexes::{
    verify_bounded_layout_index_artifact_from_reader, BoundedLayoutIndexDenial,
    BoundedLayoutIndexVerificationRequest, LayoutIndexBackupFormat,
};
use worth_store_physical_format::{
    extent_payload_digest, BTreeNodeCellV1, BTreeNodeV1, PersistedRecordIdentity,
};

#[derive(Debug, Clone)]
pub(super) enum IndexOp {
    InsertOrUpdate { key: u8, value: Vec<u8> },
    Delete { key: u8 },
    RangeScan { lo: u8, hi: u8 },
}

pub(super) fn hex_digest(digest: [u8; 32]) -> String {
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

fn record_id(ordinal: u64) -> PersistedRecordIdentity {
    PersistedRecordIdentity::new([0x42; 16], ordinal).unwrap()
}

pub(super) struct PersistedBTreeModel {
    max_leaf_cells: usize,
    first_leaf: Option<PersistedRecordIdentity>,
    leaf_pages: BTreeMap<PersistedRecordIdentity, Vec<u8>>,
    interior_page: Option<Vec<u8>>,
}

impl PersistedBTreeModel {
    pub(super) fn new(max_leaf_cells: usize) -> Self {
        Self {
            max_leaf_cells,
            first_leaf: None,
            leaf_pages: BTreeMap::new(),
            interior_page: None,
        }
    }

    fn verify_encoded_node(bytes: &[u8]) -> Result<(), BoundedLayoutIndexDenial> {
        let digest = extent_payload_digest(bytes);
        let identity = format!("index:sha256:{}", hex_digest(digest));
        let req = BoundedLayoutIndexVerificationRequest::new(
            LayoutIndexBackupFormat::BTreeNodeV1,
            &identity,
            bytes.len() as u64,
            digest,
            bytes.len(),
        );
        let mut cursor = Cursor::new(bytes);
        let obs =
            verify_bounded_layout_index_artifact_from_reader(&mut cursor, bytes.len() as u64, req)?;
        assert_eq!(obs.bytes_read(), bytes.len() as u64);
        assert_eq!(obs.peak_buffer_bytes(), bytes.len() as u64);
        Ok(())
    }

    fn decode_leaf(&self, id: PersistedRecordIdentity) -> BTreeNodeV1 {
        let encoded = self
            .leaf_pages
            .get(&id)
            .unwrap_or_else(|| panic!("persisted child/sibling link {id:?} missing from page map"));
        BTreeNodeV1::decode(encoded).expect("persisted leaf decodes")
    }

    fn decode_all_entries(&self) -> Vec<(Vec<u8>, Vec<u8>)> {
        let mut out = Vec::new();
        let mut current = self.first_leaf;
        let mut prev = None;
        let mut visited = 0_usize;
        while let Some(id) = current {
            let leaf = self.decode_leaf(id);
            assert_eq!(
                leaf.previous_sibling(),
                prev,
                "decoded previous_sibling link must match prior leaf"
            );
            for cell in leaf.cells() {
                out.push((cell.key().to_vec(), cell.leaf_value().unwrap().to_vec()));
            }
            visited += 1;
            prev = Some(id);
            current = leaf.next_sibling();
        }
        assert_eq!(
            visited,
            self.leaf_pages.len(),
            "next_sibling chain must traverse every persisted leaf exactly once"
        );
        out
    }

    fn rebuild_pages(&mut self, entries: Vec<(Vec<u8>, Vec<u8>)>) {
        self.first_leaf = None;
        self.leaf_pages.clear();
        self.interior_page = None;
        if entries.is_empty() {
            return;
        }
        let chunks: Vec<&[(Vec<u8>, Vec<u8>)]> = entries.chunks(self.max_leaf_cells).collect();
        let count = chunks.len();
        let mut separators = Vec::new();
        self.first_leaf = Some(record_id(1));
        for (i, chunk) in chunks.iter().enumerate() {
            let id = record_id((i + 1) as u64);
            let prev = (i > 0).then(|| record_id(i as u64));
            let next = (i + 1 < count).then(|| record_id((i + 2) as u64));
            let cells: Vec<_> = chunk
                .iter()
                .map(|(k, v)| BTreeNodeCellV1::leaf(k.clone(), v.clone()))
                .collect();
            let leaf = BTreeNodeV1::leaf(1, cells, prev, next).expect("valid leaf node");
            let encoded = leaf.encode(4096).expect("leaf encodes");
            Self::verify_encoded_node(&encoded).expect("bounded verifier accepts leaf");
            if i > 0 {
                separators.push(BTreeNodeCellV1::interior(chunk[0].0.clone(), id));
            }
            self.leaf_pages.insert(id, encoded);
        }
        if !separators.is_empty() {
            let interior = BTreeNodeV1::interior(1, 1, record_id(1), separators, None, None)
                .expect("interior");
            let encoded = interior.encode(4096).expect("interior encodes");
            Self::verify_encoded_node(&encoded).expect("bounded verifier accepts interior");
            self.interior_page = Some(encoded);
        }
    }

    fn route_leaf_id(&self, key: &[u8]) -> Option<PersistedRecordIdentity> {
        let first = self.first_leaf?;
        let Some(ref encoded_root) = self.interior_page else {
            return Some(first);
        };
        let root = BTreeNodeV1::decode(encoded_root).expect("interior decodes");
        let first_child = root
            .first_child()
            .expect("interior root retains first_child");
        assert_eq!(
            first_child, first,
            "decoded first_child must match first leaf"
        );
        let idx = root.cells().partition_point(|cell| cell.key() <= key);
        if idx == 0 {
            Some(first_child)
        } else {
            Some(
                root.cells()[idx - 1]
                    .child()
                    .expect("interior separator cell retains child identity"),
            )
        }
    }

    pub(super) fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        let leaf_id = self.route_leaf_id(key)?;
        let leaf = self.decode_leaf(leaf_id);
        leaf.cells()
            .binary_search_by(|cell| cell.key().cmp(key))
            .ok()
            .map(|pos| leaf.cells()[pos].leaf_value().unwrap().to_vec())
    }

    pub(super) fn range_scan(&self, lo: &[u8], hi: &[u8]) -> Vec<(Vec<u8>, Vec<u8>)> {
        let mut current = self.route_leaf_id(lo);
        let mut prev: Option<PersistedRecordIdentity> = None;
        let mut out = Vec::new();
        while let Some(id) = current {
            let leaf = self.decode_leaf(id);
            if let Some(expected_prev) = prev {
                assert_eq!(
                    leaf.previous_sibling(),
                    Some(expected_prev),
                    "range scan previous_sibling back-link must match prior leaf"
                );
            }
            for cell in leaf.cells() {
                if cell.key() < lo {
                    continue;
                }
                if cell.key() > hi {
                    return out;
                }
                out.push((cell.key().to_vec(), cell.leaf_value().unwrap().to_vec()));
            }
            prev = Some(id);
            current = leaf.next_sibling();
        }
        out
    }

    pub(super) fn apply(&mut self, op: &IndexOp) {
        let mut entries = self.decode_all_entries();
        match op {
            IndexOp::InsertOrUpdate { key, value } => {
                let k = vec![*key];
                match entries.binary_search_by(|(ek, _)| ek.cmp(&k)) {
                    Ok(pos) => entries[pos].1 = value.clone(),
                    Err(pos) => entries.insert(pos, (k, value.clone())),
                }
                self.rebuild_pages(entries);
            }
            IndexOp::Delete { key } => {
                let k = vec![*key];
                if let Ok(pos) = entries.binary_search_by(|(ek, _)| ek.cmp(&k)) {
                    entries.remove(pos);
                    self.rebuild_pages(entries);
                }
            }
            IndexOp::RangeScan { .. } => {}
        }
    }
}
