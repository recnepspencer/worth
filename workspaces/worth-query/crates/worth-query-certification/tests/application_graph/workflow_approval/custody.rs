//! Already-published approval facts are immutable at the native commit boundary.

use super::*;
use worth_relational::facade::transactions::{
    ConflictClass, InvariantViolationFields, TransactionCommitError,
};

#[test]
fn native_writer_cannot_revise_a_published_approval_or_delete_its_evidence() {
    let (application, _, instance, proposal, required, _) = approval_journey("applied", 1_880);
    let approval = match approve_instance(
        &application,
        instance.clone(),
        &required,
        &proposal,
        WorkflowApprovalDecision::Approve,
        1_890,
    ) {
        Ok(WorkflowProgressOutcome::Completed(performed)) => performed
            .approval()
            .expect("approval publication must expose its durable fact")
            .entity(),
        other => panic!("expected a published approval, got {other:?}"),
    };
    let error = application
        .runtime()
        .attempt_workflow_approval_field_update_for_test(&instance, approval)
        .expect_err("a native writer cannot revise an issued decision");
    assert_fact_custody_denial(error);
    let error = application
        .runtime()
        .attempt_workflow_approval_evidence_delete_for_test(&instance, approval)
        .expect_err("a native writer cannot delete issued approval evidence");
    assert_fact_custody_denial(error);
}

fn assert_fact_custody_denial(error: TransactionCommitError) {
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
