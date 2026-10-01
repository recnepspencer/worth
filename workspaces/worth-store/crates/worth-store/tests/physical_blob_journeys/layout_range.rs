use std::num::NonZeroU64;

use worth_store::physical_runtime::{
    BlobCheckpointLimit, BlobIngestDeclaration, BlobReadLimits, PhysicalIndexPointKey,
    PhysicalIndexPrefix, PhysicalIndexRange, PhysicalIndexScanBudget,
    PhysicalIndexScanRequestDenial, PhysicalMutationDeadline,
};
use worth_store_blob_chunks::BlobChunkSize;
use worth_store_contracts::DurableArtifactFamilyId;

use super::fixture::{admitted_blob_scope, placement, serving_from_initialization};

#[test]
fn catalog_range_and_object_prefix_follow_selected_leaf_without_scan() {
    const CHUNK_BYTES: usize = 64 * 1024;
    let root = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(root.path());
    let scope = admitted_blob_scope("c11.layout.catalog.range");
    let limits = BlobReadLimits::new(NonZeroU64::new(128).unwrap());
    let blobs = serving.blobs().unwrap();
    let mut keys = Vec::new();
    for byte in [0x17, 0x38, 0x59] {
        let object = blobs.issue_object_id(limits).unwrap();
        let declaration = BlobIngestDeclaration::new(
            object,
            BlobChunkSize::from_bytes(CHUNK_BYTES as u64).unwrap(),
            (2 * CHUNK_BYTES) as u64,
            &scope,
            BlobCheckpointLimit::bounded_horizon(16).unwrap(),
            PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
        )
        .unwrap();
        let mut ingest = blobs
            .begin_ingest(declaration, placement(), CHUNK_BYTES as u64, limits)
            .unwrap();
        ingest.push(&vec![byte; CHUNK_BYTES]).unwrap();
        ingest
            .push(&vec![byte.wrapping_add(1); CHUNK_BYTES])
            .unwrap();
        let published = ingest.finish().unwrap();
        let key =
            PhysicalIndexPointKey::blob_catalog(object, published.generation().sequence()).unwrap();
        keys.push((key, object));
    }
    keys.sort_by_key(|(key, _)| key.canonical_bytes());
    drop(blobs);
    let layouts = serving.layouts().unwrap();
    let index = layouts.btree(DurableArtifactFamilyId::BlobCatalog).unwrap();
    let budget = PhysicalIndexScanBudget::pages(1).unwrap();
    let mut range = index
        .range(
            PhysicalIndexRange::between(keys[0].0, keys[2].0).unwrap(),
            budget,
        )
        .unwrap();
    let entries: Vec<_> = range.by_ref().map(Result::unwrap).collect();
    assert_eq!(
        entries
            .iter()
            .map(|entry| entry.canonical_key())
            .collect::<Vec<_>>(),
        keys[..2]
            .iter()
            .map(|(key, _)| key.canonical_bytes())
            .collect::<Vec<_>>()
    );
    assert_eq!(range.counters().page_touches(), 1);
    assert_eq!(range.counters().node_decodes(), 1);
    for (key, object) in keys {
        let mut prefix = index
            .prefix(PhysicalIndexPrefix::blob_object(object), budget)
            .unwrap();
        let hits: Vec<_> = prefix.by_ref().map(Result::unwrap).collect();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].canonical_key(), key.canonical_bytes());
        assert_eq!(prefix.counters().page_touches(), 1);
    }
    assert_eq!(
        PhysicalIndexScanBudget::pages(0),
        Err(PhysicalIndexScanRequestDenial::ZeroPageBudget)
    );
    assert_eq!(
        PhysicalIndexScanBudget::pages(65),
        Err(PhysicalIndexScanRequestDenial::PageBudgetTooWide)
    );
    drop(range);
    drop(index);
    drop(layouts);
    serving.close();
}
