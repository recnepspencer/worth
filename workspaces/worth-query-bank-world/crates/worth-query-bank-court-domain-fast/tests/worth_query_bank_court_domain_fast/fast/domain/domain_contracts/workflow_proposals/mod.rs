pub(crate) use worth_query_bank_worlds::facade::domain::domain_contracts::workflow_proposals::*;

#[test]
fn business_payment_requires_a_distinct_actor_and_one_decision() {
    let world = prepared_business_world();
    let source = world.business_account(id(BusinessId::new, 1)).unwrap();
    let initiated = BankProposalEngine::prepare_initiate_business_payment(
        &world,
        binding(4),
        &key("initiate"),
        id(BankPrincipalId::new, 1),
        &InitiateBusinessPayment {
            business: id(BusinessId::new, 1),
            from: source,
            recipient: id(BankPrincipalId::new, 2),
            amount: Money::from_minor(5_000).unwrap(),
        },
    )
    .unwrap()
    .proposed_snapshot()
    .clone();
    let payment = initiated.payments().next().unwrap();
    assert_eq!(payment.status(), PaymentStatus::ApprovalRequired);

    let self_approval = ApprovePayment {
        payment: payment.id(),
        approver: id(BankPrincipalId::new, 1),
    };
    assert_eq!(
        BankProposalEngine::prepare_approve_payment(
            &initiated,
            binding(5),
            &key("self"),
            &self_approval,
        )
        .err(),
        Some(BankProposalDenial::SelfApproval)
    );

    let approved = BankProposalEngine::prepare_approve_payment(
        &initiated,
        binding(5),
        &key("approve"),
        &ApprovePayment {
            payment: payment.id(),
            approver: id(BankPrincipalId::new, 3),
        },
    )
    .unwrap()
    .proposed_snapshot()
    .clone();
    assert_eq!(
        approved.payment(payment.id()).unwrap().status(),
        PaymentStatus::Committed
    );
    assert!(matches!(
        BankProposalEngine::prepare_reject_payment(
            &approved,
            binding(5),
            &key("late-reject"),
            &RejectPayment {
                payment: payment.id(),
                rejecting_principal: id(BankPrincipalId::new, 3),
            },
        )
        .err(),
        Some(BankProposalDenial::PaymentAlreadyDecided(_))
    ));
}

#[test]
fn account_authorization_grant_and_revoke_are_typed_effects() {
    let world = prepared_business_world();
    let account = world.business_account(id(BusinessId::new, 1)).unwrap();
    let granted = BankProposalEngine::prepare_grant_account_authorization(
        &world,
        binding(6),
        &key("grant"),
        &GrantAccountAuthorization {
            account,
            principal: id(BankPrincipalId::new, 3),
            role: CustomerRole::Approver,
        },
    )
    .unwrap()
    .proposed_snapshot()
    .clone();
    let authorization = *granted.authorizations().next().unwrap();
    let revoked = BankProposalEngine::prepare_revoke_account_authorization(
        &granted,
        binding(6),
        &key("revoke"),
        &RevokeAccountAuthorization {
            account,
            authorization: authorization.id(),
        },
    )
    .unwrap();
    assert_eq!(revoked.proposed_snapshot().authorizations().len(), 0);
}

#[test]
fn idempotency_intent_is_stable_and_detects_binding_or_payload_drift() {
    let world = fixture();
    let input = CreatePersonalAccount {
        institution: id(InstitutionId::new, 1),
        owner: id(BankPrincipalId::new, 1),
        display_name: AccountName::new("Daily").unwrap(),
    };
    let first = BankProposalEngine::prepare_create_personal_account(
        &world,
        binding(7),
        &key("same-key"),
        &input,
    )
    .unwrap();
    let equivalent = BankProposalEngine::prepare_create_personal_account(
        &world,
        binding(7),
        &key("same-key"),
        &input,
    )
    .unwrap();
    let binding_drift = BankProposalEngine::prepare_create_personal_account(
        &world,
        binding(8),
        &key("same-key"),
        &input,
    )
    .unwrap();
    let payload_drift = BankProposalEngine::prepare_create_personal_account(
        &world,
        binding(7),
        &key("same-key"),
        &CreatePersonalAccount {
            display_name: AccountName::new("Different").unwrap(),
            ..input
        },
    )
    .unwrap();
    assert_eq!(first.idempotency_intent(), equivalent.idempotency_intent());
    assert_ne!(
        first.idempotency_intent(),
        binding_drift.idempotency_intent()
    );
    assert_ne!(
        first.idempotency_intent(),
        payload_drift.idempotency_intent()
    );
    assert_eq!(
        first.idempotency_key_identity(),
        payload_drift.idempotency_key_identity(),
        "the provider must detect one key reused for a different semantic payload"
    );
    assert_ne!(
        first.idempotency_key_identity(),
        binding_drift.idempotency_key_identity(),
        "principal, operation, and scope binding must partition key identity"
    );
}

#[test]
fn initiation_grants_its_approval_workflow_to_each_source_approver_only() {
    let approvers = 3..3 + MAX_PAYMENT_APPROVAL_GRANTEES as u64;
    let world = with_authorizations(
        prepared_business_world(),
        approvers
            .clone()
            .map(|principal| (principal, CustomerRole::Approver))
            .chain([(12, CustomerRole::Viewer)]),
    );
    let (payment, grantees) = decide_initiation(&world).unwrap().into_parts();
    assert_eq!(payment.status(), PaymentStatus::ApprovalRequired);
    assert_eq!(
        grantees,
        approvers
            .map(|principal| id(BankPrincipalId::new, principal))
            .collect::<Vec<_>>(),
        "every source-account approver, and no viewer, holds the new workflow grant"
    );
}

#[test]
fn initiation_refuses_more_approvers_than_its_candidate_ceiling_grants() {
    let world = with_authorizations(
        prepared_business_world(),
        (3..=3 + MAX_PAYMENT_APPROVAL_GRANTEES as u64)
            .map(|principal| (principal, CustomerRole::Approver)),
    );
    assert_eq!(
        decide_initiation(&world),
        Err(BankProposalDenial::TooManyPaymentApprovers)
    );
}
