use worth_query_host::facade::declaration::application_program::{
    ApplicationWorkflowControlOutcome, ApplicationWorkflowDefinitionBuilder,
    ApplicationWorkflowRetry, ApplicationWorkflowSubjectSelector, ValidatedWorkflowDefinition,
};

use super::super::schema::PartDimensionQuery;
use super::definition::definition_limits;
use super::{ReviewedGeometryWorkflow, WorkflowDefinitionAuthoringOperation};

pub fn assessment_join_terminal_definition() -> ValidatedWorkflowDefinition<ReviewedGeometryWorkflow>
{
    assessment_join_terminal_definition_with_policy(
        worth_query_host::facade::declaration::application_program::ApplicationWorkflowEvidenceJoinPolicy::AllRequiredPassing,
    )
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
