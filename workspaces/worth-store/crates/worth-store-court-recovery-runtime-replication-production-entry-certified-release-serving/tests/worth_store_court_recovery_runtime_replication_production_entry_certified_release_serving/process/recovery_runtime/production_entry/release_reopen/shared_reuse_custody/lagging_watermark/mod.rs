//! Releasing a lagging directory watermark while a newer publication survives:
//! the replacement binding drops only the invalid watermark, keeps the newer
//! latest hint, and leaves catalog reads stale until an ordinary rebuild.

#[allow(unused_imports)] use crate::production_entry::release_reopen::shared_reuse_custody::lagging_watermark::*;
use {std::num::NonZeroU64};

use {worth_store::physical_runtime::AdmittedBlobScope, worth_store::physical_runtime::AdmittedPhysicalRecordFormat, worth_store::physical_runtime::AdmittedPhysicalRecordResidencyPolicy, worth_store::physical_runtime::BlobCheckpointLimit, worth_store::physical_runtime::BlobIngestDeclaration, worth_store::physical_runtime::BlobIngestFailure, worth_store::physical_runtime::BlobReadLimits, worth_store::physical_runtime::BlobReadOpenFailure, worth_store::physical_runtime::LayoutRebuildLimits, worth_store::physical_runtime::PhysicalLayoutDenial, worth_store::physical_runtime::PhysicalLayoutMaintenanceFailure, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::PhysicalRecordResidencyPolicy, worth_store::physical_runtime::PhysicalSpeculativeWorkKind as Speculation, worth_store::physical_runtime::PublishedBlobGeneration, worth_store::physical_runtime::ServingPhysicalRuntime};
use {worth_store_blob_chunks::BlobChunkSize};
use {worth_store_contracts::DurableArtifactFamilyId};
use {worth_store_physical_format::IndexedThroughBlobPublication};
use {worth_store_recovery_runtime::PhysicalRecoveryStaticConfiguration};
use {worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld};

use {worth_store_testkit_recovery_runtime::production_entry::blob_scope_admission::admitted_blob_scope, crate::production_entry::release_reopen::shared_reuse_custody::assert_absent, crate::production_entry::release_reopen::shared_reuse_custody::assert_retired, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving, crate::production_entry::release_reopen::shared_reuse_custody::checkpoint, crate::production_entry::release_reopen::shared_reuse_custody::fresh_process, crate::production_entry::release_reopen::shared_reuse_custody::read_exact, crate::production_entry::release_reopen::shared_reuse_custody::recover_custody, crate::production_entry::release_reopen::shared_reuse_custody::recover_serving, crate::production_entry::release_reopen::shared_reuse_custody::release, crate::production_entry::release_reopen::shared_reuse_custody::selected_marker, crate::production_entry::release_reopen::shared_reuse_custody::shared_format, crate::production_entry::release_reopen::shared_reuse_custody::world, worth_store::physical_runtime::PhysicalOperationAllocationScope as Scope, crate::production_entry::release_reopen::shared_reuse_custody::CHUNK, crate::production_entry::release_reopen::shared_reuse_custody::SCOPE_NAME};

#[test]
fn released_lagging_watermark_keeps_the_newer_latest_publication() {
    std::thread::Builder::new()
        .name("lagging-watermark-release".to_owned())
        .stack_size(16 << 20)
        .spawn(run)
        .expect("recovery journey worker")
        .join()
        .expect("recovery journey worker did not panic");
}
