use super::*;

#[test]
#[ignore = "launched and forcibly terminated by the Phase 7 cleanup crash-matrix parent"]
fn killed_cleanup_process() {
    let root = required_child_root();
    let marker = required_crash_marker();
    let (schedule, gate) = paused_cleanup_schedule();
    let worker = std::thread::spawn(move || recover_with_schedule(&root, schedule));
    gate.wait_until_reached();
    std::fs::write(marker, b"cleanup effect reached before settlement").unwrap();
    std::hint::black_box(&worker);
    loop {
        std::thread::park();
    }
}

#[test]
#[ignore = "launched by the Phase 7 cleanup crash-matrix parent"]
fn indeterminate_cleanup_process() {
    let root = required_child_root();
    let handoff = recover_with_schedule(
        &root,
        cleanup_fault(worth_store::physical_runtime::certification::MediaFaultDirective::IndeterminateAfterEffect),
    );
    assert!(matches!(
        handoff.cleanup_posture(),
        worth_store_recovery_runtime::RecoveryCleanupPosture::Deferred(_)
    ));
}

#[test]
#[ignore = "launched by the Phase 7 cleanup crash-matrix parent"]
fn reopen_after_cleanup_process() {
    let root = required_child_root();
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) =
        reopen_with_schedule(&root, empty_fault_schedule()).finish()
    else {
        panic!("ambiguous cleanup debt cannot invalidate a fresh recovery")
    };
    assert!(!handoff
        .cleanup_posture()
        .evidence()
        .dispositions()
        .is_empty());
}
use worth_store::physical_runtime::certification::MediaFaultDirective;
use {
    worth_store::physical_runtime::PhysicalRecoveryCleanupAdmissionDenialKind,
    worth_store::physical_runtime::PhysicalRecoveryCleanupCommandStage,
    worth_store::physical_runtime::PhysicalRecoveryCleanupFreshnessReadDenialKind,
    worth_store::physical_runtime::PhysicalRecoveryCleanupRemovalDenialKind,
    worth_store::physical_runtime::PhysicalRecoveryCleanupRemovalIndeterminate,
    worth_store::physical_runtime::PhysicalSignalSettlementOutcome,
    worth_store::physical_runtime::PhysicalWorkSchedulerPosture,
    worth_store::physical_runtime::StoreRecoveryCleanupFreshnessDenial,
};
use {
    worth_store_recovery_runtime::PhysicalRecoveryOutcome,
    worth_store_recovery_runtime::RecoveryCleanupDeferralEvidence,
    worth_store_recovery_runtime::RecoveryCleanupDispositionKind,
    worth_store_recovery_runtime::RecoveryCleanupPosture,
};
