#[path = "business_authorization/fixture.rs"]
mod business_authorization_fixture;
mod support;

use bank_domain::model::{BankPrincipalId, BusinessId, CustomerRole, InstitutionId, Money};
use bank_domain::proposals::BankProposalEngine;
use bank_domain::schema::{
    ApprovePayment, InitiateBusinessPayment, RevokeAccountAuthorization,
    MAX_PAYMENT_APPROVAL_GRANTEES,
};
use bank_server::{
    BankApprovedPaymentWorkflowError, BankBusinessOwnerSeed, BankEmployeeAssignmentSeed,
    BankPrincipalSeed, BankWorldSeed,
};
use worth_query_host::facade::application_entry::{
    WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPublicationOutcome,
    WorkflowInstanceStartOutcome, WorthQueryApplicationMutationOutcome,
    WorthQueryApplicationRequestMutationDenial,
    WorthQueryWorkflowDefinitionPublicationPreparationDenial,
};
use worth_query_host::facade::primary_graph::WorthQueryOperationAuthorizationDenialKind;

use business_authorization_fixture::{
    approver_crowded_world, binding, id, key, pending_business_payment_world,
};
use support::{
    block_on, request_scope, runtime, CausalCredential, DynamicIdentity, TestIdentityWorld,
};

// A business payment is approved only through its Bank-owned workflow, so the
// real authorization graph answers at the workflow's first authored step.

#[test]
fn real_graph_allows_distinct_approver_and_deny_precedence_blocks_initiator() {
    let (snapshot, payment) = pending_business_payment_world();
    let initiator = DynamicIdentity::new("initiator");
    let recipient = DynamicIdentity::new("recipient");
    let approver = DynamicIdentity::new("approver");
    let world = runtime(
        BankWorldSeed::new(snapshot)
            .principal(BankPrincipalSeed::enabled(
                id(BankPrincipalId::new, 1),
                initiator.external(),
            ))
            .principal(BankPrincipalSeed::enabled(
                id(BankPrincipalId::new, 2),
                recipient.external(),
            ))
            .principal(BankPrincipalSeed::enabled(
                id(BankPrincipalId::new, 3),
                approver.external(),
            ))
            .business_owner(BankBusinessOwnerSeed::new(
                id(BusinessId::new, 1),
                id(BankPrincipalId::new, 1),
            )),
    );
    let authority = ApprovePayment {
        payment,
        approver: id(BankPrincipalId::new, 3),
    };
    let published = publish(&world, &approver, authority.clone(), "approver")
        .expect("the distinct approver publishes the payment workflow");
    let WorkflowDefinitionPublicationOutcome::Published(published) = published else {
        panic!("expected definition publication, got {published:?}");
    };
    let request = request_scope();
    let actor = authenticate(&world, &approver);
    let started = world
        .runtime
        .approved_business_payment(&actor, &request)
        .start(
            published.definition().clone(),
            authority,
            &key("approver:start"),
        )
        .expect("the distinct approver starts the payment workflow");
    assert!(matches!(started, WorkflowInstanceStartOutcome::Started(_)));

    assert_authorization_denied(
        publish(
            &world,
            &initiator,
            ApprovePayment {
                payment,
                approver: id(BankPrincipalId::new, 1),
            },
            "initiator",
        ),
        WorthQueryOperationAuthorizationDenialKind::ExplicitDenyRuleMatched,
    );
}

#[test]
fn viewer_cross_business_and_employee_roles_do_not_combine_into_approval() {
    let (snapshot, payment) = pending_business_payment_world();
    let first = DynamicIdentity::new("initiator");
    let combined = DynamicIdentity::new("viewer-cross-business-teller");
    let third = DynamicIdentity::new("approver");
    let world = runtime(
        BankWorldSeed::new(snapshot)
            .principal(BankPrincipalSeed::enabled(
                id(BankPrincipalId::new, 1),
                first.external(),
            ))
            .principal(BankPrincipalSeed::enabled(
                id(BankPrincipalId::new, 2),
                combined.external(),
            ))
            .principal(BankPrincipalSeed::enabled(
                id(BankPrincipalId::new, 3),
                third.external(),
            ))
            .employee(BankEmployeeAssignmentSeed::new(
                id(bank_domain::model::EmployeeAssignmentId::new, 1),
                id(InstitutionId::new, 1),
                id(BankPrincipalId::new, 2),
                bank_domain::model::EmployeeRole::Teller,
            )),
    );
    assert_authorization_denied(
        publish(
            &world,
            &combined,
            ApprovePayment {
                payment,
                approver: id(BankPrincipalId::new, 2),
            },
            "cross-business",
        ),
        WorthQueryOperationAuthorizationDenialKind::CapabilityGrantMissing,
    );
}

#[test]
fn revoked_approver_membership_is_absent_from_current_authorization_graph() {
    let (snapshot, payment) = pending_business_payment_world();
    let authorization = snapshot
        .authorizations()
        .find(|candidate| {
            candidate.principal() == id(BankPrincipalId::new, 3)
                && candidate.role() == CustomerRole::Approver
        })
        .copied()
        .unwrap();
    let revoked = BankProposalEngine::prepare_revoke_account_authorization(
        &snapshot,
        binding(6),
        &key("revoke-approver"),
        &RevokeAccountAuthorization {
            account: authorization.account(),
            authorization: authorization.id(),
        },
    )
    .unwrap()
    .proposed_snapshot()
    .clone();
    let first = DynamicIdentity::new("initiator");
    let second = DynamicIdentity::new("recipient");
    let revoked_approver = DynamicIdentity::new("revoked-approver");
    let world = runtime(
        BankWorldSeed::new(revoked)
            .principal(BankPrincipalSeed::enabled(
                id(BankPrincipalId::new, 1),
                first.external(),
            ))
            .principal(BankPrincipalSeed::enabled(
                id(BankPrincipalId::new, 2),
                second.external(),
            ))
            .principal(BankPrincipalSeed::enabled(
                id(BankPrincipalId::new, 3),
                revoked_approver.external(),
            )),
    );
    assert_authorization_denied(
        publish(
            &world,
            &revoked_approver,
            ApprovePayment {
                payment,
                approver: id(BankPrincipalId::new, 3),
            },
            "revoked",
        ),
        WorthQueryOperationAuthorizationDenialKind::CapabilityGrantMissing,
    );
}

#[test]
fn runtime_initiation_grants_its_workflow_to_every_source_approver_at_the_ceiling() {
    let approvers = u64::try_from(MAX_PAYMENT_APPROVAL_GRANTEES).unwrap();
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
    let scope = request_scope();
    let initiated = world
        .runtime
        .request(&initiator, &scope)
        .mutate(InitiateBusinessPayment {
            business: id(BusinessId::new, 1),
            from: source,
            recipient: id(BankPrincipalId::new, 2),
            amount: Money::from_minor(700).unwrap(),
        })
        .idempotency(&key("crowded-initiation"))
        .execute_in_program(world.runtime.application_program())
        .expect("the initiation reaches its handler");
    let WorthQueryApplicationMutationOutcome::Committed { result, .. } = initiated else {
        panic!("an initiation within the approver ceiling commits: {initiated:?}");
    };
    for principal in 3..3 + approvers {
        let identity = &identities[usize::try_from(principal - 1).unwrap()];
        let published = publish(
            &world,
            identity,
            ApprovePayment {
                payment: result.payment,
                approver: id(BankPrincipalId::new, principal),
            },
            &format!("crowded-approver-{principal}"),
        );
        // The first approver publishes; later ones pass authorization and
        // then meet the already-published definition.
        assert!(
            if principal == 3 {
                matches!(
                    published,
                    Ok(WorkflowDefinitionPublicationOutcome::Published(_))
                )
            } else {
                published.is_ok()
            },
            "approver {principal} holds the new payment's workflow grant: {published:?}"
        );
    }
}

fn authenticate(
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

fn publish(
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

fn assert_authorization_denied(
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
