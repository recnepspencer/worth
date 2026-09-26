use worth_query_host::facade::declaration::application_program::{
    ApplicationWorkflowControlOutcome, ApplicationWorkflowDefinitionBuilder,
    ApplicationWorkflowRetry, ApplicationWorkflowSubjectSelector, ValidatedWorkflowDefinition,
};

use super::super::{
    dimension_entry::ReviewedSetPartDimensionBinding,
    schema::{PartDimensionConditionQuery, PartDimensionQuery},
};
use super::definition::definition_limits;
use super::review_requirement::ReviewRequired;
use super::{
    ReviewedGeometryWorkflow, WorkflowApprovalCapability, WorkflowDefinitionAuthoringOperation,
};

pub fn assessment_join_terminal_definition() -> ValidatedWorkflowDefinition<ReviewedGeometryWorkflow>
{
    assessment_join_terminal_definition_with_policy(
        worth_query_host::facade::declaration::application_program::ApplicationWorkflowEvidenceJoinPolicy::AllRequiredPassing,
    )
}

pub fn early_assessment_definition() -> ValidatedWorkflowDefinition<ReviewedGeometryWorkflow> {
    let mut builder = ApplicationWorkflowDefinitionBuilder::<ReviewedGeometryWorkflow>::new(
        "early-assessment-coverage",
        definition_limits(),
    )
    .expect("the workflow identity is valid");
    let propose = builder
        .operation::<WorkflowDefinitionAuthoringOperation>("propose", false)
        .expect("the proposal node is valid");
    let first = builder
        .assessment::<PartDimensionQuery>("checks/first")
        .expect("the first assessment is valid");
    let second = builder
        .assessment::<PartDimensionQuery>("checks/second")
        .expect("the second assessment is valid");
    let related = builder
        .assessment_for::<PartDimensionQuery>(
            "checks/related",
            ApplicationWorkflowSubjectSelector::related(),
        )
        .expect("the related assessment is valid");
    let join = builder
        .evidence_join(
            "checks/join",
            worth_query_host::facade::declaration::application_program::ApplicationWorkflowEvidenceJoinPolicy::AllRequiredPassing,
        )
        .expect("the join is valid");
    let completed = builder
        .terminal("completed")
        .expect("the terminal is valid");
    let rejected = builder
        .terminal("rejected")
        .expect("the rejection terminal is valid");
    builder
        .start(&propose)
        .control(
            &propose,
            ApplicationWorkflowControlOutcome::Completed,
            &first,
        )
        .control(
            &first,
            ApplicationWorkflowControlOutcome::Completed,
            &second,
        )
        .control(
            &second,
            ApplicationWorkflowControlOutcome::Completed,
            &related,
        )
        .control(
            &related,
            ApplicationWorkflowControlOutcome::Completed,
            &join,
        )
        .control(
            &join,
            ApplicationWorkflowControlOutcome::EvidenceSatisfied,
            &completed,
        )
        .control(
            &join,
            ApplicationWorkflowControlOutcome::EvidenceFailed,
            &rejected,
        )
        .proposal_for_assessment(&propose, &first)
        .proposal_for_assessment(&propose, &second)
        .proposal_for_assessment(&propose, &related)
        .assessment_evidence(&first, &join)
        .assessment_evidence(&second, &join)
        .assessment_evidence(&related, &join);
    builder
        .finish()
        .expect("the definition is complete")
        .validate()
        .expect("the definition is valid")
}

/// The Related review is authored in advance but becomes required only when
/// the installed Resource→Related review relation exists at the join.
pub fn conditionally_required_related_assessment_definition(
) -> ValidatedWorkflowDefinition<ReviewedGeometryWorkflow> {
    let mut builder = ApplicationWorkflowDefinitionBuilder::<ReviewedGeometryWorkflow>::new(
        "conditional-related-review",
        definition_limits(),
    )
    .expect("the workflow identity is valid");
    let proposal = builder
        .operation::<WorkflowDefinitionAuthoringOperation>("propose", false)
        .unwrap();
    let condition = builder
        .condition::<PartDimensionConditionQuery>("positive-dimension")
        .unwrap();
    let related = builder
        .assessment_when_related_relation_present::<PartDimensionQuery, _, _, _>(
            "checks/related",
            ReviewRequired::reference(),
        )
        .unwrap();
    let first = builder
        .assessment::<PartDimensionQuery>("checks/first")
        .unwrap();
    let second = builder
        .assessment::<PartDimensionQuery>("checks/second")
        .unwrap();
    let join = builder.evidence_join(
        "checks/join",
        worth_query_host::facade::declaration::application_program::ApplicationWorkflowEvidenceJoinPolicy::AllRequiredPassing,
    ).unwrap();
    let approval = builder
        .approval::<WorkflowApprovalCapability>("approval")
        .unwrap();
    let apply = builder
        .operation_binding::<ReviewedSetPartDimensionBinding>("apply")
        .unwrap();
    let completed = builder.terminal("completed").unwrap();
    let rejected = builder.terminal("rejected").unwrap();
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
            &related,
        )
        .control(
            &condition,
            ApplicationWorkflowControlOutcome::ConditionUnsatisfied,
            &first,
        )
        .control(
            &related,
            ApplicationWorkflowControlOutcome::Completed,
            &first,
        )
        .control(
            &first,
            ApplicationWorkflowControlOutcome::Completed,
            &second,
        )
        .control(&second, ApplicationWorkflowControlOutcome::Completed, &join)
        .control(
            &join,
            ApplicationWorkflowControlOutcome::EvidenceSatisfied,
            &approval,
        )
        .control(
            &join,
            ApplicationWorkflowControlOutcome::EvidenceFailed,
            &rejected,
        )
        .control(
            &approval,
            ApplicationWorkflowControlOutcome::Approved,
            &apply,
        )
        .control(
            &approval,
            ApplicationWorkflowControlOutcome::Rejected,
            &rejected,
        )
        .control(
            &apply,
            ApplicationWorkflowControlOutcome::Completed,
            &completed,
        )
        .condition_subject(&proposal, &condition)
        .proposal_for_assessment(&proposal, &related)
        .proposal_for_assessment(&proposal, &first)
        .proposal_for_assessment(&proposal, &second)
        .assessment_evidence(&related, &join)
        .assessment_evidence(&first, &join)
        .assessment_evidence(&second, &join)
        .proposal_for_approval(&proposal, &approval)
        .joined_evidence(&join, &approval)
        .approval_authority(&approval, &apply)
        .operation_input(&proposal, &apply);
    builder.finish().unwrap().validate().unwrap()
}

pub fn assessment_join_terminal_definition_with_policy(
    policy: worth_query_host::facade::declaration::application_program::ApplicationWorkflowEvidenceJoinPolicy,
) -> ValidatedWorkflowDefinition<ReviewedGeometryWorkflow> {
    let mut builder = ApplicationWorkflowDefinitionBuilder::<ReviewedGeometryWorkflow>::new(
        "assessment-join-terminal",
        definition_limits(),
    )
    .expect("the workflow identity is valid");
    let propose = builder
        .operation::<WorkflowDefinitionAuthoringOperation>("propose", false)
        .expect("the proposal node is valid");
    let first = builder
        .assessment::<PartDimensionQuery>("checks/first")
        .expect("the first assessment is valid");
    let second = builder
        .assessment::<PartDimensionQuery>("checks/second")
        .expect("the second assessment is valid");
    let join = builder
        .evidence_join("checks/join", policy)
        .expect("the join is valid");
    let terminal = builder
        .terminal("completed")
        .expect("the terminal is valid");
    let rejected = builder
        .terminal("rejected")
        .expect("the rejection terminal is valid");
    builder
        .start(&propose)
        .control(
            &propose,
            ApplicationWorkflowControlOutcome::Completed,
            &first,
        )
        .control(
            &first,
            ApplicationWorkflowControlOutcome::Completed,
            &second,
        )
        .control(&second, ApplicationWorkflowControlOutcome::Completed, &join)
        .control(
            &join,
            ApplicationWorkflowControlOutcome::EvidenceSatisfied,
            &terminal,
        )
        .control(
            &join,
            ApplicationWorkflowControlOutcome::EvidenceFailed,
            &rejected,
        )
        .proposal_for_assessment(&propose, &first)
        .proposal_for_assessment(&propose, &second)
        .assessment_evidence(&first, &join)
        .assessment_evidence(&second, &join);
    builder
        .finish()
        .expect("the definition is complete")
        .validate()
        .expect("the definition is valid")
}

pub fn assessment_retry_definition() -> ValidatedWorkflowDefinition<ReviewedGeometryWorkflow> {
    assessment_retry_definition_with_subjects("assessment-retry", false)
}

pub fn multi_subject_assessment_retry_definition(
) -> ValidatedWorkflowDefinition<ReviewedGeometryWorkflow> {
    assessment_retry_definition_with_subjects("multi-subject-assessment-retry", true)
}

fn assessment_retry_definition_with_subjects(
    identity: &str,
    second_is_related: bool,
) -> ValidatedWorkflowDefinition<ReviewedGeometryWorkflow> {
    let mut builder = ApplicationWorkflowDefinitionBuilder::<ReviewedGeometryWorkflow>::new(
        identity,
        definition_limits(),
    )
    .expect("the workflow identity is valid");
    let propose = builder
        .operation::<WorkflowDefinitionAuthoringOperation>("propose", false)
        .expect("the proposal node is valid");
    let first = builder
        .assessment::<PartDimensionQuery>("checks/first")
        .expect("the first assessment is valid");
    let second = if second_is_related {
        builder
            .assessment_for::<PartDimensionQuery>(
                "checks/second",
                ApplicationWorkflowSubjectSelector::related(),
            )
            .expect("the related assessment is valid")
    } else {
        builder
            .assessment::<PartDimensionQuery>("checks/second")
            .expect("the second assessment is valid")
    };
    let join = builder
        .evidence_join(
            "checks/join",
            worth_query_host::facade::declaration::application_program::ApplicationWorkflowEvidenceJoinPolicy::AllRequiredPassing,
        )
        .expect("the join is valid");
    let completed = builder
        .terminal("completed")
        .expect("the completion terminal is valid");
    let exhausted = builder
        .terminal("retry-exhausted")
        .expect("the exhaustion terminal is valid");
    builder
        .start(&propose)
        .control(
            &propose,
            ApplicationWorkflowControlOutcome::Completed,
            &first,
        )
        .control(
            &first,
            ApplicationWorkflowControlOutcome::Completed,
            &second,
        )
        .control(&second, ApplicationWorkflowControlOutcome::Completed, &join)
        .control(
            &join,
            ApplicationWorkflowControlOutcome::EvidenceSatisfied,
            &completed,
        )
        .retry(
            &join,
            ApplicationWorkflowRetry::new(
                ApplicationWorkflowControlOutcome::EvidenceFailed,
                "assessment-correction",
                1,
            )
            .expect("the assessment correction is bounded"),
            &first,
        )
        .control(
            &join,
            ApplicationWorkflowControlOutcome::RetryExhausted,
            &exhausted,
        )
        .proposal_for_assessment(&propose, &first)
        .proposal_for_assessment(&propose, &second)
        .assessment_evidence(&first, &join)
        .assessment_evidence(&second, &join);
    builder
        .finish()
        .expect("the definition is complete")
        .validate()
        .expect("the definition is valid")
}
