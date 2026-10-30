//! A real failed-ingest V1 control survives a NoRelease checkpoint, two
//! distinct released-generation V3 drops, and an ordinary selected tail.
//! Store must rejoin that otherwise unrelated selected control independently.

#[allow(unused_imports)]
use crate::production_entry::mixed_retained_control::*;

#[test]
fn mixed_v1_two_v3_historical_rejoin_opens_then_resealed_v1_denies_serving() {
    let world = worth_store_testkit_recovery_runtime::production_entry::pending_wal_world::first_with_failed_ingest_control();
    world.kill_distinct_release_before_checkpoint();
    let checkpoint = std::fs::read(world.root().join("families/checkpoint.current")).unwrap();
    let first = recover(world.root());
    let seal = first
        .into_core()
        .certification_release_media_into_checkpoint_custody()
        .expect("two real V3 drops must receive Store seal");
    let serving = worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::admit_serving_with_seal(world.root(), seal);
    let before_tail = selected_generation(world.root());
    append_ordinary_tail(&serving);
    assert!(
        selected_generation(world.root()) > before_tail,
        "ordinary publication must advance the selected root after both V3 drops"
    );
    serving.close();
    assert_eq!(
        std::fs::read(world.root().join("families/checkpoint.current")).unwrap(),
        checkpoint,
        "NoRelease checkpoint must remain selected across both V3s and ordinary tail"
    );

    let fresh = recover(world.root());
    let routes = fresh.selected_sources().page_facts().placements();
    assert_eq!(
        routes
            .iter()
            .filter(|route| {
                route.content_class()
                    == worth_store_physical_format::SelectedRecordContentClass::Blob(
                        worth_store_physical_format::BlobRecordKind::ReclaimDescriptorV3,
                    )
            })
            .count(),
        2,
        "both genuine V3 results must remain selected"
    );
    assert!(
        routes.iter().any(|route| {
            route.content_class()
                == worth_store_physical_format::SelectedRecordContentClass::Blob(
                    worth_store_physical_format::BlobRecordKind::DropSetManifestV2,
                )
        }),
        "genuine failed-ingest V2 manifest must remain selected"
    );
    let descriptor = routes
        .iter()
        .copied()
        .find(|route| {
            route.content_class()
                == worth_store_physical_format::SelectedRecordContentClass::Blob(
                    worth_store_physical_format::BlobRecordKind::ReclaimDescriptor,
                )
        })
        .expect("failed-ingest V1 descriptor must survive the ordinary selected tail");
    let original_frame = read_v1_descriptor_frame(world.root(), descriptor);
    let seal = fresh
        .into_core()
        .certification_release_media_into_checkpoint_custody()
        .expect("historical-only mixed custody must seal");
    worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::open_serving_with_seal_without_checkpoint(world.root(), seal);

    let negative = recover(world.root());
    let seal = negative
        .into_core()
        .certification_release_media_into_checkpoint_custody()
        .expect("unchanged mixed media must receive a second genuine Store seal");
    reseal_v1_descriptor(world.root(), descriptor, &original_frame);
    worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::open_serving_with_seal_expect_mismatch(world.root(), seal);
}
