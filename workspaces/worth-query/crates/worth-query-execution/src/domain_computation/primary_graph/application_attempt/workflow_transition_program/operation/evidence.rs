//! Resolve additional evidence without confusing its absence with unavailable
//! approval authority. Common occurrence binding remains the operation owner's job.

use super::{
    check_request_live, mismatch, observe_retained_transition, ApplicationOperationMarkerIdentity,
    ApplicationSchema, CompiledWorkflowDefinition, CompiledWorkflowNodeKind,
    WorkflowInstanceProgress, WorthQueryApplicationAttemptDenial,
    WorthQueryApplicationAttemptDenialKind, WorthQueryApplicationObservedFact,
    WorthQueryCompleteApplicationReadSet, WorthQueryProjectedApplicationMutation,
};
use crate::domain_computation::authorization::WorthQueryWorkflowApprovalAuthorityBasis;
use crate::domain_computation::primary_graph::workflow::instance::{
    SelectedWorkflowOperation, SelectedWorkflowTransition,
};

pub(super) enum WorkflowOperationEvidence {
    Satisfied(Option<WorthQueryWorkflowApprovalAuthorityBasis>),
    Unavailable,
}

impl<Schema, Operation, Input, Scope>
    WorthQueryCompleteApplicationReadSet<
        Schema,
        Operation,
        Input,
        Scope,
        WorthQueryProjectedApplicationMutation,
    >
where
    Schema: ApplicationSchema,
    Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
{
    /// Work is bounded by the selected node's declared evidence, not history or
    /// unrelated workflow instances. The compiled source lookup is indexed.
    pub(super) fn resolve_operation_evidence(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        compiled: &CompiledWorkflowDefinition,
        layout: &crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout,
        instance: &super::super::super::PublishedWorkflowInstanceRef,
        selected: &SelectedWorkflowTransition,
        progress: &WorkflowInstanceProgress,
        operation: &SelectedWorkflowOperation,
        facts: &mut Vec<WorthQueryApplicationObservedFact>,
    ) -> Result<WorkflowOperationEvidence, WorthQueryApplicationAttemptDenial> {
        let mut sources = compiled.approval_authority_sources(selected.node());
        let Some(approval) = sources.next() else {
            return if operation
                .workflow_execution_requirements
                .as_ref()
                .is_some_and(|requirements| requirements.approval().is_none())
            {
                Ok(WorkflowOperationEvidence::Satisfied(None))
            } else {
                Err(mismatch(selected.node_path()))
            };
        };
        let CompiledWorkflowNodeKind::Approval {
            capability,
            capability_type,
            ..
        } = approval.kind()
        else {
            return Err(mismatch(selected.node_path()));
        };
        if sources.next().is_some()
            || operation
                .workflow_execution_requirements
                .as_ref()
                .and_then(|requirements| requirements.approval())
                .is_some_and(|(required_capability, required_type)| {
                    required_capability != capability || required_type != capability_type
                })
        {
            return Err(mismatch(selected.node_path()));
        }
        let transition = progress
            .latest_transition(approval.entity())
            .ok_or_else(|| mismatch(selected.node_path()))?;
        let settlement = self.lease.handle().with_runtime(execution, |runtime| {
            observe_retained_transition(
                execution,
                runtime,
                self.lease.snapshot(),
                layout,
                transition,
                facts,
            )
        })?;
        if settlement.outcome() != worth_query_declaration::facade::application_program::ApplicationWorkflowControlOutcome::Approved {
            return Err(mismatch(selected.node_path()));
        }
        check_request_live(
            self.admission.publication_request(),
            self.admission.operation(),
        )?;
        match super::super::approval_decision::observe_operation_approval_inputs(
            execution, compiled, layout, &self.lease.layout, self.admission.scope_entity_id(),
            instance, progress, approval.entity(), &operation.operation, self.lease.handle(),
            self.lease.snapshot(), self.admission.publication_request(),
        ) {
            Ok((mut observed, authority)) => {
                facts.append(&mut observed);
                Ok(WorkflowOperationEvidence::Satisfied(Some(authority)))
            }
            Err(denial) if matches!(denial.kind(),
                WorthQueryApplicationAttemptDenialKind::WorkflowAssessmentEvidenceMismatch
                    | WorthQueryApplicationAttemptDenialKind::WorkflowAssessmentEvidenceIncomplete) => {
                Ok(WorkflowOperationEvidence::Unavailable)
            }
            Err(denial) => Err(denial),
        }
    }
}
