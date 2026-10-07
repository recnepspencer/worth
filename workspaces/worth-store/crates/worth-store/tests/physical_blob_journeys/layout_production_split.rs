use std::num::NonZeroU64;

use worth_store::physical_runtime::{
    BlobCheckpointLimit, BlobIngestDeclaration, BlobReadLimits, PageFillPercent,
    PhysicalIndexPointKey, PhysicalIndexPrefix, PhysicalIndexRange, PhysicalIndexScanBudget,
    PhysicalLayoutDenial, PhysicalMutationDeadline, PhysicalRecordPlacementPolicy,
};
use worth_store_blob_chunks::BlobChunkSize;
use worth_store_contracts::DurableArtifactFamilyId;

use super::blob_ingest_process::observe_closed_store_named;
use super::fixture::{
    admitted_blob_scope, configuration, serving_from_initialization_with_placement,
    serving_from_open,
};

#[test]
fn production_catalog_split_answers_point_range_prefix_from_selected_pages() {
    let root = tempfile::tempdir().unwrap();
    let (format, _, _) = configuration();
    // An admitted low fill exercises real COW splits without 260 heavyweight
    // blob publications. It is not a fixed-topology certification tree.
    let placement = PhysicalRecordPlacementPolicy::builder()
        .page_fill(PageFillPercent::new(10).unwrap())
        .admit(format)
        .unwrap();
    let serving = serving_from_initialization_with_placement(root.path(), placement);
    let scope = admitted_blob_scope("c11.layout.production-split");
    let limits = BlobReadLimits::new(NonZeroU64::new(4096).unwrap());
    let mut model = Vec::new();
    for ordinal in 0..30_u8 {
        let blobs = serving.blobs().unwrap();
        let object = blobs.issue_object_id(limits).unwrap();
        let declaration = BlobIngestDeclaration::new(
            object,
            BlobChunkSize::from_bytes(64 * 1024).unwrap(),
            2,
            &scope,
            BlobCheckpointLimit::bounded_horizon(16).unwrap(),
            PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
        )
        .unwrap();
        let mut ingest = blobs
            .begin_ingest(declaration, placement, 1, limits)
            .unwrap();
        ingest.push(&[ordinal]).unwrap();
        ingest.push(&[ordinal ^ 0x5A]).unwrap();
        let published = ingest.finish().unwrap();
        let marker = serving
            .certification_selected_latest_blob_publication()
            .unwrap()
            .unwrap();
        model.push((
            PhysicalIndexPointKey::blob_catalog(object, published.generation().sequence()).unwrap(),
            object,
            marker.record(),
        ));
    }
    model.sort_by_key(|(key, _, _)| key.canonical_bytes());
    serving.close();

    let reopened = serving_from_open(root.path());
    let layouts = reopened.layouts().unwrap();
    let catalog = layouts.btree(DurableArtifactFamilyId::BlobCatalog).unwrap();
    let ordinary = catalog.point(model[0].0).unwrap();
    for (key, _, record) in &model {
        let point = catalog.point(*key).unwrap();
        assert_eq!(point.selected_record(), Some(*record));
        assert_eq!(
            point.counters().page_touches(),
            point.counters().node_decodes()
        );
        assert_eq!(
            point.counters().page_touches(),
            2,
            "production insertion split the root"
        );
    }
    catalog.certification_inject_extra_protected_page_read();
    let injected = catalog.point(model[0].0).unwrap();
    assert_eq!(injected.selected_record(), ordinary.selected_record());
    assert_eq!(
        injected.counters().node_decodes(),
        ordinary.counters().node_decodes()
    );
    assert_eq!(
        injected.counters().page_touches(),
        ordinary.counters().page_touches() + 1
    );
    let mut short = catalog
        .range(
            PhysicalIndexRange::from(model[0].0),
            PhysicalIndexScanBudget::pages(2).unwrap(),
        )
        .unwrap();
    assert!(short
        .by_ref()
        .any(|entry| matches!(entry, Err(PhysicalLayoutDenial::ScanBudgetExhausted))));
    let full = catalog
        .range(
            PhysicalIndexRange::from(model[0].0),
            PhysicalIndexScanBudget::pages(64).unwrap(),
        )
        .unwrap();
    let observed = full
        .map(Result::unwrap)
        .map(|entry| (entry.canonical_key(), entry.selected_record()))
        .collect::<Vec<_>>();
    let expected = model
        .iter()
        .map(|(key, _, record)| (key.canonical_bytes(), *record))
        .collect::<Vec<_>>();
    assert_eq!(observed, expected);
    let prefix = catalog
        .prefix(
            PhysicalIndexPrefix::blob_object(model[15].1),
            PhysicalIndexScanBudget::pages(64).unwrap(),
        )
        .unwrap()
        .map(Result::unwrap)
        .collect::<Vec<_>>();
    assert_eq!(prefix.len(), 1);
    assert_eq!(prefix[0].selected_record(), model[15].2);
    drop(short);
    drop(catalog);
    drop(layouts);
    reopened.close();

    let report = observe_closed_store_named(root.path(), "c11-layout-cost", "production-split");
    assert_eq!(report["completeness"], "complete", "{report}");
    let roots = report["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|artifact| {
            artifact["family"] == "btree_node"
                && artifact["index_family"] == "blob_catalog"
                && artifact.get("expected_point_page_touches").is_some()
        })
        .collect::<Vec<_>>();
    let node_postures = report["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|artifact| {
            artifact["family"] == "btree_node"
                || artifact["family"] == "derived_family_root_directory"
                || (artifact["family"] == "root_manifest"
                    && artifact["outcome"]["posture"] == "intact")
        })
        .map(|artifact| {
            (
                &artifact["family"],
                &artifact["identity"],
                &artifact["generation"],
                &artifact["path"],
                &artifact["range"],
                &artifact["outcome"],
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        roots.len(),
        1,
        "one complete catalog root: {node_postures:?}"
    );
    assert_eq!(roots[0]["outcome"]["posture"], "intact");
    let expected = roots[0]["expected_point_page_touches"].as_u64().unwrap();
    assert_eq!(ordinary.counters().page_touches(), expected);
    assert_eq!(injected.counters().page_touches(), expected + 1);
}
