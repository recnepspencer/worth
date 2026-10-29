pub(crate) use worth_query_worlds::facade::query::suite::advancement_custody::*;

pub(crate) mod artifact_roots;

pub(crate) mod direct_roots;

pub(crate) mod workflow_retry;

pub(crate) mod workflow_roots;

#[test]
fn workflow_start_refuses_before_any_stage_or_reader() {
    let host_request = worth_query_worlds::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let _observation = worth_query_execution::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    for placement in [Placement::Serial, Placement::Leased(NonZeroUsize::MIN)] {
        let _placement = Restore(place(placement), bound(None));
        let mut admitted_workspace =
            workflow_workspace(request_execution, "workflow-custody-control").unwrap();
        let admitted = bind(&admitted_workspace)
            .admit_workflow_resources(execution_resource_request(), &admitted_workspace)
            .unwrap();
        let before = reads();
        let control = admitted.start_workflow(&mut admitted_workspace);
        assert!(
            matches!(control, TransitionOutcome::Success(_)),
            "admitted workflow starts"
        );
        assert!(
            reads() > before,
            "admitted workflow start contacts its real source"
        );

        for zero_memory in [false, true] {
            let mut workspace =
                workflow_workspace(request_execution, "workflow-request-custody").unwrap();
            let admitted = bind(&workspace)
                .admit_workflow_resources(execution_resource_request(), &workspace)
                .unwrap();
            let _restore = refuse(zero_memory, placement);
            let before = reads();
            reports();
            let TransitionOutcome::Denied(denial) = admitted.start_workflow(&mut workspace) else {
                panic!("the root refuses before workflow preparation");
            };
            assert_eq!(
                denial.kind(),
                &domain::WorthQueryWorkflowStartDenialKind::ExecutionRequest(expected(
                    zero_memory,
                    placement
                ))
            );
            assert_eq!(denial.counters().stage_executor_contacts, 0);
            assert_eq!(reads(), before);
        }
    }
}

#[test]
fn projection_readmission_refuses_before_reading_the_pair() {
    let host_request = worth_query_worlds::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let _observation = worth_query_execution::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    for placement in [Placement::Serial, Placement::Leased(NonZeroUsize::MIN)] {
        let _placement = Restore(place(placement), bound(None));
        let mut workspace =
            controlled_workflow_workspace(request_execution, "provider-request-readmission")
                .unwrap();
        let prior = workspace.domain(GeometryDomain).unwrap();
        let (settled, _) = settle_workflow(&mut workspace);
        let live = promoted(request_execution, settled, &mut workspace);
        workspace.advance_domain_installation_generation().unwrap();
        let (_, receipt) = workspace
            .rebind_domain(prior.rebind_request())
            .unwrap()
            .into_parts();
        let (candidate, _) = settle_workflow(&mut workspace);
        let candidate = candidate.into_lifecycle();
        let witness = live.rebind_witness_for(&candidate, receipt).unwrap();

        let before = reads();
        let control = live.rebind_with(request_execution, candidate, witness, &mut workspace);
        assert!(
            reads() > before,
            "admitted projection readmission contacts its real source"
        );
        assert!(
            !matches!(
                control,
                domain::WorthQueryWorkflowProjectionRebindOutcome::Stopped(_)
            ),
            "admitted pair readmits"
        );

        use super::workflow_projection_lifecycle::{promoted, settle_workflow};
        for zero_memory in [false, true] {
            let mut workspace =
                controlled_workflow_workspace(request_execution, "provider-request-readmission")
                    .unwrap();
            let prior = workspace.domain(GeometryDomain).unwrap();
            let (settled, _) = settle_workflow(&mut workspace);
            let live = promoted(request_execution, settled, &mut workspace);
            workspace.advance_domain_installation_generation().unwrap();
            let (_, receipt) = workspace
                .rebind_domain(prior.rebind_request())
                .unwrap()
                .into_parts();
            let (candidate, _) = settle_workflow(&mut workspace);
            let candidate = candidate.into_lifecycle();
            let witness = live.rebind_witness_for(&candidate, receipt).unwrap();
            let _restore = refuse(zero_memory, placement);
            let before = reads();
            reports();
            let domain::WorthQueryWorkflowProjectionRebindOutcome::Stopped(stop) =
                live.rebind_with(request_execution, candidate, witness, &mut workspace)
            else {
                panic!("readmission cannot enter its provider under a refused root");
            };
            assert_eq!(
                stop.kind(),
                domain::WorthQueryProjectionTransitionDenialKind::ExecutionRequest(expected(
                    zero_memory,
                    placement
                ))
            );
            assert_eq!(stop.work().compatibility_readmissions(), 0);
            assert_eq!(stop.work().candidate().planning_attempts, 0);
            assert_eq!(reads(), before);
        }
    }
}
