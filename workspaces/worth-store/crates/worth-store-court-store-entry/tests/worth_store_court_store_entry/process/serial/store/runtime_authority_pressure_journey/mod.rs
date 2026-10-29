#[allow(unused_imports)]
use crate::runtime_authority_pressure_journey::*;
use {
    std::ffi::OsString, std::path::Path, std::path::PathBuf, std::sync::atomic::AtomicU64,
    std::sync::atomic::Ordering, std::sync::Arc, std::sync::Barrier,
};

use {
    worth_store::physical_runtime::AdmissionError,
    worth_store::physical_runtime::PhysicalRuntimeAdmission,
    worth_store::physical_runtime::PhysicalStore,
};

use crate::pressure_outcome::PressureOutcome;
use {
    crate::lifecycle_oracle::assert_terminal_observation, crate::lifecycle_oracle::LifecycleAction,
};

#[test]
fn runtime_authority_pressure_journey_keeps_observation_read_only_and_phase_scoped() {
    if crate::process_death_probe::run_child_mode() {
        return;
    }

    let first = run_pressure_world();
    let replay = run_pressure_world();
    assert_eq!(replay, first);
}
