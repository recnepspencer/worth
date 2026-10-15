use super::*;

#[test]
fn public_consumer_executes_ordinary_mutation_families_without_bypassing_workflow_approval() {
    let fixture = ordinary_read_world("ordinary-mutations", 0);
    let owner = fixture.authenticate(OWNER);
    let recipient = fixture.authenticate(RECIPIENT);
    let approver = fixture.authenticate(APPROVER);
    let teller = fixture.authenticate(TELLER);
    let stranger = fixture.authenticate(STRANGER);
    let discovery = fixture
        .world
        .runtime
        .query(queries::accounts())
        .as_principal(&stranger)
        .controls(read_controls())
        .execute()
        .expect("the prospective account owner must be discoverable");
    assert!(discovery.rows().is_empty());

    {
        let scope = request_scope();
        let request = fixture.world.runtime.request(&teller, &scope);
        let operation = fixture
            .world
            .runtime
            .mutate(mutations::create_personal_account(CreatePersonalAccount {
                institution: fixture.institution,
                owner: principal_id(STRANGER),
                display_name: AccountName::new("Stranger account").unwrap(),
            }))
            .as_principal(&teller)
            .controls(BankMutationControls::new(
                scope.clone(),
                key("create-personal"),
            ))
            .retain_operation()
            .unwrap_or_else(|(_, cause)| panic!("declared plain action: {cause:?}"));
        let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Completed(
            done,
        ) = operation.advance(&request)
        else {
            panic!("fresh plain completion")
        };
        let worth_query_host::facade::application_entry::WorthQuerySourceCompletion::Fresh {
            receipt,
            result,
            ..
        } = done.evidence()
        else {
            panic!("fresh handler provenance")
        };
        let work = receipt.mutation_work().unwrap();
        assert!(
            work.decision_fact_count() > 0
                && work.proposed_fact_count() > 0
                && work.relational_invariant_execution_count() > 0
        );
        let result = *result;
        done.close(&request)
            .unwrap_or_else(|(_, cause)| panic!("plain close: {cause:?}"));
        result
    };
    let created_accounts = fixture
        .world
        .runtime
        .query(queries::accounts())
        .as_principal(&stranger)
        .controls(read_controls())
        .execute()
        .expect("the created account must be query-visible");
    let created_personal = created_accounts.rows()[0].id();
    {
        let scope = request_scope();
        let request = fixture.world.runtime.request(&teller, &scope);
        let operation = fixture
            .world
            .runtime
            .mutate(mutations::create_business_account(CreateBusinessAccount {
                institution: fixture.institution,
                business: id(BusinessId::new, 2),
                display_name: AccountName::new("Second business").unwrap(),
            }))
            .as_principal(&teller)
            .controls(BankMutationControls::new(
                scope.clone(),
                key("create-business"),
            ))
            .retain_operation()
            .unwrap_or_else(|(_, cause)| panic!("declared plain action: {cause:?}"));
        let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Completed(
            done,
        ) = operation.advance(&request)
        else {
            panic!("fresh plain completion")
        };
        let worth_query_host::facade::application_entry::WorthQuerySourceCompletion::Fresh {
            receipt,
            result,
            ..
        } = done.evidence()
        else {
            panic!("fresh handler provenance")
        };
        let work = receipt.mutation_work().unwrap();
        assert!(
            work.decision_fact_count() > 0
                && work.proposed_fact_count() > 0
                && work.relational_invariant_execution_count() > 0
        );
        let result = *result;
        done.close(&request)
            .unwrap_or_else(|(_, cause)| panic!("plain close: {cause:?}"));
        result
    };
    retained_money::fund_opening(&fixture, &teller, created_personal);
    super::money_families::exercise(&fixture);

    let initiation = {
        let scope = request_scope();
        let request = fixture.world.runtime.request(&owner, &scope);
        let operation = fixture
            .world
            .runtime
            .mutate(mutations::initiate_business_payment(
                InitiateBusinessPayment {
                    business: id(BusinessId::new, 1),
                    from: fixture.business_account,
                    recipient: principal_id(RECIPIENT),
                    amount: Money::from_minor(300).unwrap(),
                },
            ))
            .as_principal(&owner)
            .controls(BankMutationControls::new(scope.clone(), key("initiate")))
            .retain_operation()
            .unwrap_or_else(|(_, cause)| panic!("declared plain action: {cause:?}"));
        let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Completed(
            done,
        ) = operation.advance(&request)
        else {
            panic!("fresh plain completion")
        };
        let worth_query_host::facade::application_entry::WorthQuerySourceCompletion::Fresh {
            receipt,
            result,
            ..
        } = done.evidence()
        else {
            panic!("fresh handler provenance")
        };
        let work = receipt.mutation_work().unwrap();
        assert!(
            work.decision_fact_count() > 0
                && work.proposed_fact_count() > 0
                && work.relational_invariant_execution_count() > 0
        );
        let result = *result;
        done.close(&request)
            .unwrap_or_else(|(_, cause)| panic!("plain close: {cause:?}"));
        result
    };
    assert!(pending_payments(&fixture, &approver)
        .iter()
        .any(|payment| payment.id() == initiation.payment));
    let available = account_activity(&fixture, &approver, fixture.business_account)
        .into_iter()
        .map(|item| item.amount().minor_units())
        .sum::<i64>();
    assert!(available >= 900);
    // The typed ordinary surface offers no approval; a raw request for one
    // is refused before it can bypass the workflow.
    let approval_scope = request_scope();
    let approval = fixture
        .world
        .runtime
        .request(&approver, &approval_scope)
        .advanced()
        .mutate(ApprovePayment {
            payment: fixture.payment,
            approver: principal_id(APPROVER),
        })
        .idempotency(&key("approve"))
        .execute(worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation);
    assert!(matches!(
        approval,
        Err(WorthQueryApplicationRequestMutationDenial::RequiresWorkflowTransition)
    ));
    let pending = pending_payments(&fixture, &approver);
    assert!(pending
        .iter()
        .any(|payment| payment.id() == fixture.payment));
    let pending = pending
        .iter()
        .find(|payment| payment.id() != fixture.payment)
        .expect("the newly initiated payment remains pending");
    {
        let scope = request_scope();
        let request = fixture.world.runtime.request(&approver, &scope);
        let operation = fixture
            .world
            .runtime
            .mutate(mutations::reject_payment(RejectPayment {
                payment: pending.id(),
                rejecting_principal: principal_id(APPROVER),
            }))
            .as_principal(&approver)
            .controls(BankMutationControls::new(scope.clone(), key("reject")))
            .retain_operation()
            .unwrap_or_else(|(_, cause)| panic!("declared plain action: {cause:?}"));
        let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Completed(
            done,
        ) = operation.advance(&request)
        else {
            panic!("fresh plain completion")
        };
        let worth_query_host::facade::application_entry::WorthQuerySourceCompletion::Fresh {
            receipt,
            result,
            ..
        } = done.evidence()
        else {
            panic!("fresh handler provenance")
        };
        let work = receipt.mutation_work().unwrap();
        assert!(
            work.decision_fact_count() > 0
                && work.proposed_fact_count() > 0
                && work.relational_invariant_execution_count() > 0
        );
        let result = *result;
        done.close(&request)
            .unwrap_or_else(|(_, cause)| panic!("plain close: {cause:?}"));
        result
    };

    {
        let scope = request_scope();
        let request = fixture.world.runtime.request(&owner, &scope);
        let operation = fixture
            .world
            .runtime
            .mutate(mutations::grant_account_access(GrantAccountAuthorization {
                account: fixture.personal_account,
                principal: principal_id(STRANGER),
                role: CustomerRole::Viewer,
            }))
            .as_principal(&owner)
            .controls(BankMutationControls::new(scope.clone(), key("grant")))
            .retain_operation()
            .unwrap_or_else(|(_, cause)| panic!("declared plain action: {cause:?}"));
        let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Completed(
            done,
        ) = operation.advance(&request)
        else {
            panic!("fresh plain completion")
        };
        let worth_query_host::facade::application_entry::WorthQuerySourceCompletion::Fresh {
            receipt,
            result,
            ..
        } = done.evidence()
        else {
            panic!("fresh handler provenance")
        };
        let work = receipt.mutation_work().unwrap();
        assert!(
            work.decision_fact_count() > 0
                && work.proposed_fact_count() > 0
                && work.relational_invariant_execution_count() > 0
        );
        let result = *result;
        done.close(&request)
            .unwrap_or_else(|(_, cause)| panic!("plain close: {cause:?}"));
        result
    };
    let authorization = authorized_users(&fixture, &owner)
        .into_iter()
        .find(|user| user.principal() == principal_id(STRANGER))
        .expect("granted authorization must be query-visible")
        .authorization();
    {
        let scope = request_scope();
        let request = fixture.world.runtime.request(&owner, &scope);
        let operation = fixture
            .world
            .runtime
            .mutate(mutations::revoke_account_access(
                RevokeAccountAuthorization {
                    account: fixture.personal_account,
                    authorization,
                },
            ))
            .as_principal(&owner)
            .controls(BankMutationControls::new(scope.clone(), key("revoke")))
            .retain_operation()
            .unwrap_or_else(|(_, cause)| panic!("declared plain action: {cause:?}"));
        let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Completed(
            done,
        ) = operation.advance(&request)
        else {
            panic!("fresh plain completion")
        };
        let worth_query_host::facade::application_entry::WorthQuerySourceCompletion::Fresh {
            receipt,
            result,
            ..
        } = done.evidence()
        else {
            panic!("fresh handler provenance")
        };
        let work = receipt.mutation_work().unwrap();
        assert!(
            work.decision_fact_count() > 0
                && work.proposed_fact_count() > 0
                && work.relational_invariant_execution_count() > 0
        );
        let result = *result;
        done.close(&request)
            .unwrap_or_else(|(_, cause)| panic!("plain close: {cause:?}"));
        result
    };

    let journal = account_activity(&fixture, &recipient, fixture.recipient_account)
        .into_iter()
        .find(|item| item.purpose() == PostingPurpose::Deposit)
        .expect("deposit journal must be query-visible")
        .journal();
    retained_money::reverse_deposit(&fixture, &teller, journal);
}

#[path = "program_families/retained_money.rs"]
mod retained_money;
