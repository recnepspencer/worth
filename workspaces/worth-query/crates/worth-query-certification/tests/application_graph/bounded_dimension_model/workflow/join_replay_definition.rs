use worth_query_host::facade::declaration::application_program::{
    ApplicationWorkflowControlOutcome, ApplicationWorkflowDefinitionBuilder,
    ValidatedWorkflowDefinition,
};

use super::super::schema::PartDimensionQuery;
use super::definition::definition_limits;
use super::{ReviewedGeometryWorkflow, WorkflowDefinitionAuthoringOperation};

pub fn assessment_join_terminal_definition() -> ValidatedWorkflowDefinition<ReviewedGeometryWorkflow>
{
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
        .evidence_join("checks/join")
        .expect("the join is valid");
    let terminal = builder
        .terminal("completed")
        .expect("the terminal is valid");
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
            ApplicationWorkflowControlOutcome::Completed,
            &terminal,
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
