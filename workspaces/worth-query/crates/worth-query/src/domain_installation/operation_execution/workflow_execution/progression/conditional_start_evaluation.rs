use crate::basis_lifecycle::BasisOperationLane;

pub(super) enum ConditionalWorkflowStartStop {
    Deferred(Vec<crate::domain_installation::WorthQueryConditionalProvenance>),
    Denied(super::WorthQueryWorkflowStartDenialKind),
}

pub(super) struct ConditionalWorkflowStartEvaluationPass<'a> {
    pub(super) execution: worth_execution::ExecutionRequest<'a, 'a>,
    pub(super) workspace: &'a mut crate::runtime::WorthQueryWorkspace,
    pub(super) snapshot: &'a crate::memory_workspace::WorthQuerySnapshotIdentity,
    pub(super) run_identity: &'a str,
    pub(super) attempt: u64,
    pub(super) resources: &'a super::WorthQueryAdmittedExecutionResourcePlan,
    pub(super) resource_evidence: &'a super::WorthQueryExecutionResourceAttemptEvidence,
    pub(super) run_counters: &'a mut super::WorthQueryWorkflowRunCounters,
}

pub(super) fn evaluate<D: 'static, O: 'static, F: 'static, L: BasisOperationLane>(
    bound: &crate::domain_installation::WorthQueryBoundDomainOperation<D, O, F, L>,
    pass: ConditionalWorkflowStartEvaluationPass<'_>,
) -> Result<
    Vec<crate::domain_installation::WorthQueryConditionalProvenance>,
    ConditionalWorkflowStartStop,
> {
    let ConditionalWorkflowStartEvaluationPass {
        execution,
        workspace,
        snapshot,
        run_identity,
        attempt,
        resources,
        resource_evidence,
        run_counters,
    } = pass;
    let execution_identity = format!("{}:{run_identity}:operation", bound.binding_identity());
    let mut counters = crate::domain_installation::WorthQueryOperationExecutionCounters::default();
    let outcome = crate::domain_installation::evaluate_bound_conditionals(
        bound,
        crate::domain_installation::WorthQueryConditionalEvaluationPass {
            execution,
            workspace,
            snapshot,
            execution_identity: &execution_identity,
            scope: crate::domain_installation::WorthQueryConditionalEvaluationScope::Operation,
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
        )) => Err(ConditionalWorkflowStartStop::Deferred(conditional)),
        Err(crate::domain_installation::WorthQueryConditionalEvaluationStop::Failed {
            kind,
            ..
        }) => Err(ConditionalWorkflowStartStop::Denied(
            super::WorthQueryWorkflowStartDenialKind::ConditionalExecution(kind),
        )),
        Err(crate::domain_installation::WorthQueryConditionalEvaluationStop::Reentry(denial)) => {
            Err(ConditionalWorkflowStartStop::Denied(
                super::WorthQueryWorkflowStartDenialKind::ConditionalReentry(denial),
            ))
        }
    }
}
