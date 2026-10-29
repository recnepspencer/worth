pub(crate) use worth_query_worlds::facade::query::suite::advancement_custody::workflow_roots::*;

#[test]
fn workflow_stage_advance_opens_before_its_reader() {
    let host_request = worth_query_worlds::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    verify(
        "workflow advance",
        || run_setup(request_execution),
        |run, workspace| match run.advance(
            "start",
            domain::WorthQueryWorkflowValue::NotRequired,
            workspace,
        ) {
            TransitionOutcome::Success(_) => None,
            TransitionOutcome::Denied(denial) => match denial.kind() {
                domain::WorthQueryWorkflowAdvanceDenialKind::ExecutionRequest(cause) => {
                    Some(*cause)
                }
                other => panic!("unexpected advance denial: {other:?}"),
            },
            _ => panic!("unexpected advance posture"),
        },
    );
}

#[test]
fn workflow_parallel_frontier_opens_before_its_reader() {
    let host_request = worth_query_worlds::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    verify(
        "workflow parallel frontier",
        || {
            let (mut workspace, run) = run_setup(request_execution);
            let run = run
                .advance(
                    "start",
                    domain::WorthQueryWorkflowValue::NotRequired,
                    &mut workspace,
                )
                .unwrap();
            (workspace, run)
        },
        |run, workspace| {
            let stages = ["left", "right"].map(|stage| {
                (
                    stage.to_owned(),
                    domain::WorthQueryWorkflowValue::Text("start".into()),
                )
            });
            match run.advance_admitted_frontier(stages, workspace) {
                TransitionOutcome::Success(_) => None,
                TransitionOutcome::Denied(denial) => match denial.kind() {
                    domain::WorthQueryWorkflowAdvanceDenialKind::ExecutionRequest(cause) => {
                        Some(*cause)
                    }
                    other => panic!("unexpected frontier denial: {other:?}"),
                },
                _ => panic!("unexpected frontier posture"),
            }
        },
    );
}

#[test]
fn workflow_projection_promotion_opens_before_its_reader() {
    let host_request = worth_query_worlds::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    verify(
        "workflow projection promotion",
        || {
            let mut workspace =
                workflow_workspace(request_execution, "workflow-promotion-custody").unwrap();
            let (settled, _) = settle_workflow(&mut workspace);
            (workspace, settled.into_lifecycle())
        },
        |current, workspace| match current.promote(request_execution, workspace) {
            domain::WorthQueryWorkflowProjectionPromotionOutcome::Promoted(_) => None,
            domain::WorthQueryWorkflowProjectionPromotionOutcome::Denied(stop) => match stop.kind()
            {
                domain::WorthQueryProjectionPromotionDenialKind::ExecutionRequest(cause) => {
                    Some(cause)
                }
                other => panic!("unexpected promotion denial: {other:?}"),
            },
            _ => panic!("unexpected promotion posture"),
        },
    );
}

#[test]
fn workflow_projection_replacement_opens_before_its_reader() {
    let host_request = worth_query_worlds::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    verify(
        "workflow projection replacement",
        || {
            let mut workspace =
                workflow_workspace(request_execution, "workflow-replacement-custody").unwrap();
            let (settled, _) = settle_workflow(&mut workspace);
            let live = promoted(request_execution, settled, &mut workspace);
            let (candidate, _) = settle_workflow(&mut workspace);
            let candidate = candidate.into_lifecycle();
            let witness = live.replacement_witness_for(&candidate).unwrap();
            (workspace, (live, candidate, witness))
        },
        |(live, candidate, witness), workspace| match live.replace_with(
            request_execution,
            candidate,
            witness,
            workspace,
        ) {
            domain::WorthQueryWorkflowProjectionReplacementOutcome::Replaced(_) => None,
            domain::WorthQueryWorkflowProjectionReplacementOutcome::Stopped(stop) => {
                match stop.kind() {
                    domain::WorthQueryProjectionTransitionDenialKind::ExecutionRequest(cause) => {
                        Some(cause)
                    }
                    other => panic!("unexpected replacement denial: {other:?}"),
                }
            }
            _ => panic!("unexpected replacement posture"),
        },
    );
}
