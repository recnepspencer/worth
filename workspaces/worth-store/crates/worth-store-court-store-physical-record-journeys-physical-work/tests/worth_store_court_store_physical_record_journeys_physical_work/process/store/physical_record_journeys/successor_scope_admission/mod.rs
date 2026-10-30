#[allow(unused_imports)]
use crate::physical_record_journeys::successor_scope_admission::*;
use {std::num::NonZeroU32, std::num::NonZeroU64};

use worth_store_physical_format::store_namespace::StableStoreIdentity;
use {
    worth_store::physical_runtime::PhysicalOperationAllocationScope as Scope,
    worth_store::physical_runtime::PhysicalRecordInitialization,
    worth_store::physical_runtime::PhysicalRecordPressureEvidence,
    worth_store::physical_runtime::PhysicalRecordResidencyPolicy,
    worth_store::physical_runtime::PhysicalResidencyDimension,
    worth_store::physical_runtime::PhysicalSpeculativeWorkKind as Speculation,
    worth_store::physical_runtime::ServingPhysicalRuntime,
};

use {
    worth_store_testkit_store::physical_record_journeys::configuration,
    worth_store_testkit_store::physical_record_journeys::media,
    worth_store_testkit_store::physical_record_journeys::success,
};

#[test]
fn successor_scopes_are_exact_isolated_global_and_released() {
    let root = tempfile::tempdir().unwrap();
    let serving = successor_scope_runtime(root.path());
    assert_exact_scope_ceilings(&serving);
    assert_scope_isolation(&serving);
    assert_global_envelope_and_release(&serving);
}
