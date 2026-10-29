pub use crate::facade::server::fixture;
pub use crate::facade::server::support;

pub use std::time::{Duration, Instant};

pub use bank_domain::model::{BusinessId, Money};
pub use bank_domain::proposals::{BankIdempotencyKey, BankProposalDenial};
pub use bank_domain::schema::{ApprovePayment, InitiateBusinessPayment, RejectPayment};
pub use bank_server::{
    mutations, queries, BankApplicationP1, BankMutationControls, BankPendingPaymentContinuation,
    BankReadControls,
};

pub use fixture::{ordinary_read_world, principal_id, APPROVER, OWNER, RECIPIENT};
pub use support::request_scope;
pub use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};
pub use worth_query_host::facade::application_contribution::{
    WorthQueryAdvancementDenial, WorthQueryManagedComputationInterruption,
};
pub use worth_query_host::facade::application_entry::{
    WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPublicationOutcome,
    WorkflowInstanceStartOutcome, WorthQueryApplicationMutationOutcome,
    WorthQueryApplicationRequestMutationDenial, WorthQueryApplicationRequestMutationDenialKind,
    WorthQueryBranchAdoptionPublicationOutcome,
    WorthQueryWorkflowDefinitionPublicationPreparationDenial,
};
pub use worth_query_host::facade::application_entry::{
    WorthQuerySourceCompletion as Completion, WorthQuerySourceOnlyAdvance as Advance,
    WorthQuerySourceOnlyDenial,
};

pub fn pending_continuation(
    fixture: &fixture::OrdinaryReadFixture,
    principal: &bank_server::BankAuthenticatedPrincipal,
) -> BankPendingPaymentContinuation {
    let result = fixture
        .world
        .runtime
        .query(queries::pending_payments())
        .as_principal(principal)
        .controls(BankReadControls::current(request_scope(), 16).unwrap())
        .execute()
        .expect("pending payments must be readable");
    let summary = result
        .rows()
        .iter()
        .find(|payment| payment.id() == fixture.payment)
        .cloned()
        .expect("fixture pending payment must be present");
    BankPendingPaymentContinuation::from_summary(summary)
        .expect("an approval-required payment must yield a continuation")
}

pub fn controls(idempotency: &str) -> BankMutationControls {
    BankMutationControls::new(
        request_scope(),
        BankIdempotencyKey::new(idempotency).unwrap(),
    )
}

#[path = "workflow_continuations/actor_identity.rs"]
pub mod actor_identity;
