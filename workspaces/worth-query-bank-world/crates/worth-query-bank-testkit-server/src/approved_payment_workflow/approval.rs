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
pub use super::authentication::{complete_approval, invalid_credential, valid_credential};

pub fn require_authenticated_approval_and_replay(
    workflow: &BankApprovedPaymentWorkflow<'_, '_, '_>,
    instance: &PublishedWorkflowInstanceRef,
    required: &RequiredWorkflowApproval,
    proposal: &PublishedWorkflowProposalRef,
    authority: &ApprovePayment,
) {
    let denied = complete_approval(workflow.approve(
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

    let original = complete_approval(workflow.approve(
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
    let replay = complete_approval(workflow.approve(
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
    super::recorded_commit_axes::assert_agrees(&data, original.receipt());
}
