use super::tests::{free_entry, record, route, snapshot};
use super::*;
use worth_store_physical_format::{
    DerivedFamilyRootDirectoryBinding, IndexedThroughBlobPublication,
};

#[test]
fn dropped_indexed_watermark_requires_atomic_directory_replacement() {
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let source_routes = [route(1, 1, 0), route(2, 2, 4096)];
    let result_routes = [route(2, 2, 4096), route(3, 3, 8192)];
    let projected = [route(3, 3, 8192)];
    let dropped = [record(1)];
    let source_free_entries = [free_entry(8192, 12288, 1)];
    let result_free_entries = [free_entry(12288, 8192, 2)];
    let watermark = IndexedThroughBlobPublication::new(1, record(1), [1; 32]).unwrap();
    let (source_unbound, source_free) = snapshot(
        1,
        &source_routes,
        source_free_entries[0],
        3,
        Some(watermark),
        true,
        format,
    );
    let selected_directory = DerivedFamilyRootDirectoryBinding::new(record(2), Some(watermark));
    let source_root = DurablePhysicalRootManifest::builder(
        source_unbound.generation(),
        source_unbound.tree_identity(),
        source_unbound.node_capacity(),
        source_unbound.free_space_checksum(),
    )
    .record_count(source_unbound.record_count())
    .routing_root(source_unbound.routing_root())
    .free_space_root(source_unbound.free_space_root())
    .next_block(source_unbound.next_block())
    .latest_blob_publication(Some(watermark))
    .derived_family_directory(Some(selected_directory))
    .admit()
    .unwrap()
    .with_maintenance_protocol();
    let (result_root, result_free) = snapshot(
        2,
        &result_routes,
        result_free_entries[0],
        4,
        None,
        true,
        format,
    );
    let source = ReleasedInventoryView::new(
        &source_root,
        &source_free,
        &source_routes,
        &[],
        &source_free_entries,
    );
    let result = ReleasedInventoryView::new(
        &result_root,
        &result_free,
        &result_routes,
        &[],
        &result_free_entries,
    );
    assert!(!root_semantics_match(
        source, result, &dropped, &projected, None, None
    ));
    assert_eq!(
        VerifiedReleasedV3InventoryTransition::admit(
            source,
            result,
            &dropped,
            &projected,
            format,
            16,
            1 << 20,
            None,
        ),
        Err(ReleasedV3InventoryTransitionDenial::InvalidDelta),
    );
}
