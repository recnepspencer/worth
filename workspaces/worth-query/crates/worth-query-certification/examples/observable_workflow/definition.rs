//! One native graph contrasts occurrence-only capture with approved archival.
use crate::document_retention_model::{
    retention_entry::OccurrenceSetRetentionBinding,
    schema::*,
    workflow::{
        self, governed_read::*, WorkflowApprovalBinding, WorkflowApprovalCapability,
        WorkflowDefinitionAuthoringOperation,
    },
};
use worth_query_decl::facade::{
    application_capability::ApplicationWorkflowRecordedActorSource, application_program::*,
};

pub struct MixedWorkflow;
impl ApplicationWorkflowSpec for MixedWorkflow {
    type Schema = DocumentRetentionSchema;
    const IDENTITY: ApplicationWorkflowSpecIdentity =
        ApplicationWorkflowSpecIdentity::new("neutral.observable.workflow.v1");
    fn observation_policy() -> ApplicationWorkflowObservationPolicy {
        ApplicationWorkflowObservationPolicy::governed(
            ApplicationWorkflowReadRule::subject::<Self, SubjectRead>(),
            ApplicationWorkflowReadRule::definition::<Self, DefinitionRead>(),
            Some(ApplicationWorkflowReadRule::definition_metadata::<
                Self,
                MetadataRead,
            >()),
            ApplicationWorkflowAuditPolicy::Governed(ApplicationWorkflowReadRule::subject::<
                Self,
                AuditRead,
            >()),
        )
    }
    fn actor_policy() -> ApplicationWorkflowActorPolicy {
        ApplicationWorkflowActorPolicy::for_approval::<Self, WorkflowApprovalBinding>(
            ApplicationWorkflowRecordedActorSource::BoundProposal,
        )
    }
}
pub fn definition() -> ValidatedWorkflowDefinition<MixedWorkflow> {
    let limits = workflow::definition_limits()
        .with_total_deadline(std::time::Duration::from_secs(300))
        .unwrap();
    let mut graph = ApplicationWorkflowDefinitionBuilder::<MixedWorkflow>::new(
        "observable-document-archive",
        limits,
    )
    .unwrap();
    let proposal = graph
        .operation::<WorkflowDefinitionAuthoringOperation>("propose", false)
        .unwrap();
    let capture = graph
        .operation_binding::<OccurrenceSetRetentionBinding>("capture")
        .unwrap();
    let condition = graph
        .condition(
            "positive-retention",
            "retained",
            ApplicationWorkflowConditionOperands::new()
                .query::<DocumentRetentionConditionQuery>("retained"),
        )
        .unwrap();
    let approval = graph
        .approval::<WorkflowApprovalCapability>("approval")
        .unwrap();
    let archive = graph
        .operation_binding::<crate::effect::ArchiveBinding>("archive")
        .unwrap();
    let inbound = graph
        .await_inbound::<crate::effect::ArchiveEffect>(
            "archive-completion",
            &archive,
            crate::effect::inbound(),
            ApplicationWorkflowInboundWait::UntilInstanceDeadline,
        )
        .unwrap();
    let completed = graph.terminal("archived").unwrap();
    let rejected = graph.terminal("rejected").unwrap();
    graph
        .start(&proposal)
        .control(
            &proposal,
            ApplicationWorkflowControlOutcome::Completed,
            &capture,
        )
        .control(
            &capture,
            ApplicationWorkflowControlOutcome::Completed,
            &condition,
        )
        .control(
            &condition,
            ApplicationWorkflowControlOutcome::ConditionSatisfied,
            &approval,
        )
        .control(
            &condition,
            ApplicationWorkflowControlOutcome::ConditionUnsatisfied,
            &rejected,
        )
        .control(
            &approval,
            ApplicationWorkflowControlOutcome::Approved,
            &archive,
        )
        .control(
            &approval,
            ApplicationWorkflowControlOutcome::Rejected,
            &rejected,
        )
        .control(
            &archive,
            ApplicationWorkflowControlOutcome::Completed,
            &inbound,
        )
        .control(
            &inbound,
            ApplicationWorkflowControlOutcome::Completed,
            &completed,
        )
        .condition_subject(&proposal, &condition)
        .proposal_for_approval(&proposal, &approval)
        .approval_authority(&approval, &archive)
        .operation_input(&proposal, &capture)
        .operation_input(&proposal, &archive);
    graph
        .finish()
        .unwrap()
        .validate()
        .expect("the complete mixed graph validates")
}
