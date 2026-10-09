use worth_store::physical_runtime::{
    PhysicalIndexRange, PhysicalIndexScanBudget, PhysicalLayoutDenial, PhysicalMutationDeadline,
};
use worth_store_contracts::DurableArtifactFamilyId;

use super::fixture::{placement, serving_from_initialization};

#[test]
fn protected_layout_probe_height_two_has_exact_path_and_range_budget() {
    let root = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(root.path());
    let keys = serving
        .certification_publish_blob_catalog_probe_tree(
            2,
            placement(),
            PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
        )
        .unwrap();
    let layouts = serving.layouts().unwrap();
    let index = layouts.btree(DurableArtifactFamilyId::BlobCatalog).unwrap();
    for key in &keys {
        let point = index.point(*key).unwrap();
        assert!(point.selected_record().is_some());
        assert_eq!(point.counters().page_touches(), 2);
        assert_eq!(point.counters().node_decodes(), 2);
    }
    let ordinary = index.point(keys[0]).unwrap();
    index.certification_inject_extra_protected_page_read();
    let injected = index.point(keys[0]).unwrap();
    assert_eq!(injected.selected_record(), ordinary.selected_record());
    assert_eq!(
        injected.counters().page_touches(),
        ordinary.counters().page_touches() + 1
    );
    assert_eq!(
        injected.counters().node_decodes(),
        ordinary.counters().node_decodes()
    );
    let mut short = index
        .range(
            PhysicalIndexRange::from(keys[0]),
            PhysicalIndexScanBudget::pages(2).unwrap(),
        )
        .unwrap();
    assert_eq!(
        short.next().unwrap().unwrap().canonical_key(),
        keys[0].canonical_bytes()
    );
    assert!(matches!(
        short.next(),
        Some(Err(PhysicalLayoutDenial::ScanBudgetExhausted))
    ));
    assert_eq!(short.counters().page_touches(), 2);
    assert!(short.next().is_none());
    let mut full = index
        .range(
            PhysicalIndexRange::from(keys[0]),
            PhysicalIndexScanBudget::pages(3).unwrap(),
        )
        .unwrap();
    let found: Vec<_> = full.by_ref().map(Result::unwrap).collect();
    assert_eq!(found.len(), 2);
    assert_eq!(full.counters().page_touches(), 3);
    drop(short);
    drop(full);
    drop(index);
    drop(layouts);
    serving.close();
}

#[test]
fn protected_layout_probe_height_three_counts_actual_node_path() {
    let root = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(root.path());
    let keys = serving
        .certification_publish_blob_catalog_probe_tree(
            3,
            placement(),
            PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
        )
        .unwrap();
    let layouts = serving.layouts().unwrap();
    let index = layouts.btree(DurableArtifactFamilyId::BlobCatalog).unwrap();
    for key in &keys {
        assert_eq!(index.point(*key).unwrap().counters().page_touches(), 3);
    }
    let mut across = index
        .range(
            PhysicalIndexRange::between(keys[1], keys[3]).unwrap(),
            PhysicalIndexScanBudget::pages(5).unwrap(),
        )
        .unwrap();
    let found: Vec<_> = across.by_ref().map(Result::unwrap).collect();
    assert_eq!(
        found
            .iter()
            .map(|entry| entry.canonical_key())
            .collect::<Vec<_>>(),
        vec![keys[1].canonical_bytes(), keys[2].canonical_bytes()]
    );
    assert_eq!(across.counters().page_touches(), 5);
    drop(across);
    drop(index);
    drop(layouts);
    serving.close();
}

#[test]
fn protected_layout_probe_height_five_remains_readable_with_bounded_paths() {
    let root = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(root.path());
    let keys = serving
        .certification_publish_blob_catalog_probe_tree(
            5,
            placement(),
            PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
        )
        .unwrap();
    assert_eq!(keys.len(), 16);
    let layouts = serving.layouts().unwrap();
    let index = layouts.btree(DurableArtifactFamilyId::BlobCatalog).unwrap();
    for key in &keys {
        let point = index.point(*key).unwrap();
        assert!(point.selected_record().is_some());
        assert_eq!(point.counters().page_touches(), 5);
        assert_eq!(point.counters().node_decodes(), 5);
    }
    let mut scan = index
        .range(
            PhysicalIndexRange::from(keys[0]),
            PhysicalIndexScanBudget::pages(5).unwrap(),
        )
        .unwrap();
    assert_eq!(
        scan.next().unwrap().unwrap().canonical_key(),
        keys[0].canonical_bytes()
    );
    assert_eq!(scan.counters().page_touches(), 5);
    drop(scan);
    drop(index);
    drop(layouts);
    serving.close();
}
