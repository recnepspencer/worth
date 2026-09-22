use worth_query_host::facade::declaration::application_program::{
    ApplicationWorkflowControlOutcome, ApplicationWorkflowDefinitionBuilder,
    ApplicationWorkflowRetry, ValidatedWorkflowDefinition,
};

use super::definition::definition_limits;
use super::{ReviewedGeometryWorkflow, WorkflowDefinitionAuthoringOperation};

pub fn bounded_retry_definition() -> ValidatedWorkflowDefinition<ReviewedGeometryWorkflow> {
    bounded_retry_definition_with_attempts(2)
}

pub fn bounded_retry_definition_with_attempts(
    attempts: u16,
) -> ValidatedWorkflowDefinition<ReviewedGeometryWorkflow> {
    let mut builder = ApplicationWorkflowDefinitionBuilder::<ReviewedGeometryWorkflow>::new(
        "bounded-retry",
        definition_limits(),
    )
    .expect("the bounded-retry identity is valid");
    let first = builder
        .operation::<WorkflowDefinitionAuthoringOperation>("proposal/first", false)
        .expect("the first proposal node is valid");
    let revise = builder
        .operation::<WorkflowDefinitionAuthoringOperation>("proposal/revise", false)
        .expect("the revision node is valid");
    let exhausted = builder
        .terminal("retry-exhausted")
        .expect("the exhaustion terminal is valid");
    let retry = ApplicationWorkflowRetry::new(
        ApplicationWorkflowControlOutcome::Completed,
        "proposal-revision",
        attempts,
    )
    .expect("the retry policy is bounded");
    builder
        .start(&first)
        .control(
            &first,
            ApplicationWorkflowControlOutcome::Completed,
            &revise,
        )
        .retry(&revise, retry, &first)
        .control(
            &revise,
            ApplicationWorkflowControlOutcome::RetryExhausted,
            &exhausted,
        );
    builder
        .finish()
        .expect("the bounded-retry definition is complete")
        .validate()
        .expect("the bounded-retry definition is valid")
}
