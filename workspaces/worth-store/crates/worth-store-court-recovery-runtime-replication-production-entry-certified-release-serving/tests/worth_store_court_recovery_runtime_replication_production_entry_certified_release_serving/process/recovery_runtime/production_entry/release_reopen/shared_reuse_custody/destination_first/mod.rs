//! Destination-first release of a reused publication: the drop invalidates the
//! directory watermark, publishes a replacement binding in the same transition,
//! and the surviving source stays readable through ordinary catalog reads
//! before and after a checkpoint and fresh C8 reopen.

#[allow(unused_imports)]
use crate::production_entry::release_reopen::shared_reuse_custody::destination_first::*;
use worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld;
use {
    worth_store::physical_runtime::BlobReclaimDeferral,
    worth_store::physical_runtime::BlobReclaimDisposition,
    worth_store::physical_runtime::BlobReclaimFailure,
    worth_store::physical_runtime::ServingPhysicalRuntime,
};

use {
    crate::production_entry::release_reopen::shared_reuse_custody::assert_absent,
    crate::production_entry::release_reopen::shared_reuse_custody::assert_retired,
    crate::production_entry::release_reopen::shared_reuse_custody::checkpoint,
    crate::production_entry::release_reopen::shared_reuse_custody::fresh_process,
    crate::production_entry::release_reopen::shared_reuse_custody::read_exact,
    crate::production_entry::release_reopen::shared_reuse_custody::recover_serving,
    crate::production_entry::release_reopen::shared_reuse_custody::release,
    crate::production_entry::release_reopen::shared_reuse_custody::release_request,
    crate::production_entry::release_reopen::shared_reuse_custody::selected_marker,
    crate::production_entry::release_reopen::shared_reuse_custody::shared_format,
    crate::production_entry::release_reopen::shared_reuse_custody::world,
    crate::production_entry::release_reopen::shared_reuse_custody::SCOPE_NAME,
    worth_store::physical_runtime::PhysicalOperationAllocationScope as Scope,
    worth_store_testkit_recovery_runtime::production_entry::blob_scope_admission::admitted_blob_scope,
};

#[test]
fn destination_first_release_keeps_the_source_routed_through_fresh_recovery_custody() {
    std::thread::Builder::new()
        .name("destination-first-c8-custody".to_owned())
        .stack_size(16 << 20)
        .spawn(run)
        .expect("recovery journey worker")
        .join()
        .expect("recovery journey worker did not panic");
}
