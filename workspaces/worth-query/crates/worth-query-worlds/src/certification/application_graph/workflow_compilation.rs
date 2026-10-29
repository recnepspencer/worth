pub use worth_query_host::facade::application_entry::{
    WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPublicationOutcome,
    WorkflowInstanceStartOutcome,
};
pub use worth_relational::facade::transactions::{
    ConflictClass, InvariantViolationFields, TransactionCommitError,
};

pub use super::document_retention_model::{
    host::publish_workflow_on_first_program,
    workflow::{publish_definition, start_instance, terminal_definition},
};

pub fn assert_fact_custody_denial(error: TransactionCommitError) {
    let TransactionCommitError::Conflict { error, .. } = error else {
        panic!("expected a native invariant conflict, got {error:?}");
    };
    let ConflictClass::InvariantViolation {
        fields:
            InvariantViolationFields::CustomInvariantViolation {
                identity,
                refusal: _,
            },
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

pub fn assert_membership_cardinality_denial(error: TransactionCommitError) {
    let TransactionCommitError::Conflict { error, .. } = error else {
        panic!("expected a native invariant conflict, got {error:?}");
    };
    let ConflictClass::InvariantViolation {
        fields:
            InvariantViolationFields::RelationCardinalityEndpoint {
                contract_id,
                count: 2,
                limit: 1,
                ..
            },
        ..
    } = error.class
    else {
        panic!("expected the membership cardinality denial, got {error:?}");
    };
    assert_eq!(
        contract_id.as_str(),
        "worth-query-workflow-definition-node:cardinality"
    );
}
