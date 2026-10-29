use {worth_store_recovery_runtime::RecoveryReportOutcome};

use {crate::phase_eight_process::production::harness::ProcessWorld};

#[test]
fn shipped_recovery_process_reports_each_operation_fate() {
    let world = crate::phase_eight_process::production::harness::ProcessWorld::start("candidate-publication", 0, 1);
    let runtime = world.recover("fates");
    assert_eq!(runtime.report.outcome(), worth_store_recovery_runtime::RecoveryReportOutcome::Recovered);
    assert_eq!(runtime.indexed_fates.len() as u64, runtime.fates.total());
    assert!(runtime.fates.acknowledged > 0);
    assert_eq!(runtime.fates.durable_unacknowledged, 0);
    assert!(runtime.fates.proven_no_effect > 0);
    assert!(runtime.fates.indeterminate > 0);
}

#[test]
fn killed_durable_before_ack_process_reports_durable_unacknowledged_fate() {
    let world = crate::phase_eight_process::production::harness::ProcessWorld::start_durable_unacknowledged(17, 18);
    let runtime = world.recover("durable-unacknowledged");
    assert_eq!(runtime.report.outcome(), worth_store_recovery_runtime::RecoveryReportOutcome::Recovered);
    assert!(runtime.fates.durable_unacknowledged > 0);
    assert_eq!(runtime.fates.proven_no_effect, 1);
}
