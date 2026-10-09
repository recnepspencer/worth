//! A retained deferred start opens once before its real condition reader.
use super::*;
#[test]
fn deferred_start_retry_opens_before_the_same_condition_reader() {
    super::super::advancement_custody::verify(
        "deferred workflow retry",
        || {
            let node = domain_condition_node("deferred-start-retry-custody");
            let mut installation = conditional_installation(&node);
            installation.providers =
                worth_runtime_bridge::facade::BridgeConditionalProviderSet::new().condition(
                    StaticCondition(
                        worth_signal::facade::InstalledSignalConditionDecision::Deferred,
                    ),
                );
            let mut workspace = operation_conditional_workflow_workspace_with(
                "deferred-start-retry-custody",
                node,
                installation,
                CountedWorkflowCompute(Arc::new(std::sync::atomic::AtomicUsize::new(0))),
            )
            .unwrap();
            let installed = workspace.domain(GeometryDomain).unwrap();
            let bound = workspace
                .observe_operating_world(workspace.current_world())
                .unwrap()
                .family(ReadFamily)
                .bind(&installed, WorkflowRead)
                .unwrap();
            let TransitionOutcome::Deferred(deferred) = bound
                .admit_workflow_resources(
                    crate::suite::installed_operation_fixture::execution_resource_request(),
                    &workspace,
                )
                .unwrap()
                .start_workflow(&mut workspace)
            else {
                panic!("the declared condition produces a retained deferred start");
            };
            (workspace, deferred)
        },
        |deferred, workspace| match deferred.retry(workspace) {
            TransitionOutcome::Deferred(_) => None,
            TransitionOutcome::Denied(denial) => match denial.kind() {
                domain::WorthQueryWorkflowStartDenialKind::ExecutionRequest(cause) => Some(*cause),
                other => panic!("unexpected retry denial: {other:?}"),
            },
            _ => panic!("the same admitted condition remains deferred"),
        },
    );
}
