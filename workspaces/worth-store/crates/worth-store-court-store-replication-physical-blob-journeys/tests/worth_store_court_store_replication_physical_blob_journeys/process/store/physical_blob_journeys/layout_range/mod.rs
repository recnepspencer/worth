use {
    worth_store_testkit_store::fixture::admitted_blob_scope,
    worth_store_testkit_store::fixture::placement,
    worth_store_testkit_store::fixture::serving_from_initialization,
};

#[test]
fn catalog_range_and_object_prefix_follow_selected_leaf_without_scan() {
    const CHUNK_BYTES: usize = 64 * 1024;
    let root = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(root.path());
    let scope = admitted_blob_scope("c11.layout.catalog.range");
    let limits =
        worth_store::physical_runtime::BlobReadLimits::new(std::num::NonZeroU64::new(128).unwrap());
    let blobs = serving.blobs().unwrap();
    let mut keys = Vec::new();
    for byte in [0x17, 0x38, 0x59] {
        let object = blobs.issue_object_id(limits).unwrap();
        let declaration = worth_store::physical_runtime::BlobIngestDeclaration::new(
            object,
            worth_store_blob_chunks::BlobChunkSize::from_bytes(CHUNK_BYTES as u64).unwrap(),
            (2 * CHUNK_BYTES) as u64,
            &scope,
            worth_store::physical_runtime::BlobCheckpointLimit::bounded_horizon(16).unwrap(),
            worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000)
                .unwrap(),
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
        let key = worth_store::physical_runtime::PhysicalIndexPointKey::blob_catalog(
            object,
            published.generation().sequence(),
        )
        .unwrap();
        keys.push((key, object));
    }
    keys.sort_by_key(|(key, _)| key.canonical_bytes());
    let layouts = serving.layouts().unwrap();
    let index = layouts
        .btree(worth_store_contracts::DurableArtifactFamilyId::BlobCatalog)
        .unwrap();
    let budget = worth_store::physical_runtime::PhysicalIndexScanBudget::pages(1).unwrap();
    let mut range = index
        .range(
            worth_store::physical_runtime::PhysicalIndexRange::between(keys[0].0, keys[2].0)
                .unwrap(),
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
            .prefix(
                worth_store::physical_runtime::PhysicalIndexPrefix::blob_object(object),
                budget,
            )
            .unwrap();
        let hits: Vec<_> = prefix.by_ref().map(Result::unwrap).collect();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].canonical_key(), key.canonical_bytes());
        assert_eq!(prefix.counters().page_touches(), 1);
    }
    assert_eq!(
        worth_store::physical_runtime::PhysicalIndexScanBudget::pages(0),
        Err(worth_store::physical_runtime::PhysicalIndexScanRequestDenial::ZeroPageBudget)
    );
    assert_eq!(
        worth_store::physical_runtime::PhysicalIndexScanBudget::pages(65),
        Err(worth_store::physical_runtime::PhysicalIndexScanRequestDenial::PageBudgetTooWide)
    );
    drop(range);
    drop(layouts);
    serving.close();
}
