#[allow(unused_imports)]
use crate::sealed_runtime_lifecycle_journey::*;
use {
    std::alloc::System, std::ffi::OsString, std::path::Path, std::path::PathBuf,
    std::sync::atomic::AtomicU64, std::sync::atomic::Ordering,
};

use {
    stats_alloc::Region, stats_alloc::Stats, stats_alloc::StatsAlloc,
    stats_alloc::INSTRUMENTED_SYSTEM,
};
use {
    worth_store::physical_runtime::AdmissionError,
    worth_store::physical_runtime::CapabilityAvailability,
    worth_store::physical_runtime::DeclaredStoreRootDenialKind,
    worth_store::physical_runtime::LifecycleObservation,
    worth_store::physical_runtime::PhysicalCapability,
    worth_store::physical_runtime::PhysicalRuntimeAdmission,
    worth_store::physical_runtime::PhysicalStore,
    worth_store::physical_runtime::ProcessRuntimeCounterSnapshot,
    worth_store::physical_runtime::RootAdmissionObservation,
};

use {
    crate::lifecycle_oracle::assert_terminal_observation, crate::lifecycle_oracle::LifecycleAction,
};

#[test]
fn sealed_runtime_lifecycle_journey_establishes_the_admission_foundation() {
    let counters_before = worth_store::physical_runtime::PhysicalStore::diagnostics();
    prove_invalid_admission_is_allocation_free();
    prove_declared_root_closes_without_retaining_authority();
    prove_existing_root_is_unchanged();
    assert_process_counter_delta(
        counters_before,
        worth_store::physical_runtime::PhysicalStore::diagnostics(),
    );
}
