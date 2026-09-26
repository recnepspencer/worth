use worth_query_host::facade::{
    application_entry::{
        PublishedWorkflowDefinitionRef, PublishedWorkflowProposalRef, RequiredWorkflowApproval,
        WorkflowApprovalDecision, WorkflowDefinitionExpectedPredecessor,
        WorkflowDefinitionPublicationOutcome, WorkflowDefinitionRetirementOutcome,
        WorkflowInstanceStartOutcome, WorkflowProgressOutcome, WorkflowProposalOutcome,
        WorthQueryApplicationRequestExt, WorthQueryWorkflowAdvancePreparationDenial,
        WorthQueryWorkflowDefinitionPublicationPreparationDenial,
        WorthQueryWorkflowDefinitionRetirementPreparationDenial,
        WorthQueryWorkflowInstanceStartPreparationDenial,
        WorthQueryWorkflowProposalPreparationDenial,
    },
    declaration::application_program::{
        ApplicationWorkflowComponentLimits, ApplicationWorkflowControlOutcome,
        ApplicationWorkflowDefinitionBuilder, ApplicationWorkflowDefinitionLimits,
        AuthoredWorkflowDefinition, ValidatedWorkflowDefinition,
    },
};
use worth_query_installation::facade::WorthQueryInstalledWorkflowDefinitionContract;

use super::super::{
    dimension_entry::{ReviewedSetPartDimensionBinding, PART_IDENTITY},
    host::BoundedDimensionWorkflowRuntime,
    operator_identity::{authenticate_operator, block_on, request_scope},
    programs::DimensionProgramP0,
    schema::{BoundedDimensionSchema, PartDimensionConditionQuery, PartDimensionQuery},
};
use super::{
    ReviewedGeometryWorkflow, WorkflowAdvanceInput, WorkflowAdvanceIntent,
    WorkflowApprovalCapability, WorkflowApprovalIntent, WorkflowDefinitionAuthoringInput,
    WorkflowDefinitionAuthoringIntent, WorkflowDefinitionAuthoringOperation,
    WorkflowInstanceStartInput, WorkflowInstanceStartIntent,
};

#[path = "definition/instance.rs"]
mod instance;
pub use instance::{
    advance_instance, approve_instance, migrate_instance, propose_authoring_instance,
    propose_authoring_instance_with_dimension, propose_instance, propose_instance_on_branch,
    start_instance,
};

pub fn reviewed_geometry_definition(
    completion_identity: &str,
) -> ValidatedWorkflowDefinition<ReviewedGeometryWorkflow> {
    reviewed_geometry_definition_with_join_policy(
        completion_identity,
        worth_query_host::facade::declaration::application_program::ApplicationWorkflowEvidenceJoinPolicy::AllRequiredPassing,
    )
}

pub fn reviewed_geometry_definition_with_join_policy(
    completion_identity: &str,
    join_policy: worth_query_host::facade::declaration::application_program::ApplicationWorkflowEvidenceJoinPolicy,
) -> ValidatedWorkflowDefinition<ReviewedGeometryWorkflow> {
    reviewed_geometry_definition_with_policy(completion_identity, join_policy, Rejection::Terminal)
}

/// A rejected approval loops back to a fresh proposal.
pub fn reproposing_geometry_definition(
    completion_identity: &str,
) -> ValidatedWorkflowDefinition<ReviewedGeometryWorkflow> {
    reviewed_geometry_definition_with_policy(
        completion_identity,
        worth_query_host::facade::declaration::application_program::ApplicationWorkflowEvidenceJoinPolicy::AllRequiredPassing,
        Rejection::Repropose,
    )
}

enum Rejection {
    Terminal,
    Retry(u16),
    Repropose,
}

pub fn approval_retry_definition() -> ValidatedWorkflowDefinition<ReviewedGeometryWorkflow> {
    reviewed_geometry_definition_with_policy(
        "applied",
        worth_query_host::facade::declaration::application_program::ApplicationWorkflowEvidenceJoinPolicy::AllRequiredPassing,
        Rejection::Retry(2),
    )
}

fn reviewed_geometry_definition_with_policy(
    completion_identity: &str,
    join_policy: worth_query_host::facade::declaration::application_program::ApplicationWorkflowEvidenceJoinPolicy,
    rejection: Rejection,
) -> ValidatedWorkflowDefinition<ReviewedGeometryWorkflow> {
    let mut builder = ApplicationWorkflowDefinitionBuilder::<ReviewedGeometryWorkflow>::new(
        "reviewed-geometry",
        definition_limits(),
    )
    .expect("the workflow identity is valid");
    let propose = builder
        .operation::<WorkflowDefinitionAuthoringOperation>("propose", false)
        .expect("the proposal node is valid");
    let structural = builder
        .assessment::<PartDimensionQuery>("checks/structural")
        .expect("the structural assessment node is valid");
    let manufacturability = builder
        .assessment::<PartDimensionQuery>("checks/manufacturability")
        .expect("the manufacturability assessment node is valid");
    let evidence = builder
        .evidence_join("checks/evidence", join_policy)
        .expect("the evidence node is valid");
    let approval = builder
        .approval::<WorkflowApprovalCapability>("approval")
        .expect("the approval node is valid");
    let apply = builder
        .operation_binding::<ReviewedSetPartDimensionBinding>("apply")
        .expect("the guarded operation node is valid");
    let completed = builder
        .terminal(completion_identity)
        .expect("the completion node is valid");
    let rejected = builder
        .terminal("rejected")
        .expect("the rejection node is valid");
    builder
        .start(&propose)
        .control(
            &propose,
            ApplicationWorkflowControlOutcome::Completed,
            &structural,
        )
        .control(
            &structural,
            ApplicationWorkflowControlOutcome::Completed,
            &manufacturability,
        )
        .control(
            &manufacturability,
            ApplicationWorkflowControlOutcome::Completed,
            &evidence,
        )
        .control(
            &evidence,
            ApplicationWorkflowControlOutcome::EvidenceSatisfied,
            &approval,
        )
        .control(
            &evidence,
            ApplicationWorkflowControlOutcome::EvidenceFailed,
            &rejected,
        )
        .control(
            &approval,
            ApplicationWorkflowControlOutcome::Approved,
            &apply,
        )
        .control(
            &apply,
            ApplicationWorkflowControlOutcome::Completed,
            &completed,
        )
        .proposal_for_assessment(&propose, &structural)
        .proposal_for_assessment(&propose, &manufacturability)
        .assessment_evidence(&structural, &evidence)
        .assessment_evidence(&manufacturability, &evidence)
        .proposal_for_approval(&propose, &approval)
        .joined_evidence(&evidence, &approval)
        .approval_authority(&approval, &apply)
        .operation_input(&propose, &apply);
    match rejection {
        Rejection::Retry(bound) => {
            builder.retry(
            &approval,
            worth_query_host::facade::declaration::application_program::ApplicationWorkflowRetry::new(
                ApplicationWorkflowControlOutcome::Rejected, "reconsider", bound,
            ).expect("the reconsideration bound is valid"),
            &approval,
        ).control(&approval, ApplicationWorkflowControlOutcome::RetryExhausted, &rejected);
        }
        Rejection::Terminal => {
            builder.control(
                &approval,
                ApplicationWorkflowControlOutcome::Rejected,
                &rejected,
            );
        }
        Rejection::Repropose => {
            builder.retry(
                &approval,
                worth_query_host::facade::declaration::application_program::ApplicationWorkflowRetry::new(
                    ApplicationWorkflowControlOutcome::Rejected, "repropose", 2,
                ).expect("the reproposal bound is valid"),
                &propose,
            ).control(&approval, ApplicationWorkflowControlOutcome::RetryExhausted, &rejected);
        }
    }
    builder
        .finish()
        .expect("the authored definition is complete")
        .validate()
        .expect("the reviewed-geometry definition is valid")
}

pub fn terminal_definition(
    terminal_identity: &str,
) -> ValidatedWorkflowDefinition<ReviewedGeometryWorkflow> {
    let mut builder = ApplicationWorkflowDefinitionBuilder::<ReviewedGeometryWorkflow>::new(
        "reviewed-geometry-terminal",
        definition_limits(),
    )
    .expect("the terminal workflow identity is valid");
    let terminal = builder
        .terminal(terminal_identity)
        .expect("the terminal start node is valid");
    builder.start(&terminal);
    builder
        .finish()
        .expect("the terminal definition is complete")
        .validate()
        .expect("the terminal definition is valid")
}

pub fn proposal_terminal_definition() -> ValidatedWorkflowDefinition<ReviewedGeometryWorkflow> {
    let mut builder = ApplicationWorkflowDefinitionBuilder::<ReviewedGeometryWorkflow>::new(
        "proposal-terminal",
        definition_limits(),
    )
    .expect("the proposal-terminal workflow identity is valid");
    let proposal = builder
        .operation::<WorkflowDefinitionAuthoringOperation>("proposal", false)
        .expect("the proposal node is valid");
    let terminal = builder
        .terminal("completed")
        .expect("the terminal node is valid");
    builder.start(&proposal).control(
        &proposal,
        ApplicationWorkflowControlOutcome::Completed,
        &terminal,
    );
    builder
        .finish()
        .expect("the proposal-terminal definition is complete")
        .validate()
        .expect("the proposal-terminal definition is valid")
}

pub fn condition_terminal_definition() -> ValidatedWorkflowDefinition<ReviewedGeometryWorkflow> {
    condition_terminal_draft()
        .validate()
        .expect("the condition definition is valid")
}

pub fn condition_terminal_draft() -> AuthoredWorkflowDefinition<ReviewedGeometryWorkflow> {
    let mut builder = ApplicationWorkflowDefinitionBuilder::<ReviewedGeometryWorkflow>::new(
        "condition-terminal",
        definition_limits(),
    )
    .expect("the condition workflow identity is valid");
    let proposal = builder
        .operation::<WorkflowDefinitionAuthoringOperation>("proposal", false)
        .expect("the proposal node is valid");
    let condition = builder
        .condition::<PartDimensionConditionQuery>("positive-dimension")
        .expect("the condition node is valid");
    let satisfied = builder
        .terminal("satisfied")
        .expect("the satisfied terminal is valid");
    let unsatisfied = builder
        .terminal("unsatisfied")
        .expect("the unsatisfied terminal is valid");
    builder
        .start(&proposal)
        .control(
            &proposal,
            ApplicationWorkflowControlOutcome::Completed,
            &condition,
        )
        .control(
            &condition,
            ApplicationWorkflowControlOutcome::ConditionSatisfied,
            &satisfied,
        )
        .control(
            &condition,
            ApplicationWorkflowControlOutcome::ConditionUnsatisfied,
            &unsatisfied,
        )
        .condition_subject(&proposal, &condition);
    builder
        .finish()
        .expect("the condition definition is complete")
}

pub fn repeated_proposal_definition() -> ValidatedWorkflowDefinition<ReviewedGeometryWorkflow> {
    let mut builder = ApplicationWorkflowDefinitionBuilder::<ReviewedGeometryWorkflow>::new(
        "repeated-proposal",
        definition_limits(),
    )
    .expect("the repeated-proposal workflow identity is valid");
    let first = builder
        .operation::<WorkflowDefinitionAuthoringOperation>("proposal/first", false)
        .expect("the first proposal node is valid");
    let second = builder
        .operation::<WorkflowDefinitionAuthoringOperation>("proposal/second", false)
        .expect("the second proposal node is valid");
    let terminal = builder
        .terminal("completed")
        .expect("the terminal node is valid");
    builder
        .start(&first)
        .control(
            &first,
            ApplicationWorkflowControlOutcome::Completed,
            &second,
        )
        .control(
            &second,
            ApplicationWorkflowControlOutcome::Completed,
            &terminal,
        );
    builder
        .finish()
        .expect("the repeated-proposal definition is complete")
        .validate()
        .expect("the repeated-proposal definition is valid")
}

pub fn bind_definition(
    application: &BoundedDimensionWorkflowRuntime,
    definition: ValidatedWorkflowDefinition<ReviewedGeometryWorkflow>,
) -> WorthQueryInstalledWorkflowDefinitionContract<
    BoundedDimensionSchema,
    ReviewedGeometryWorkflow,
    DimensionProgramP0,
> {
    application
        .workflow_spec()
        .bind_definition(definition)
        .expect("the workflow definition binds to installed vocabulary")
}

pub fn publish_definition(
    application: &BoundedDimensionWorkflowRuntime,
    definition: ValidatedWorkflowDefinition<ReviewedGeometryWorkflow>,
    expected_predecessor: WorkflowDefinitionExpectedPredecessor,
    idempotency: u64,
) -> Result<
    WorkflowDefinitionPublicationOutcome,
    WorthQueryWorkflowDefinitionPublicationPreparationDenial,
> {
    let contract = bind_definition(application, definition);
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    runtime
        .request(&principal, &scope)
        .mutate(authoring_intent())
        .without_source()
        .idempotency(&idempotency)
        .prepare_workflow_publication(contract, expected_predecessor)
        .map(|request| request.execute())
}

pub fn retire_definition(
    application: &BoundedDimensionWorkflowRuntime,
    definition: PublishedWorkflowDefinitionRef,
    idempotency: u64,
) -> Result<
    WorkflowDefinitionRetirementOutcome,
    WorthQueryWorkflowDefinitionRetirementPreparationDenial,
> {
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    runtime
        .request(&principal, &scope)
        .mutate(authoring_intent())
        .without_source()
        .idempotency(&idempotency)
        .prepare_workflow_definition_retirement(application, definition)
        .map(|request| request.execute())
}

/// The authoring request every definition publication and retirement presents.
pub fn authoring_intent() -> WorkflowDefinitionAuthoringIntent {
    WorkflowDefinitionAuthoringIntent {
        input: WorkflowDefinitionAuthoringInput {
            identity: PART_IDENTITY.to_owned(),
            dimension: 8,
        },
    }
}

pub(crate) fn definition_limits() -> ApplicationWorkflowDefinitionLimits {
    ApplicationWorkflowDefinitionLimits::new(
        32,
        64,
        4,
        ApplicationWorkflowComponentLimits::new(32, 4, 128, 256, 256).unwrap(),
        64 * 1024,
    )
    .expect("the workflow definition limits are nonzero")
}
