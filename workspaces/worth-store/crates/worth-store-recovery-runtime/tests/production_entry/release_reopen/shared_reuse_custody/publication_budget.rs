//! The genuine KiB64 publication boundary, before the longer C8 reuse journey.

use std::num::NonZeroU64;

use worth_store::physical_runtime::{
    AdmittedBlobScope, BlobCheckpointLimit, BlobIngestDeclaration, BlobIngestFailure,
    BlobReadLimits, BlobReadOpenFailure, LayoutRebuildFailure, PhysicalLayoutDenial,
    PhysicalLayoutMaintenanceFailure, PhysicalMutationDeadline, PhysicalOperationAllocationScope,
    PhysicalRecordResidencyFailureReason, PhysicalResidencyRetryPosture, PublishedBlobGeneration,
};
use worth_store_blob_chunks::BlobChunkSize;
use worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld;

use super::{admitted_blob_scope, read_exact, selected_marker, shared_format, world, CHUNK};

const MANIFEST_CAPACITY: u16 = 512;
const MAINTENANCE_LOW_BYTES: u64 = 1 << 20;

#[test]
fn ki_b64_original_and_reused_publications_are_indexed_and_readable() {
    std::thread::Builder::new()
        .name("shared-publication-catalog".to_owned())
        .stack_size(16 << 20)
        .spawn(|| {
            let world = PhysicalResidencyStoreWorld::
                initialize_for_recovery_with_format_and_manifest_capacity(
                    "c11-shared-publication-catalog",
                    shared_format(),
                    MANIFEST_CAPACITY,
                )
                .expect("32 MiB KiB64 production admission");
            let scope = admitted_blob_scope("c11.shared.catalog.budget.positive");
            let payload = world::payload();
            let original = world::publish_one(&world, &scope, &payload);
            assert_catalog_publication(&world, &scope, original);

            let destination = world::publish_one(&world, &scope, &payload);
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
                PhysicalResidencyStoreWorld::initialize_for_recovery_with_maintenance_scope(
                    "c11-shared-publication-maintenance-denial",
                    shared_format(),
                    MANIFEST_CAPACITY,
                    NonZeroU64::new(MAINTENANCE_LOW_BYTES).unwrap(),
                )
                .expect("same KiB64 world with only Maintenance scope narrowed");
            let serving = world.serving();
            let policy = serving.residency_observation().admitted_policy();
            assert_eq!(policy.operation_bytes(), 32 << 20);
            assert_eq!(
                policy.scope_bytes(PhysicalOperationAllocationScope::Maintenance),
                MAINTENANCE_LOW_BYTES
            );
            let scope = admitted_blob_scope("c11.shared.catalog.budget.negative");
            let payload = world::payload();
            let blobs = serving.blobs().unwrap();
            let limits = BlobReadLimits::new(NonZeroU64::new(256).unwrap());
            let object = blobs
                .issue_object_id(limits)
                .expect("Store-issued identity");
            let declaration = BlobIngestDeclaration::new(
                object,
                BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
                payload.len() as u64,
                &scope,
                BlobCheckpointLimit::bounded_horizon(16).unwrap(),
                PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
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
                Err(BlobIngestFailure::PublishedIndexPending {
                    published,
                    cause: PhysicalLayoutMaintenanceFailure::FullAuthority(validation),
                }) => (published, validation),
                other => {
                    panic!("catalog validation must deny after durable publication: {other:?}")
                }
            };
            let failure = match *validation {
                LayoutRebuildFailure::Allocation(failure) => failure,
                other => panic!("catalog validation must retain typed allocation cause: {other:?}"),
            };
            assert_eq!(
                failure.reason(),
                PhysicalRecordResidencyFailureReason::PhysicalPressure
            );
            let pressure = failure.pressure().expect("typed Maintenance pressure");
            assert_eq!(
                pressure.scope(),
                PhysicalOperationAllocationScope::Maintenance
            );
            assert_eq!(pressure.limit(), MAINTENANCE_LOW_BYTES);
            assert_eq!(pressure.requested(), 4 << 20);
            assert_eq!(pressure.admitted(), 0);
            assert_eq!(
                pressure.retry_posture(),
                PhysicalResidencyRetryPosture::AfterConfigurationChange
            );
            assert!(!pressure.effect_may_have_started());
            drop(blobs);

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
                Err(BlobReadOpenFailure::Layout(
                    PhysicalLayoutDenial::IndexStaleRequiresRebuild {
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

fn assert_catalog_publication(
    world: &PhysicalResidencyStoreWorld,
    scope: &AdmittedBlobScope,
    publication: PublishedBlobGeneration,
) {
    let resolved = world
        .serving()
        .blobs()
        .unwrap()
        .resolve_publication(
            publication.object().bytes(),
            publication.generation().sequence(),
            scope,
            BlobReadLimits::new(NonZeroU64::new(256).unwrap()),
        )
        .expect("selected publication must have a readable catalog binding");
    assert_eq!(resolved, publication);
}
