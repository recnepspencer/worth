//! The genuine KiB64 publication boundary, before the longer C8 reuse journey.

pub(crate) use {std::num::NonZeroU64};

pub(crate) use {worth_store::physical_runtime::AdmittedBlobScope, worth_store::physical_runtime::BlobCheckpointLimit, worth_store::physical_runtime::BlobIngestDeclaration, worth_store::physical_runtime::BlobIngestFailure, worth_store::physical_runtime::BlobReadLimits, worth_store::physical_runtime::BlobReadOpenFailure, worth_store::physical_runtime::LayoutRebuildFailure, worth_store::physical_runtime::PhysicalLayoutDenial, worth_store::physical_runtime::PhysicalLayoutMaintenanceFailure, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::PhysicalOperationAllocationScope, worth_store::physical_runtime::PhysicalRecordResidencyFailureReason, worth_store::physical_runtime::PhysicalResidencyRetryPosture, worth_store::physical_runtime::PublishedBlobGeneration};
pub(crate) use {worth_store_blob_chunks::BlobChunkSize};
pub(crate) use {worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld};

pub(crate) use {worth_store_testkit_recovery_runtime::production_entry::blob_scope_admission::admitted_blob_scope, crate::production_entry::release_reopen::shared_reuse_custody::read_exact, crate::production_entry::release_reopen::shared_reuse_custody::selected_marker, crate::production_entry::release_reopen::shared_reuse_custody::shared_format, crate::production_entry::release_reopen::shared_reuse_custody::world, crate::production_entry::release_reopen::shared_reuse_custody::CHUNK};

pub(crate) const MANIFEST_CAPACITY: u16 = 512;
pub(crate) const MAINTENANCE_LOW_BYTES: u64 = 1 << 20;

pub(crate) fn assert_catalog_publication(
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
            worth_store::physical_runtime::BlobReadLimits::new(std::num::NonZeroU64::new(256).unwrap()),
        )
        .expect("selected publication must have a readable catalog binding");
    assert_eq!(resolved, publication);
}