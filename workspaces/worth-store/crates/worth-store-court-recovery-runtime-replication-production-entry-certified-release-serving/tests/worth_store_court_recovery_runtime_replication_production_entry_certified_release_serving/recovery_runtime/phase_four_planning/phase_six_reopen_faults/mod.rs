#[allow(unused_imports)]
use crate::phase_four_planning::phase_six_reopen_faults::*;

#[test]
fn selector_scheduler_failure_retains_the_completed_read() {
    assert_reopen_scheduler_failure(
        worth_store::physical_runtime::PhysicalRecoveryFreshReopenStage::CurrentSelector,
        1,
        0,
    );
}

#[test]
fn root_scheduler_failure_retains_both_completed_reads() {
    assert_reopen_scheduler_failure(
        worth_store::physical_runtime::PhysicalRecoveryFreshReopenStage::RootManifest,
        1,
        1,
    );
}

#[test]
fn selector_signal_failure_retains_the_completed_read() {
    assert_reopen_signal_failure(
        worth_store::physical_runtime::PhysicalRecoveryFreshReopenStage::CurrentSelector,
        1,
        0,
    );
}

#[test]
fn root_signal_failure_retains_both_completed_reads() {
    assert_reopen_signal_failure(
        worth_store::physical_runtime::PhysicalRecoveryFreshReopenStage::RootManifest,
        1,
        1,
    );
}
