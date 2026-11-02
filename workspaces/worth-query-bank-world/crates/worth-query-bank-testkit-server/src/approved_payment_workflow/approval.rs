pub use bank_domain::schema::ApprovePayment;
pub use bank_server::BankApprovedPaymentWorkflow;
pub use worth_query_host::facade::admission::authentication_event::{
    WorthQueryAuthenticationEventDenial, WorthQueryAuthenticationEventVerifierFailure,
};
pub use worth_query_host::facade::application_entry::{
    PublishedWorkflowInstanceRef, PublishedWorkflowProposalRef, RequiredWorkflowApproval,
    WorkflowApprovalDecision,
};

pub use super::assertions::key;
pub use super::authentication::{invalid_credential, valid_credential};
pub use super::support::block_on;

pub fn require_authenticated_approval_and_replay(
    workflow: &BankApprovedPaymentWorkflow<'_, '_, '_>,
    instance: &PublishedWorkflowInstanceRef,
    required: &RequiredWorkflowApproval,
    proposal: &PublishedWorkflowProposalRef,
    authority: &ApprovePayment,
) {
    let denied = block_on(workflow.approve(
        instance.clone(),
        required,
        proposal,
        WorkflowApprovalDecision::Approve,
        invalid_credential(),
        authority.clone(),
        &key("approved-payment:approval"),
    ))
    .expect_err("a principal without the installed factor cannot approve");
    assert_eq!(
        denied.authentication_denial(),
        Some(WorthQueryAuthenticationEventDenial::VerifierFailed(
            WorthQueryAuthenticationEventVerifierFailure::CredentialRejected
        ))
    );

    let original = block_on(workflow.approve(
        instance.clone(),
        required,
        proposal,
        WorkflowApprovalDecision::Approve,
        valid_credential(),
        authority.clone(),
        &key("approved-payment:approval"),
    ))
    .expect("the Bank approver approves the exact proposal and evidence");
    let super::assertions::WorkflowProgressOutcome::Completed(original) = original else {
        panic!("the original approval must perform");
    };
    assert_eq!(original.node_path(), "approval");
    let replay = block_on(workflow.approve(
        instance.clone(),
        required,
        proposal,
        WorkflowApprovalDecision::Approve,
        invalid_credential(),
        authority.clone(),
        &key("approved-payment:approval"),
    ))
    .expect("the exact committed approval replays without reusing authentication");
    let super::assertions::WorkflowProgressOutcome::RecoveredCommitted(data) = replay else {
        panic!("approval retry returns data, not a performed stage");
    };
    let live = original.receipt();
    assert_eq!(data.commit_reference(), live.commit_reference());
    assert_eq!(Some(data.outcome_identity()), live.outcome_identity());
    assert_eq!(
        data.idempotency_key_identity(),
        live.idempotency_binding().key_identity()
    );
    assert_eq!(data.changed_record_count(), live.changed_record_count());
    assert_eq!(data.emitted_effect_count(), live.emitted_effect_count());
    assert_eq!(
        data.exact_output_settlement(),
        live.recorded_output_settlement()
    );
    let publication = live.committed_product_publication();
    let head = data
        .new_product_head()
        .memory()
        .expect("the fixture has a real memory owner");
    assert_eq!(
        head.owner(),
        publication.product_branch().owner_identity().get()
    );
    assert_eq!(head.branch(), publication.product_branch().name().as_str());
    assert_eq!(
        head.incarnation(),
        publication.product_incarnation().ordinal()
    );
    assert_eq!(head.generation(), publication.product_generation().get());
    assert_eq!(
        head.selected_commit(),
        publication.composite_commit().ordinal()
    );
    assert_eq!(
        data.attempt_identity().ordinal(),
        publication.publication_attempt().ordinal()
    );
}
