#![allow(
    clippy::result_large_err,
    reason = "Authorization courts preserve complete native workflow refusal evidence"
)]

pub(crate) use worth_query_bank_testkit_server::facade::business_authorization::*;

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
        WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::ExplicitDenyRuleMatched,
        ),
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
        WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::CapabilityGrantMissing,
        ),
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
        WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::CapabilityGrantMissing,
        ),
    );
}

#[test]
fn runtime_initiation_grants_its_workflow_to_every_source_approver_at_the_ceiling() {
    let approvers = u64::try_from(MAX_PAYMENT_APPROVAL_GRANTEES).unwrap();
    let (world, identities, initiated) = crowded_initiation(approvers);
    let actor = authenticate(&world, &identities[0]);
    let scope = request_scope();
    let request = world.runtime.request(&actor, &scope);
    let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Completed(done) =
        initiated.advance(&request)
    else {
        panic!("within ceiling completes")
    };
    let worth_query_host::facade::application_entry::WorthQuerySourceCompletion::Fresh {
        result,
        ..
    } = done.evidence()
    else {
        panic!("fresh result")
    };
    let pending = bank_server::BankPendingPaymentContinuation::from_payment_id(result.payment);
    done.close(&request)
        .unwrap_or_else(|(_, cause)| panic!("initiation close: {cause:?}"));
    for principal in 3..3 + approvers {
        let identity = &identities[usize::try_from(principal - 1).unwrap()];
        let published = publish(
            &world,
            identity,
            ApprovePayment {
                payment: pending.payment_id(),
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

#[test]
fn runtime_initiation_past_the_approver_ceiling_is_domain_denied() {
    let approvers = u64::try_from(MAX_PAYMENT_APPROVAL_GRANTEES).unwrap() + 1;
    let (world, identities, initiated) = crowded_initiation(approvers);
    let actor = authenticate(&world, &identities[0]);
    let scope = request_scope();
    let request = world.runtime.request(&actor, &scope);
    let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Terminal(done) =
        initiated.advance(&request)
    else {
        panic!("over ceiling domain denial")
    };
    assert!(matches!(
        done.native_outcome(),
        Some(WorthQueryApplicationMutationOutcome::DomainDenied(
            BankProposalDenial::TooManyPaymentApprovers
        ))
    ));
    done.close(&request)
        .unwrap_or_else(|(_, cause)| panic!("denied close: {cause:?}"));
}
