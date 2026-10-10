use super::yield_fixture::YieldProvider;
use super::*;
use crate::domain_computation::{
    WorthQueryDirectGraphStepOutcome, WorthQueryDirectReadmissionOutcome,
    WorthQueryReadmissionEvidence, WorthQueryWorkflowGraphStepOutcome,
    WorthQueryWorkflowReadmissionOutcome,
};

#[test]
fn successful_direct_readmission_carries_exact_query_and_bridge_work() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;
        let active_request = execution;

        let (yielded, bridge, runtime) =
            super::readmission_direct::yielded_direct(execution, resource_request);
        let readmitted = match yielded.readmit_same_runtime(active_request, &runtime, &bridge) {
            WorthQueryDirectReadmissionOutcome::Readmitted(readmitted) => readmitted,
            _ => panic!("owner runtime must readmit the direct yielded authority"),
        };

        assert_committed_owner_work(readmitted.readmission_evidence(), 0);
        let terminal = match readmitted.into_active().abandon() {
            WorthQueryDirectGraphStepOutcome::Failed(terminal) => terminal,
            _ => panic!("readmitted direct execution must terminalize"),
        };
        terminal
            .cleanup()
            .expect("readmitted direct execution must release its authorities");
    });
}

#[test]
fn successful_workflow_readmission_carries_exact_query_and_bridge_work() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;
        let active_request = execution;

        let (yielded, bridge, runtime, old_producer) =
            super::readmission_workflow::yielded_workflow(
                execution,
                YieldProvider::installed(7),
                resource_request,
            );
        let readmitted = match yielded.readmit_same_runtime(active_request, &runtime, &bridge) {
            WorthQueryWorkflowReadmissionOutcome::Readmitted(readmitted) => readmitted,
            _ => panic!("owner runtime must readmit the workflow yielded authority"),
        };

        assert_committed_owner_work(readmitted.readmission_evidence(), 1);
        drop(old_producer);
        let terminal = match readmitted.into_active().abandon() {
            WorthQueryWorkflowGraphStepOutcome::Failed(terminal) => terminal,
            _ => panic!("readmitted workflow execution must terminalize"),
        };
        assert!(matches!(
            terminal.cleanup(),
            WorthQueryWorkflowRunCleanupOutcome::Complete(_)
        ));
    });
}

fn assert_committed_owner_work(evidence: WorthQueryReadmissionEvidence, artifact_attempts: usize) {
    let query = evidence.query_counters();
    assert_eq!(query.preflight_check_count(), 1);
    assert_eq!(query.fresh_resource_attempt_count(), 1);
    assert_eq!(query.bridge_readmission_attempt_count(), 1);
    assert_eq!(query.provider_restore_attempt_count(), 1);
    assert_eq!(query.artifact_generation_attempt_count(), artifact_attempts);
    assert_eq!(query.artifact_generation_commit_count(), artifact_attempts);
    assert_eq!(query.committed_attempt_count(), 1);

    let bridge = evidence
        .bridge_counters()
        .expect("successful readmission must carry Bridge owner evidence");
    assert_eq!(bridge.preflight_check_count(), 1);
    assert_eq!(bridge.reservation_check_count(), 1);
    assert_eq!(bridge.signal_attempt_admission_count(), 1);
    assert_eq!(bridge.signal_attempt_check_count(), 1);
    assert_eq!(bridge.signal_queue_binding_count(), 1);
    assert_eq!(bridge.abort_count(), 0);
    assert_eq!(bridge.commit_count(), 1);
}
