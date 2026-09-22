use worth_query_host::facade::{
    application_entry::{
        WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPublicationOutcome,
        WorkflowInstanceStartOutcome, WorthQueryApplicationRequestExt,
        WorthQueryOrdinaryWorkflowPublicationDenial,
    },
    declaration::application_program::{
        ApplicationWorkflowDefinitionBuilder, AuthoredWorkflowDefinition,
    },
};

use super::bounded_dimension_model::{
    dimension_entry::PART_IDENTITY,
    host::publish_workflow_on_first_program,
    operator_identity::{authenticate_operator, request_scope},
    workflow::{
        start_instance, ReviewedGeometryWorkflow, WorkflowDefinitionAuthoringInput,
        WorkflowDefinitionAuthoringIntent,
    },
};

fn terminal_draft() -> AuthoredWorkflowDefinition<ReviewedGeometryWorkflow> {
    let mut builder = ApplicationWorkflowDefinitionBuilder::<ReviewedGeometryWorkflow>::new(
        "ordinary-terminal",
        super::bounded_dimension_model::workflow::definition_limits(),
    )
    .expect("the ordinary workflow identity is valid");
    let done = builder.terminal("done").expect("the terminal is valid");
    builder.start(&done);
    builder.finish().expect("the draft is complete")
}

#[test]
fn ordinary_publication_uses_installed_binding_and_real_commit_authority() {
    let application = publish_workflow_on_first_program();
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let key = 918_001_u64;

    let publish = || {
        runtime
            .request(&principal, &scope)
            .mutate(WorkflowDefinitionAuthoringIntent {
                input: WorkflowDefinitionAuthoringInput {
                    identity: PART_IDENTITY.to_owned(),
                    dimension: 8,
                },
            })
            .workflow(&application, terminal_draft())
            .expect("the draft binds to installed support")
            .publish(WorkflowDefinitionExpectedPredecessor::Absent)
            .idempotency(&key)
            .execute()
            .expect("the ordinary publication prepares")
    };
    let first = match publish() {
        WorkflowDefinitionPublicationOutcome::Published(performed) => performed,
        other => panic!("the ordinary definition must publish: {other:?}"),
    };
    assert!(!first.replayed());
    let replay = match publish() {
        WorkflowDefinitionPublicationOutcome::Published(performed) => performed,
        other => panic!("the exact ordinary retry must publish: {other:?}"),
    };
    assert!(replay.replayed());
    assert_eq!(first.definition(), replay.definition());
    match start_instance(&application, first.definition().clone(), 918_002)
        .expect("the ordinary definition start prepares")
    {
        WorkflowInstanceStartOutcome::Started(started) => {
            assert_eq!(started.instance().current_node_path(), "done")
        }
        other => panic!("the ordinary definition must start: {other:?}"),
    }
}

#[test]
fn ordinary_publication_rejects_invalid_draft_before_mutation_preparation() {
    let application = publish_workflow_on_first_program();
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let draft = ApplicationWorkflowDefinitionBuilder::<ReviewedGeometryWorkflow>::new(
        "missing-start",
        super::bounded_dimension_model::workflow::definition_limits(),
    )
    .expect("the identity is valid")
    .finish()
    .expect("the incomplete draft can be represented");
    let result = runtime
        .request(&principal, &scope)
        .mutate(WorkflowDefinitionAuthoringIntent {
            input: WorkflowDefinitionAuthoringInput {
                identity: PART_IDENTITY.to_owned(),
                dimension: 8,
            },
        })
        .workflow(&application, draft);
    assert!(matches!(
        result,
        Err(WorthQueryOrdinaryWorkflowPublicationDenial::InvalidDefinition(_))
    ));
}

#[test]
fn ordinary_publication_rejects_a_workflow_runtime_from_another_application() {
    let application = publish_workflow_on_first_program();
    let foreign = publish_workflow_on_first_program();
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let result = runtime
        .request(&principal, &scope)
        .mutate(WorkflowDefinitionAuthoringIntent {
            input: WorkflowDefinitionAuthoringInput {
                identity: PART_IDENTITY.to_owned(),
                dimension: 8,
            },
        })
        .workflow(&foreign, terminal_draft());
    assert!(matches!(
        result,
        Err(WorthQueryOrdinaryWorkflowPublicationDenial::RuntimeMismatch)
    ));
}
