pub use worth_query::facade::domain;

pub use super::installed_operation_fixture::{
    provisional_workflow_workspace, GeometryDomain, ProvisionalDiscardFamily, ProvisionalWorkflow,
};

pub fn discard_intent(input: &str) -> domain::WorthQueryNormalizedWorkflowIntent {
    domain::WorthQueryNormalizedWorkflowIntent::new(vec![
        domain::WorthQueryWorkflowIntentStage::new(
            "apply",
            domain::WorthQueryWorkflowIntentValue::EntityIdentity(input.into()),
        ),
    ])
    .unwrap()
}
