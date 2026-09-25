use worth_query_host::facade::application_entry::{
    WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPublicationOutcome,
    WorkflowInstanceStartOutcome,
};
use worth_relational::facade::transactions::{
    ConflictClass, InvariantViolationFields, TransactionCommitError,
};

use super::bounded_dimension_model::{
    host::publish_workflow_on_first_program,
    workflow::{publish_definition, start_instance, terminal_definition},
};

#[test]
fn native_membership_mutation_is_denied_without_poisoning_warm_compilation() {
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

    let mutation = application
        .runtime()
        .attempt_workflow_definition_membership_cycle_for_test(definition.definition())
        .expect_err("published membership must be immutable through the native writer");
    assert_publication_immutability_denial(mutation);
    let field_mutation = application
        .runtime()
        .attempt_workflow_node_field_update_for_test(definition.definition())
        .expect_err("published node meaning must be immutable through the native writer");
    assert_publication_immutability_denial(field_mutation);
    let attachment = application
        .runtime()
        .attempt_workflow_existing_node_attachment_for_test(definition.definition())
        .expect_err("a new definition must not attach an existing published node");
    assert_publication_immutability_denial(attachment);

    assert!(matches!(
        start_instance(&application, definition.definition().clone(), 1_203)
            .expect("unchanged publication must still prepare"),
        WorkflowInstanceStartOutcome::Started(_)
    ));
    let after = application.runtime().workflow_compilation_reuse_counters();
    assert_eq!(after.warm_hits(), before.warm_hits() + 1);
    assert_eq!(after.cold_misses(), before.cold_misses());
}

fn assert_publication_immutability_denial(error: TransactionCommitError) {
    let TransactionCommitError::Conflict { error, .. } = error else {
        panic!("expected a native invariant conflict, got {error:?}");
    };
    let ConflictClass::InvariantViolation {
        fields: InvariantViolationFields::CustomInvariantViolation { identity },
        ..
    } = error.class
    else {
        panic!("expected a custom invariant denial, got {error:?}");
    };
    assert_eq!(
        identity.rule_id.as_str(),
        "worth-query.workflow.publication-immutability"
    );
}
