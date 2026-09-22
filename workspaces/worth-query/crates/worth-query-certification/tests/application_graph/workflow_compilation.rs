use worth_query_host::facade::{
    application_entry::{
        WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPublicationOutcome,
        WorkflowInstancePreparationDenial, WorkflowInstanceStartOutcome,
        WorthQueryWorkflowInstanceStartPreparationDenial,
    },
    primary_graph::WorthQueryApplicationAttemptDenialKind,
};

use super::bounded_dimension_model::{
    host::publish_workflow_on_first_program,
    operator_identity::request_scope,
    workflow::{publish_definition, start_instance, terminal_definition},
};

#[test]
fn warm_compilation_denies_aba_definition_membership_change() {
    let application = publish_workflow_on_first_program();
    let definition = match publish_definition(
        &application,
        terminal_definition("completed"),
        WorkflowDefinitionExpectedPredecessor::Absent,
        1_201,
    )
    .expect("the definition publication must prepare")
    {
        WorkflowDefinitionPublicationOutcome::Published(publication) => publication,
        unexpected => panic!("expected a published definition, got {unexpected:?}"),
    };
    match start_instance(&application, definition.definition().clone(), 1_202)
        .expect("the cold workflow start must prepare")
    {
        WorkflowInstanceStartOutcome::Started(_) => {}
        unexpected => panic!("expected a started workflow, got {unexpected:?}"),
    }
    let before = application.runtime().workflow_compilation_reuse_counters();

    application
        .runtime()
        .cycle_workflow_definition_membership_for_test(definition.definition(), &request_scope());

    let denial = start_instance(&application, definition.definition().clone(), 1_203)
        .expect_err("a stale warm membership binding must be denied");
    match denial {
        WorthQueryWorkflowInstanceStartPreparationDenial::InstancePreparation(
            WorkflowInstancePreparationDenial::Attempt(attempt),
        ) => {
            assert_eq!(
                attempt.kind(),
                WorthQueryApplicationAttemptDenialKind::WorkflowDefinitionCompilationUnavailable
            );
            assert_eq!(
                attempt.subject(),
                "cached workflow publication membership is stale"
            );
        }
        unexpected => panic!("expected a compilation attempt denial, got {unexpected:?}"),
    }
    let after = application.runtime().workflow_compilation_reuse_counters();
    assert_eq!(after.warm_hits(), before.warm_hits() + 1);
    assert_eq!(after.cold_misses(), before.cold_misses());
    assert!(matches!(
        start_instance(&application, definition.definition().clone(), 1_204),
        Err(
            WorthQueryWorkflowInstanceStartPreparationDenial::InstancePreparation(
                WorkflowInstancePreparationDenial::Attempt(_)
            )
        )
    ));
}
