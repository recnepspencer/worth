#[allow(unused_imports)]
use crate::physical_blob_journeys::layout_corruption_rebuild::*;

use {
    crate::physical_blob_journeys::blob_ingest_process::observe_closed_store_named,
    worth_store_testkit_store::fixture::admitted_blob_scope,
    worth_store_testkit_store::fixture::placement,
    worth_store_testkit_store::fixture::serving_from_initialization,
    worth_store_testkit_store::fixture::serving_from_open,
};

#[test]
fn damaged_catalog_leaf_denies_point_then_rebuilds_from_selected_blob_authority() {
    let root = tempfile::tempdir().unwrap();
    let scope = admitted_blob_scope("c11.layout.corrupt-leaf.rebuild");
    let limits =
        worth_store::physical_runtime::BlobReadLimits::new(std::num::NonZeroU64::new(128).unwrap());
    let (object, published, original) = {
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
        ingest.push(&[0x31]).unwrap();
        ingest.push(&[0x72]).unwrap();
        let published = ingest.finish().unwrap();
        let key = worth_store::physical_runtime::PhysicalIndexPointKey::blob_catalog(
            object,
            published.generation().sequence(),
        )
        .unwrap();
        let original = serving
            .layouts()
            .unwrap()
            .btree(worth_store_contracts::DurableArtifactFamilyId::BlobCatalog)
            .unwrap()
            .point(key)
            .unwrap()
            .selected_record()
            .unwrap();
        serving.close();
        (object, published, original)
    };

    let before = observe_closed_store_named(root.path(), "c11-leaf-rebuild", "before");
    assert_eq!(before["completeness"], "complete", "{before}");
    let old_node_identity = corrupt_selected_catalog_leaf(root.path(), &before);
    let damaged = observe_closed_store_named(root.path(), "c11-leaf-rebuild", "damaged");
    let damaged_node = damaged["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["identity"] == old_node_identity)
        .expect("same selected catalog node after mutation");
    assert_eq!(damaged_node["family"], "btree_node");
    assert_eq!(damaged_node["outcome"]["posture"], "damaged", "{damaged}");

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
        Err(worth_store::physical_runtime::PhysicalLayoutDenial::NodeIntegrity(_))
    ));
    assert!(btree.damaged_node_scrub_target().unwrap().is_some());
    let foreign_root = tempfile::tempdir().unwrap();
    let foreign_serving = serving_from_initialization(foreign_root.path());
    let foreign_object = foreign_serving
        .blobs()
        .unwrap()
        .issue_object_id(limits)
        .unwrap();
    let foreign_key = worth_store::physical_runtime::PhysicalIndexPointKey::blob_catalog(
        foreign_object,
        published.generation().sequence(),
    )
    .unwrap();
    assert!(matches!(
        btree.point(foreign_key),
        Err(worth_store::physical_runtime::PhysicalLayoutDenial::ForeignStore)
    ));
    assert!(
        btree.damaged_node_scrub_target().unwrap().is_none(),
        "a new denied request must not inherit a prior damaged-node target"
    );
    foreign_serving.close();
    assert!(matches!(
        btree.point(key),
        Err(worth_store::physical_runtime::PhysicalLayoutDenial::NodeIntegrity(_))
    ));
    let target = btree
        .damaged_node_scrub_target()
        .unwrap()
        .expect("selected D1 lookup issues exact damaged-node diagnostic target");
    let target_record = target.scope().btree_node_identity().unwrap().0;
    assert_eq!(index_identity(target_record), old_node_identity);
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
        panic!("selected catalog leaf scrub must inspect its bounded window")
    };
    assert_eq!(window.scope.btree_node_identity().unwrap().0, target_record);
    assert!(matches!(
        window.outcome,
        worth_store_physical_integrity::PhysicalIntegrityObservationOutcome::Rejected(worth_store_physical_integrity::PhysicalIntegrityRejection::Damaged(
            localization
        )) if localization.scope().btree_node_identity().unwrap().0 == target_record
    ));
    assert_eq!(window.counters.damaged_windows, 1);
    drop(scrub);
    let selected_records_before_rebuild = selected_record_ids(&serving);
    let bounded = worth_store::physical_runtime::LayoutRebuildLimits::new(
        std::num::NonZeroU64::new(1_000).unwrap(),
        std::num::NonZeroU64::new(1_000).unwrap(),
    );
    let rebuilt = serving
        .layouts()
        .unwrap()
        .rebuild(
            worth_store_contracts::DurableArtifactFamilyId::BlobCatalog,
            bounded,
            placement(),
            worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000)
                .unwrap(),
        )
        .unwrap();
    // The receipt counts authoritative logical chunks preflighted, not the
    // later second traversal's physical validation operations.
    assert_eq!(rebuilt.validated_chunks(), 1);
    assert_eq!(
        rebuilt.scanned_records(),
        selected_records_before_rebuild.len() as u64
    );
    assert_eq!(rebuilt.deferred_derived_root_count(), 1);
    assert_eq!(rebuilt.deferred_dedupe_root(), None);
    assert_eq!(rebuilt.deferred_dedupe_cause(), None);
    assert!(matches!(
        rebuilt.deferred_catalog_cause(),
        Some(worth_store::physical_runtime::DeferredDerivedRetirementCause::NodeDamaged(_))
    ));
    let deferred_catalog = rebuilt
        .deferred_catalog_root()
        .expect("damaged old catalog root must be retained for later recovery");
    assert_eq!(index_identity(deferred_catalog), old_node_identity);
    assert!(
        selected_record_ids(&serving).contains(&deferred_catalog),
        "unproved old derived closure must remain selected, not be dropped"
    );
    assert_eq!(
        serving
            .layouts()
            .unwrap()
            .btree(worth_store_contracts::DurableArtifactFamilyId::BlobCatalog)
            .unwrap()
            .point(key)
            .unwrap()
            .selected_record(),
        Some(original)
    );
    serving.close();
    let after = observe_closed_store_named(root.path(), "c11-leaf-rebuild", "rebuilt");
    assert_eq!(after["completeness"], "complete", "{after}");
    let new_node_identity = index_identity(rebuilt.catalog_root());
    let new_node = after["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["identity"] == new_node_identity)
        .expect("offline selected walk reaches replacement catalog root");
    assert_eq!(new_node["outcome"]["posture"], "intact", "{after}");
    assert!(
        !after["artifacts"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["identity"] == old_node_identity),
        "offline selected-root walk must not present unreachable old residue as a live index node"
    );
    let reopened = serving_from_open(root.path());
    assert_eq!(
        reopened
            .layouts()
            .unwrap()
            .btree(worth_store_contracts::DurableArtifactFamilyId::BlobCatalog)
            .unwrap()
            .point(key)
            .unwrap()
            .selected_record(),
        Some(original)
    );
    reopened.close();
}
