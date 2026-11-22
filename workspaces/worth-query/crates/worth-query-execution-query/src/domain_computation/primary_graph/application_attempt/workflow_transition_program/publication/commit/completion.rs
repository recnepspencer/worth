//! Finish projection only after the authoritative commit frame has returned.
use super::*;
use crate::domain_computation::primary_graph::PrimaryGraphHandleWorkflowExt;
use worth_query_execution_attempt::facade::ports::PrimaryGraphProviderInternalPort as _;

pub struct WorkflowAdvanceCompletion {
    pub transition_identity: String,
    pub transition_identity_locator: worth_foundational::facade::AspectFieldLocator,
    pub node_path: String,
    pub terminal: bool,
    pub assessment: Option<PreparedWorkflowAssessmentProjection>,
    pub approval: Option<PreparedWorkflowApprovalProjection>,
    pub operation_receipt_identity: Option<[u8; 32]>,
    pub progress_update: Option<crate::domain_computation::primary_graph::workflow::instance::PreparedWorkflowProgressUpdate>,
}
impl WorkflowAdvanceCompletion {
    #[inline(never)]
    pub fn finish<Schema: ApplicationSchema>(
        self,
        application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        outcome: crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitOutcome,
    ) -> WorkflowProgressOutcome {
        let Self {
            transition_identity,
            transition_identity_locator,
            node_path,
            terminal,
            assessment,
            approval,
            operation_receipt_identity,
            progress_update,
        } = self;
        let projected = match outcome.landed() {
            Ok(crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationLandedCommit::Live { receipt }) => {
                application.runtime.primary_provider
                    .graph_ref()
                    .with_runtime(execution, |runtime| {
                        project(
                            execution,
                            runtime,
                            receipt,
                            transition_identity,
                            transition_identity_locator,
                            node_path,
                            terminal,
                            assessment,
                            approval,
                            operation_receipt_identity,
                            false,
                        )
                    })
            }
            Ok(crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationLandedCommit::Recovered(receipt)) => return WorkflowProgressOutcome::RecoveredCommitted(receipt),
            Err(uncommitted) => WorkflowProgressOutcome::Application(uncommitted),
        };
        if let (Some(progress_update), WorkflowProgressOutcome::Completed(performed)) =
            (progress_update, &projected)
        {
            let committed_revision = performed.receipt().commit_reference().version_id;
            let progress_key = progress_update.key();
            let _ = application
                .runtime
                .primary_provider
                .graph_ref()
                .with_workflow_instance_progress_mut(progress_key, |retention| {
                    progress_update.apply(
                        retention,
                        committed_revision,
                        performed.transition(),
                        performed
                            .assessment_evidence()
                            .map(|evidence| evidence.evidence()),
                    )
                });
        }
        projected
    }
}
