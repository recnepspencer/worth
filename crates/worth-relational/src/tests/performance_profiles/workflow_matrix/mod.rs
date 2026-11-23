use super::*;

mod intraday_risk_branch;
mod persisted_recovery_replay;
mod retention_release_reclaim;
mod trade_correction_analysis;
mod trade_correction_audit;

#[test]
#[ignore = "counted performance profile campaign; run with -- --ignored --nocapture --test-threads=1"]
fn perf_workflow_matrix() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    trade_correction_analysis::certify_trade_correction_analysis_round_trip(execution);
    intraday_risk_branch::certify_fintech_intraday_risk_branch_round_trip(execution);
    trade_correction_audit::certify_fintech_trade_correction_audit_round_trip(execution);
    persisted_recovery_replay::certify_persisted_recovery_replay_round_trip(execution);
    retention_release_reclaim::certify_retention_release_reclaim_round_trip(execution);
}
