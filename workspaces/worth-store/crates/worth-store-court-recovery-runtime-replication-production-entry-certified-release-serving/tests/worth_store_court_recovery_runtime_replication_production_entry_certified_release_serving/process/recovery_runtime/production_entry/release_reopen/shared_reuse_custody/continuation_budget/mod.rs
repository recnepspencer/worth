//! Real COW continuation backing denial, independently observed at media.

#[allow(unused_imports)] use crate::production_entry::release_reopen::shared_reuse_custody::continuation_budget::*;
use {std::cell::Cell, std::num::NonZeroU64};

use {worth_store::physical_runtime::certification::MediaOperationRole};
use {worth_store::physical_runtime::PhysicalLayoutMaintenanceFailure, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::PhysicalOperationAllocationScope as Scope, worth_store::physical_runtime::PhysicalRecordResidencyFailureReason, worth_store::physical_runtime::PhysicalResidencyRetryPosture, worth_store::physical_runtime::ServingPhysicalRuntime};
use {worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld};

use {worth_store_testkit_recovery_runtime::production_entry::blob_scope_admission::admitted_blob_scope, crate::production_entry::release_reopen::shared_reuse_custody::selected_marker, crate::production_entry::release_reopen::shared_reuse_custody::shared_format, crate::production_entry::release_reopen::shared_reuse_custody::world};

#[test]
fn continuation_backing_denies_before_cow_and_successful_twin_appends() {
    std::thread::Builder::new()
        .name("cow-continuation-backing".to_owned())
        .stack_size(16 << 20)
        .spawn(run)
        .expect("continuation worker")
        .join()
        .expect("continuation worker did not panic");
}
