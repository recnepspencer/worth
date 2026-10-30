//! The genuine KiB64 publication boundary, before the longer C8 reuse journey.

#[allow(unused_imports)]
use crate::production_entry::release_reopen::shared_reuse_custody::publication_budget::*;

use {
    crate::production_entry::release_reopen::shared_reuse_custody::read_exact,
    crate::production_entry::release_reopen::shared_reuse_custody::selected_marker,
    crate::production_entry::release_reopen::shared_reuse_custody::shared_format,
    crate::production_entry::release_reopen::shared_reuse_custody::CHUNK,
    worth_store_testkit_recovery_runtime::production_entry::blob_scope_admission::admitted_blob_scope,
};

#[test]
fn ki_b64_original_and_reused_publications_are_indexed_and_readable() {
    std::thread::Builder::new()
        .name("shared-publication-catalog".to_owned())
        .stack_size(16 << 20)
        .spawn(|| {
            let world = worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld::initialize_for_recovery_with_format_and_manifest_capacity(
                    "c11-shared-publication-catalog",
                    shared_format(),
                    MANIFEST_CAPACITY,
                )
                .expect("32 MiB KiB64 production admission");
            let scope = admitted_blob_scope("c11.shared.catalog.budget.positive");
            let payload = crate::production_entry::release_reopen::shared_reuse_custody::world::payload();
            let original = crate::production_entry::release_reopen::shared_reuse_custody::world::publish_one(&world, &scope, &payload);
            assert_catalog_publication(&world, &scope, original);

            let destination = crate::production_entry::release_reopen::shared_reuse_custody::world::publish_one(&world, &scope, &payload);
            assert_catalog_publication(&world, &scope, original);
            assert_catalog_publication(&world, &scope, destination);
            assert_eq!(read_exact(world.serving(), &scope, original, &payload), 0);
            // Each of two reused chunks selects its source chunk, publication,
            // and leaf edge while the original publication is still routed.
            assert_eq!(
                read_exact(world.serving(), &scope, destination, &payload),
                6
            );
            world.close();
        })
        .expect("publication worker")
        .join()
        .expect("publication worker did not panic");
}

#[test]
fn low_maintenance_scope_preserves_durable_publication_without_catalog_validation() {
    std::thread::Builder::new()
        .name("shared-publication-maintenance-denial".to_owned())
        .stack_size(16 << 20)
        .spawn(|| {
            let world =
                worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld::initialize_for_recovery_with_maintenance_scope(
                    "c11-shared-publication-maintenance-denial",
                    shared_format(),
                    MANIFEST_CAPACITY,
                    std::num::NonZeroU64::new(MAINTENANCE_LOW_BYTES).unwrap(),
                )
                .expect("same KiB64 world with only Maintenance scope narrowed");
            let serving = world.serving();
            let policy = serving.residency_observation().admitted_policy();
            assert_eq!(policy.operation_bytes(), 32 << 20);
            assert_eq!(
                policy.scope_bytes(worth_store::physical_runtime::PhysicalOperationAllocationScope::Maintenance),
                MAINTENANCE_LOW_BYTES
            );
            let scope = admitted_blob_scope("c11.shared.catalog.budget.negative");
            let payload = crate::production_entry::release_reopen::shared_reuse_custody::world::payload();
            let blobs = serving.blobs().unwrap();
            let limits = worth_store::physical_runtime::BlobReadLimits::new(std::num::NonZeroU64::new(256).unwrap());
            let object = blobs
                .issue_object_id(limits)
                .expect("Store-issued identity");
            let declaration = worth_store::physical_runtime::BlobIngestDeclaration::new(
                object,
                worth_store_blob_chunks::BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
                payload.len() as u64,
                &scope,
                worth_store::physical_runtime::BlobCheckpointLimit::bounded_horizon(16).unwrap(),
                worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
            )
            .unwrap();
            let mut ingest = blobs
                .begin_ingest(declaration, world.placement(), (CHUNK / 2) as u64, limits)
                .expect("physical ingest must be admitted before maintenance denial");
            for piece in payload.chunks(CHUNK / 2) {
                ingest.push(piece).expect("physical chunk append");
            }
            assert!(serving
                .certification_selected_latest_blob_publication()
                .unwrap()
                .is_none());
            let before_publication = serving
                .records()
                .unwrap()
                .protected_root()
                .root()
                .generation()
                .get();
            // The source-validation walk needs a 4 MiB Maintenance grant;
            // this limit denies there, before derived-tree retirement.
            let (published, validation) = match ingest.finish() {
                Err(worth_store::physical_runtime::BlobIngestFailure::PublishedIndexPending {
                    published,
                    cause: worth_store::physical_runtime::PhysicalLayoutMaintenanceFailure::FullAuthority(validation),
                }) => (published, validation),
                other => {
                    panic!("catalog validation must deny after durable publication: {other:?}")
                }
            };
            let failure = match *validation {
                worth_store::physical_runtime::LayoutRebuildFailure::Allocation(failure) => failure,
                other => panic!("catalog validation must retain typed allocation cause: {other:?}"),
            };
            assert_eq!(
                failure.reason(),
                worth_store::physical_runtime::PhysicalRecordResidencyFailureReason::PhysicalPressure
            );
            let pressure = failure.pressure().expect("typed Maintenance pressure");
            assert_eq!(
                pressure.scope(),
                worth_store::physical_runtime::PhysicalOperationAllocationScope::Maintenance
            );
            assert_eq!(pressure.limit(), MAINTENANCE_LOW_BYTES);
            assert_eq!(pressure.requested(), 4 << 20);
            assert_eq!(pressure.admitted(), 0);
            assert_eq!(
                pressure.retry_posture(),
                worth_store::physical_runtime::PhysicalResidencyRetryPosture::AfterConfigurationChange
            );
            assert!(!pressure.effect_may_have_started());

            let marker = selected_marker(serving);
            assert_eq!(published.object(), object);
            assert!(
                serving
                    .records()
                    .unwrap()
                    .protected_root()
                    .root()
                    .generation()
                    .get()
                    > before_publication
            );
            assert!(matches!(
                serving
                    .blobs()
                    .unwrap()
                    .resolve_publication(
                        object.bytes(),
                        published.generation().sequence(),
                        &scope,
                        limits,
                    ),
                Err(worth_store::physical_runtime::BlobReadOpenFailure::Layout(
                    worth_store::physical_runtime::PhysicalLayoutDenial::IndexStaleRequiresRebuild {
                        latest,
                        indexed: None,
                    }
                )) if latest == marker
            ));
            world.close();
        })
        .expect("maintenance-denial worker")
        .join()
        .expect("maintenance-denial worker did not panic");
}
