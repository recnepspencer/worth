#[allow(unused_imports)]
use crate::physical_blob_journeys::layout_shape_corruption::*;

use {
    crate::physical_blob_journeys::blob_ingest_process::observe_closed_store_named,
    worth_store_testkit_store::fixture::admitted_blob_scope,
    worth_store_testkit_store::fixture::placement,
    worth_store_testkit_store::fixture::serving_from_initialization,
    worth_store_testkit_store::fixture::serving_from_open,
};

#[test]
fn resealed_valid_wrong_catalog_cell_width_is_damaged_to_store_and_offline_scrub() {
    let root = tempfile::tempdir().unwrap();
    let scope = admitted_blob_scope("c11.catalog.shape.corrupt");
    let limits =
        worth_store::physical_runtime::BlobReadLimits::new(std::num::NonZeroU64::new(128).unwrap());
    let (object, published) = {
        let serving = serving_from_initialization(root.path());
        let blobs = serving.blobs().unwrap();
        let object = blobs.issue_object_id(limits).unwrap();
        let declaration = worth_store::physical_runtime::BlobIngestDeclaration::new(
            object,
            worth_store_blob_chunks::BlobChunkSize::from_bytes(64 << 10).unwrap(),
            2,
            &scope,
            worth_store::physical_runtime::BlobCheckpointLimit::bounded_horizon(16).unwrap(),
            worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000)
                .unwrap(),
        )
        .unwrap();
        let mut ingest = blobs
            .begin_ingest(declaration, placement(), 1, limits)
            .unwrap();
        ingest.push(&[0x41]).unwrap();
        ingest.push(&[0x42]).unwrap();
        let published = ingest.finish().unwrap();
        serving.close();
        (object, published)
    };
    let before = observe_closed_store_named(root.path(), "c11-width-corruption", "before");
    let identity = reshape_selected_catalog_leaf(root.path(), &before);
    let after = observe_closed_store_named(root.path(), "c11-width-corruption", "damaged");
    let row = after["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["identity"] == identity)
        .expect("independent selected walk finds same leaf");
    assert_eq!(row["family"], "btree_node");
    assert_eq!(row["outcome"]["posture"], "damaged", "{after}");

    let serving = serving_from_open(root.path());
    let key = worth_store::physical_runtime::PhysicalIndexPointKey::blob_catalog(
        object,
        published.generation().sequence(),
    )
    .unwrap();
    let layouts = serving.layouts().unwrap();
    let btree = layouts
        .btree(worth_store_contracts::DurableArtifactFamilyId::BlobCatalog)
        .unwrap();
    assert!(matches!(
        btree.point(key),
        Err(worth_store::physical_runtime::PhysicalLayoutDenial::NodeFamilyMismatch)
    ));
    let target = btree
        .damaged_node_scrub_target()
        .unwrap()
        .expect("registered-family shape denial issues selected diagnostic target");
    let target_record = target.scope().btree_node_identity().unwrap().0;
    assert_eq!(index_identity(target_record), identity);
    drop(layouts);
    let request = worth_store::physical_runtime::ManagedPhysicalIntegrityScrubRequest::new(
        serving.store_identity(),
        [target],
        1 << 20,
        1 << 20,
        std::time::Duration::from_secs(30),
    )
    .unwrap();
    let mut scrub = serving.start_physical_integrity_scrub(request).unwrap();
    let worth_store::physical_runtime::ManagedPhysicalIntegrityScrubProgress::WindowInspected(
        window,
    ) = scrub.next_window()
    else {
        panic!("selected malformed catalog node must be inspected")
    };
    assert_eq!(window.scope.btree_node_identity().unwrap().0, target_record);
    assert!(matches!(
        window.outcome,
        worth_store_physical_integrity::PhysicalIntegrityObservationOutcome::Rejected(worth_store_physical_integrity::PhysicalIntegrityRejection::Damaged(
            localization
        )) if localization.scope().btree_node_identity().unwrap().0 == target_record
            && localization.cause() == worth_store_physical_integrity::PhysicalDamageCause::MalformedStructure
            && localization.field() == Some(worth_store_physical_integrity::PhysicalFormatField::Payload)
    ));
    assert_eq!(window.validation_counters.inspected_frames(), 1);
    assert_eq!(window.validation_counters.rejected_frames(), 1);
    assert_eq!(window.validation_counters.intact_frames(), 0);
    assert_eq!(window.counters.completed_windows, 1);
    assert_eq!(window.counters.validated_windows, 0);
    assert_eq!(window.counters.damaged_windows, 1);
}
