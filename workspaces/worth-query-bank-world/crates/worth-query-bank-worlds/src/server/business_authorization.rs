#![allow(
    clippy::result_large_err,
    reason = "Authorization courts preserve complete native workflow refusal evidence"
)]

pub use worth_query_host::facade::primary_graph::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
#[path = "business_authorization/fixture.rs"]
pub mod business_authorization_fixture;
pub use crate::facade::server::support;

pub use bank_domain::model::{BankPrincipalId, BusinessId, CustomerRole, InstitutionId, Money};
pub use bank_domain::proposals::{BankProposalDenial, BankProposalEngine};
pub use bank_domain::schema::{
    ApprovePayment, InitiateBusinessPayment, RevokeAccountAuthorization,
    MAX_PAYMENT_APPROVAL_GRANTEES,
};
pub use bank_server::{
    mutations, BankApprovedPaymentWorkflowError, BankBusinessOwnerSeed, BankEmployeeAssignmentSeed,
    BankMutationControls, BankPrincipalSeed, BankWorldSeed,
};
pub use worth_query_host::facade::application_entry::{
    WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPublicationOutcome,
    WorkflowInstanceStartOutcome, WorthQueryApplicationMutationOutcome,
    WorthQueryApplicationRequestMutationDenial,
    WorthQueryWorkflowDefinitionPublicationPreparationDenial,
};
pub use worth_query_host::facade::primary_graph::WorthQueryOperationAuthorizationDenialKind;

pub use business_authorization_fixture::{
    approver_crowded_world, binding, id, key, pending_business_payment_world,
};
pub use support::{
    block_on, request_scope, runtime, CausalCredential, DynamicIdentity, TestIdentityWorld,
};

// A business payment is approved only through its Bank-owned workflow, so the
// real authorization graph answers at the workflow's first authored step.

pub fn crowded_initiation(
    approvers: u64,
) -> (
    TestIdentityWorld,
    Vec<DynamicIdentity>,
    worth_query_host::facade::application_entry::WorthQuerySourceOnlyOperation<
        bank_domain::schema::BankSchema,
        InitiateBusinessPayment,
        bank_server::BankApplication,
        bank_server::PaymentInitiationAction,
        worth_query_host::facade::declaration::application_program::ApplicationSourceCompletion,
    >,
) {
    let (snapshot, source) = approver_crowded_world(approvers);
    let identities = (1..=2 + approvers)
        .map(|principal| DynamicIdentity::new(&format!("crowded-{principal}")))
        .collect::<Vec<_>>();
    let world = runtime(identities.iter().enumerate().fold(
        BankWorldSeed::new(snapshot).business_owner(BankBusinessOwnerSeed::new(
            id(BusinessId::new, 1),
            id(BankPrincipalId::new, 1),
        )),
        |seed, (ordinal, identity)| {
            seed.principal(BankPrincipalSeed::enabled(
                id(BankPrincipalId::new, u64::try_from(ordinal).unwrap() + 1),
                identity.external(),
            ))
        },
    ));
    let initiator = authenticate(&world, &identities[0]);
    let initiated = world
        .runtime
        .mutate(mutations::initiate_business_payment(
            InitiateBusinessPayment {
                business: id(BusinessId::new, 1),
                from: source,
                recipient: id(BankPrincipalId::new, 2),
                amount: Money::from_minor(700).unwrap(),
            },
        ))
        .as_principal(&initiator)
        .controls(BankMutationControls::new(
            request_scope(),
            key("crowded-initiation"),
        ))
        .retain_operation()
        .unwrap_or_else(|cause| panic!("declared initiation: {cause:?}"));
    (world, identities, initiated)
}

pub fn authenticate(
    world: &TestIdentityWorld,
    identity: &DynamicIdentity,
) -> bank_server::BankAuthenticatedPrincipal {
    block_on(world.runtime.authenticate_with(
        &world.authentication,
        CausalCredential::for_identity(identity),
        &request_scope(),
    ))
    .unwrap()
}

pub fn publish(
    world: &TestIdentityWorld,
    identity: &DynamicIdentity,
    authority: ApprovePayment,
    label: &str,
) -> Result<WorkflowDefinitionPublicationOutcome, BankApprovedPaymentWorkflowError> {
    let actor = authenticate(world, identity);
    let request = request_scope();
    world
        .runtime
        .approved_business_payment(&actor, &request)
        .publish_definition(
            authority,
            WorkflowDefinitionExpectedPredecessor::Absent,
            &key(&format!("{label}:definition")),
        )
}

pub fn assert_authorization_denied(
    result: Result<WorkflowDefinitionPublicationOutcome, BankApprovedPaymentWorkflowError>,
    expected: WorthQueryOperationAuthorizationDenialKind,
) {
    assert!(
        matches!(
            &result,
            Err(BankApprovedPaymentWorkflowError::DefinitionPublication(
                WorthQueryWorkflowDefinitionPublicationPreparationDenial::RequestAdmission(
                    WorthQueryApplicationRequestMutationDenial::Authorization(denial)
                )
            )) if denial.kind() == expected
        ),
        "expected {expected:?} from the real authorization graph, got {result:?}"
    );
}
