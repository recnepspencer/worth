//! Real process kill after the released V3 drop descriptor WAL is durable,
//! before its root is published. C8 must roll the exact release forward.

#[allow(unused_imports)] use crate::physical_blob_journeys::blob_reclaim_released_crash::*;
use {std::num::NonZeroU16, std::num::NonZeroU64, std::path::Path, std::thread, std::time::Duration, std::time::Instant};

use {worth_proof::AdmittedBlobReleaseProof};
use {worth_store::physical_runtime::production::PhysicalMutationCheckpoint, worth_store::physical_runtime::BlobCheckpointLimit, worth_store::physical_runtime::BlobIngestDeclaration, worth_store::physical_runtime::BlobReadLimits, worth_store::physical_runtime::BlobReclaimLimits, worth_store::physical_runtime::BlobReclaimRequest, worth_store::physical_runtime::PhysicalMutationDeadline};
use {worth_store_blob_chunks::BlobChunkSize};
use {worth_store_physical_format::BlobRecordKind, worth_store_physical_format::SelectedRecordContentClass};

use {crate::physical_blob_journeys::blob_crash::establish_recovery_frontier, crate::physical_blob_journeys::blob_crash::kill_at, crate::physical_blob_journeys::blob_crash::marker_path, crate::blob_crash::recovery::recover_closed_store, crate::physical_blob_journeys::blob_crash::write_marker, worth_store_testkit_store::fixture::admitted_blob_scope, worth_store_testkit_store::fixture::assert_released_open_requires_recovered_custody, worth_store_testkit_store::fixture::configuration, worth_store_testkit_store::fixture::placement, worth_store_testkit_store::fixture::serving_from_initialization};

#[test]
fn durable_released_v3_descriptor_replays_exact_publication_drop_and_retirement() {
    let world = kill_at(ROLE, std::time::Duration::from_secs(240));
    let (before_root, format) = crate::blob_reclaim_released_crash::retirement_witness::selected_root(&world.root);
    let retired_directories = crate::blob_reclaim_released_crash::derived_witness::retired_inline_directory_slots(&world.root, format);
    let (publication_record, publication_extent) =
        crate::blob_reclaim_released_crash::retirement_witness::selected_publication_extent(&world.root, &before_root);
    recover_closed_store(&world.root);
    let (after_root, after_format) = crate::blob_reclaim_released_crash::retirement_witness::selected_root(&world.root);
    assert_eq!(after_format, format);
    assert!(
        !crate::blob_reclaim_released_crash::retirement_witness::range_published_free(
            &world.root,
            &after_root,
            publication_extent.arena_range(),
            before_root.generation(),
        ),
        "dropped native extent is still owed retirement, not already published free"
    );
    assert!(
        after_root.requires_maintenance_protocol(),
        "C8 drop root must retain displaced native extent retirement protocol"
    );
    // A released drop leaves a release-custody head, so the only path to
    // Serving is C8's handoff; an ordinary reopen must deny before effects.
    assert_released_open_requires_recovered_custody(&world.root, configuration().0);
    let placements = crate::blob_reclaim_released_crash::retirement_witness::selected_placements(&world.root, &after_root);
    for retired_directory in retired_directories {
        assert!(
            placements
                .iter()
                .all(|placement| placement.record() != retired_directory.record),
            "physically witnessed retired inline slot {:?}/{:?} must not become a live route",
            retired_directory.page,
            retired_directory.slot,
        );
    }
    let count = |kind| {
        placements
            .iter()
            .filter(|placement| placement.content_class() == worth_store_physical_format::SelectedRecordContentClass::Blob(kind))
            .count()
    };
    assert!(
        placements
            .iter()
            .all(|placement| placement.record() != publication_record),
        "redo must unroute the released publication"
    );
    assert_eq!(count(worth_store_physical_format::BlobRecordKind::GenerationPublished), 0);
    assert_eq!(
        (
            count(worth_store_physical_format::BlobRecordKind::Chunk),
            count(worth_store_physical_format::BlobRecordKind::TreeNode)
        ),
        (2, 1),
        "first bounded batch keeps payload selected"
    );
    assert_eq!(count(worth_store_physical_format::BlobRecordKind::DropSetManifestV3), 1);
    assert_eq!(
        count(worth_store_physical_format::BlobRecordKind::ReclaimDescriptorV3),
        1,
        "C8 must publish the durable V3 descriptor"
    );
    // The dropped publication was the indexed watermark: the same root binds a
    // replacement directory that no longer names it, rather than clearing it.
    let before_directory = before_root
        .derived_family_directory()
        .expect("released publication is indexed by the selected directory");
    assert_eq!(
        before_directory
            .indexed_through_blob_publication()
            .map(|publication| publication.record()),
        Some(publication_record)
    );
    let after_directory = after_root
        .derived_family_directory()
        .expect("C8 drop root binds the replacement directory");
    assert_ne!(
        after_directory.directory_record(),
        before_directory.directory_record()
    );
    assert_eq!(after_directory.indexed_through_blob_publication(), None);
    assert!(placements
        .iter()
        .any(|placement| placement.record() == after_directory.directory_record()));
    assert_eq!(after_root.latest_blob_publication(), None);
}
#[path = "repeat_c8/mod.rs"]
pub(crate) mod repeat_c8;
