use worth_query::facade::domain;

use super::installed_operation_fixture::{
    provisional_workflow_workspace, GeometryDomain, ProvisionalDiscardFamily, ProvisionalWorkflow,
};

fn discard_intent(input: &str) -> domain::WorthQueryNormalizedWorkflowIntent {
    domain::WorthQueryNormalizedWorkflowIntent::new(vec![
        domain::WorthQueryWorkflowIntentStage::new(
            "apply",
            domain::WorthQueryWorkflowIntentValue::EntityIdentity(input.into()),
        ),
    ])
    .unwrap()
}

#[test]
fn provisional_discard_consumes_only_an_effect_free_provisional_trace() {
    let mut workspace = provisional_workflow_workspace("provisional-discard").unwrap();
    let installed = workspace.domain(GeometryDomain).unwrap();
    let trace = workspace
        .prepare_mutation_operating_world(workspace.current_world())
        .unwrap()
        .family(ProvisionalDiscardFamily)
        .bind(&installed, ProvisionalWorkflow)
        .unwrap()
        .admit_workflow_resources(
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .reexecute(discard_intent("discard"), &mut workspace)
        .unwrap();
    let trace_identity = trace.identity().to_owned();
    let discarded = trace.discard_provisional().unwrap();
    assert_eq!(discarded.original_trace_identity(), trace_identity);
    assert_ne!(discarded.identity(), trace_identity);
}
