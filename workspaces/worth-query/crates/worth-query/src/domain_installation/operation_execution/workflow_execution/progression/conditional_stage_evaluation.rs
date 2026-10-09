use crate::basis_lifecycle::BasisOperationLane;

pub(super) enum ConditionalStageStop {
    Deferred(Vec<crate::domain_installation::WorthQueryConditionalProvenance>),
    Denied(super::WorthQueryWorkflowAdvanceDenialKind),
}

pub(super) fn evaluate<D: 'static, O: 'static, F: 'static, L: BasisOperationLane>(
    execution: worth_execution::ExecutionRequest<'_, '_>,
    bound: &crate::domain_installation::WorthQueryBoundDomainOperation<D, O, F, L>,
    workspace: &mut crate::runtime::WorthQueryWorkspace,
    snapshot: &crate::memory_workspace::WorthQuerySnapshotIdentity,
    stage_identity: &str,
    run_identity: &str,
    attempt: u64,
    resources: &super::WorthQueryAdmittedExecutionResourcePlan,
    resource_evidence: &super::WorthQueryExecutionResourceAttemptEvidence,
    run_counters: &mut super::WorthQueryWorkflowRunCounters,
) -> Result<Vec<crate::domain_installation::WorthQueryConditionalProvenance>, ConditionalStageStop>
{
    let execution_identity = format!(
        "{}:{}:{}",
        bound.binding_identity(),
        run_identity,
        stage_identity
    );
    let mut counters = crate::domain_installation::WorthQueryOperationExecutionCounters::default();
    let outcome = crate::domain_installation::evaluate_bound_conditionals(
        bound,
        crate::domain_installation::WorthQueryConditionalEvaluationPass {
            execution,
            workspace,
            snapshot,
            execution_identity: &execution_identity,
            scope: crate::domain_installation::WorthQueryConditionalEvaluationScope::WorkflowStage(
                stage_identity,
            ),
            workflow_run_identity: Some(run_identity),
            attempt,
            resources,
            resource_evidence,
            counters: &mut counters,
        },
    );
    super::workflow_conditional_counters::add_conditional_counters(run_counters, counters);
    match outcome {
        Ok(conditional) => Ok(conditional),
        Err(crate::domain_installation::WorthQueryConditionalEvaluationStop::Deferred(
            conditional,
        )) => Err(ConditionalStageStop::Deferred(conditional)),
        Err(crate::domain_installation::WorthQueryConditionalEvaluationStop::Failed {
            kind,
            ..
        }) => Err(ConditionalStageStop::Denied(
            super::WorthQueryWorkflowAdvanceDenialKind::ConditionalExecution(kind),
        )),
        Err(crate::domain_installation::WorthQueryConditionalEvaluationStop::Reentry(denial)) => {
            Err(ConditionalStageStop::Denied(
                super::WorthQueryWorkflowAdvanceDenialKind::ConditionalReentry(denial),
            ))
        }
    }
}
