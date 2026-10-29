//! Shared physical payload survives a source-first release and two fresh C8 seals.

#[allow(unused_imports)]
use crate::production_entry::release_reopen::shared_reuse_custody::*;
use {std::num::NonZeroU16, std::num::NonZeroU64, std::path::Path};

use worth_store_physical_format::IndexedThroughBlobPublication;
use worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld;
use {worth_proof::AdmittedBlobReleaseProof, worth_proof::TransitionOutcome};
use {
    worth_store::physical_runtime::AdmittedBlobScope,
    worth_store::physical_runtime::AdmittedPhysicalRecordFormat,
    worth_store::physical_runtime::BlobReadFailure, worth_store::physical_runtime::BlobReadLimits,
    worth_store::physical_runtime::BlobReadOpenFailure,
    worth_store::physical_runtime::BlobReclaimDisposition,
    worth_store::physical_runtime::BlobReclaimLimits,
    worth_store::physical_runtime::BlobReclaimReceipt,
    worth_store::physical_runtime::BlobReclaimRequest,
    worth_store::physical_runtime::BlobReclaimRetirement,
    worth_store::physical_runtime::PhysicalCheckpointDeadline,
    worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey,
    worth_store::physical_runtime::PhysicalCheckpointOutcome,
    worth_store::physical_runtime::PhysicalCheckpointRequest,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store::physical_runtime::PhysicalOperationAllocationScope as Scope,
    worth_store::physical_runtime::PhysicalPageSizeClass,
    worth_store::physical_runtime::PhysicalRecordFormatDeclaration,
    worth_store::physical_runtime::PublishedBlobGeneration,
    worth_store::physical_runtime::RecordReadDenial,
    worth_store::physical_runtime::RecoveredPhysicalCheckpointCustody,
    worth_store::physical_runtime::ServingPhysicalRuntime,
};
use {
    worth_store_recovery_runtime::PhysicalRecoveryOpenRequest,
    worth_store_recovery_runtime::PhysicalRecoveryOutcome,
    worth_store_recovery_runtime::WorthStoreRecovery,
};

use {
    worth_store_testkit_recovery_runtime::production_entry::blob_scope_admission::admitted_blob_scope,
    worth_store_testkit_recovery_runtime::production_entry::certified_release_serving,
};

#[test]
fn source_first_reuse_continues_only_with_fresh_recovery_custody() {
    std::thread::Builder::new()
        .name("source-first-c8-custody".to_owned())
        .stack_size(16 << 20)
        .spawn(run)
        .expect("recovery journey worker")
        .join()
        .expect("recovery journey worker did not panic");
}

#[test]
fn fresh_child_checks_source_first_reuse() {
    crate::production_entry::release_reopen::shared_reuse_custody::fresh_process::run_child();
}
#[cfg(all())]
#[path = "continuation_budget/mod.rs"]
pub(crate) mod continuation_budget;
#[cfg(all())]
#[path = "destination_first/mod.rs"]
pub(crate) mod destination_first;
#[cfg(all())]
#[path = "lagging_watermark/mod.rs"]
pub(crate) mod lagging_watermark;
