use super::*;

pub(super) fn deny_cross_direct_owners(
    execution: &crate::domain_computation::primary_graph::TestHostRequest<'_>,

    yielded: WorthQueryYieldedDirectConvergenceIteration,
    owner_runtime: &WorthQueryExecutionRuntime,
    foreign_runtime: &WorthQueryExecutionRuntime,
    foreign_bridge: &RuntimeBridge,
) -> WorthQueryYieldedDirectConvergenceIteration {
    let active_request = execution;

    let epoch_identity = yielded.epoch_identity().to_owned();
    let graph_identity = yielded.graph_authority_identity().to_owned();
    let denied = {
        let fixture_phase_result = {
            let fixture_target = yielded;
            let installed_fixture_phase = fixture_target.fixture_phase(active_request);
            fixture_target.readmit_same_runtime(
                &installed_fixture_phase,
                foreign_runtime,
                foreign_bridge,
            )
        };
        match fixture_phase_result {
            WorthQueryDirectConvergenceReadmissionOutcome::Denied(denied) => denied,
            _ => panic!("foreign direct peer must not readmit the yielded owner"),
        }
    };
    assert_foreign_query_denial(denied.readmission_evidence());
    let yielded = denied.into_yielded();
    assert_eq!(yielded.epoch_identity(), epoch_identity);
    assert_eq!(yielded.graph_authority_identity(), graph_identity);

    let denied = {
        let fixture_phase_result = {
            let fixture_target = yielded;
            let installed_fixture_phase = fixture_target.fixture_phase(active_request);
            fixture_target.readmit_same_runtime(
                &installed_fixture_phase,
                owner_runtime,
                foreign_bridge,
            )
        };
        match fixture_phase_result {
            WorthQueryDirectConvergenceReadmissionOutcome::Denied(denied) => denied,
            _ => panic!("foreign Bridge peer must not readmit the direct yielded owner"),
        }
    };
    assert_foreign_bridge_denial(denied.readmission_evidence());
    let yielded = denied.into_yielded();
    assert_eq!(yielded.epoch_identity(), epoch_identity);
    assert_eq!(yielded.graph_authority_identity(), graph_identity);
    yielded
}

pub(super) fn deny_cross_workflow_owners(
    execution: &crate::domain_computation::primary_graph::TestHostRequest<'_>,

    yielded: WorthQueryYieldedWorkflowConvergenceIteration,
    owner_runtime: &WorthQueryExecutionRuntime,
    foreign_runtime: &WorthQueryExecutionRuntime,
    foreign_bridge: &RuntimeBridge,
) -> WorthQueryYieldedWorkflowConvergenceIteration {
    let active_request = execution;

    let epoch_identity = yielded.epoch_identity().to_owned();
    let graph_identity = yielded.graph_authority_identity().to_owned();
    let denied = {
        let fixture_phase_result = {
            let fixture_target = yielded;
            let installed_fixture_phase = fixture_target.fixture_phase(active_request);
            fixture_target.readmit_same_runtime(
                &installed_fixture_phase,
                foreign_runtime,
                foreign_bridge,
            )
        };
        match fixture_phase_result {
            WorthQueryWorkflowConvergenceReadmissionOutcome::Denied(denied) => denied,
            _ => panic!("foreign workflow peer must not readmit the yielded owner"),
        }
    };
    assert_foreign_query_denial(denied.readmission_evidence());
    let yielded = denied.into_yielded();
    assert_eq!(yielded.epoch_identity(), epoch_identity);
    assert_eq!(yielded.graph_authority_identity(), graph_identity);

    let denied = {
        let fixture_phase_result = {
            let fixture_target = yielded;
            let installed_fixture_phase = fixture_target.fixture_phase(active_request);
            fixture_target.readmit_same_runtime(
                &installed_fixture_phase,
                owner_runtime,
                foreign_bridge,
            )
        };
        match fixture_phase_result {
            WorthQueryWorkflowConvergenceReadmissionOutcome::Denied(denied) => denied,
            _ => panic!("foreign Bridge peer must not readmit the workflow yielded owner"),
        }
    };
    assert_foreign_bridge_denial(denied.readmission_evidence());
    let yielded = denied.into_yielded();
    assert_eq!(yielded.epoch_identity(), epoch_identity);
    assert_eq!(yielded.graph_authority_identity(), graph_identity);
    yielded
}
