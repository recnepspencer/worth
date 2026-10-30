pub use bank_domain::schema::ApprovePayment;
pub use bank_server::BankApprovedPaymentWorkflow;
pub use worth_query_host::facade::admission::authentication_event::{
    WorthQueryAuthenticationEventDenial, WorthQueryAuthenticationEventVerifierFailure,
};
pub use worth_query_host::facade::application_entry::{
    PublishedWorkflowInstanceRef, PublishedWorkflowProposalRef, RequiredWorkflowApproval,
    WorkflowApprovalDecision,
};

pub use super::assertions::{key, require_completed};
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

    require_completed(
        block_on(workflow.approve(
            instance.clone(),
            required,
            proposal,
            WorkflowApprovalDecision::Approve,
            valid_credential(),
            authority.clone(),
            &key("approved-payment:approval"),
        ))
        .expect("the Bank approver approves the exact proposal and evidence"),
        "approval",
    );

    require_completed(
        block_on(workflow.approve(
            instance.clone(),
            required,
            proposal,
            WorkflowApprovalDecision::Approve,
            invalid_credential(),
            authority.clone(),
            &key("approved-payment:approval"),
        ))
        .expect("the exact committed approval replays without reusing authentication"),
        "approval",
    );
}
