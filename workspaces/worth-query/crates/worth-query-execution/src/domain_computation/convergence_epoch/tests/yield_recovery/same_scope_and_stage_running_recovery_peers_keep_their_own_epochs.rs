//! Same scope and stage running recovery peers keep their own epochs.

use super::*;

#[test]
fn same_scope_and_stage_running_recovery_peers_keep_their_own_epochs() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let (direct_a_identity, direct_a) = direct_paused(
            execution,
            direct_admission_fixture(FixtureDisposition::YieldThenConverged, resource_request),
            "shared-yield-recovery-scope",
        );
        let (direct_b_identity, direct_b) = direct_paused(
            execution,
            direct_admission_fixture(FixtureDisposition::YieldThenConverged, resource_request),
            "shared-yield-recovery-scope",
        );
        assert_ne!(direct_a_identity, direct_b_identity);
        let direct_a = resume_direct(foreign_direct_running_recovery(direct_a));
        let direct_b = resume_direct(foreign_direct_running_recovery(direct_b));
        let direct_a = match direct_a.yield_iteration() {
            WorthQueryDirectConvergenceYieldOutcome::Yielded(yielded) => yielded,
            _ => panic!("rightful direct peer A did not yield"),
        };
        let direct_b = match direct_b.yield_iteration() {
            WorthQueryDirectConvergenceYieldOutcome::Yielded(yielded) => yielded,
            _ => panic!("rightful direct peer B did not yield"),
        };
        assert_eq!(direct_a.epoch_identity(), direct_a_identity);
        assert_eq!(direct_b.epoch_identity(), direct_b_identity);
        cleanup_direct_peer(direct_a, &direct_a_identity);
        cleanup_direct_peer(direct_b, &direct_b_identity);

        let (workflow_a_identity, workflow_a) = workflow_paused(
            execution,
            workflow_admission_fixture(FixtureDisposition::YieldThenConverged, resource_request),
            "shared-yield-recovery-stage-scope",
        );
        let (workflow_b_identity, workflow_b) = workflow_paused(
            execution,
            workflow_admission_fixture(FixtureDisposition::YieldThenConverged, resource_request),
            "shared-yield-recovery-stage-scope",
        );
        assert_ne!(workflow_a_identity, workflow_b_identity);
        let workflow_a = resume_workflow(foreign_workflow_running_recovery(workflow_a));
        let workflow_b = resume_workflow(foreign_workflow_running_recovery(workflow_b));
        let workflow_a = match workflow_a.yield_iteration() {
            WorthQueryWorkflowConvergenceYieldOutcome::Yielded(yielded) => yielded,
            _ => panic!("rightful workflow peer A did not yield"),
        };
        let workflow_b = match workflow_b.yield_iteration() {
            WorthQueryWorkflowConvergenceYieldOutcome::Yielded(yielded) => yielded,
            _ => panic!("rightful workflow peer B did not yield"),
        };
        assert_eq!(workflow_a.epoch_identity(), workflow_a_identity);
        assert_eq!(workflow_b.epoch_identity(), workflow_b_identity);
        cleanup_workflow_peer(workflow_a, &workflow_a_identity);
        cleanup_workflow_peer(workflow_b, &workflow_b_identity);
    });
}
