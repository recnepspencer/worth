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

    assert_program_committed::<CreatePersonalAccount>(
        execute!(
            fixture,
            teller,
            mutations::create_personal_account(CreatePersonalAccount {
                institution: fixture.institution,
                owner: principal_id(STRANGER),
                display_name: AccountName::new("Stranger account").unwrap(),
            }),
            "create-personal",
        ),
        false,
    );
    let created_accounts = fixture
        .world
        .runtime
        .query(queries::accounts())
        .as_principal(&stranger)
        .controls(read_controls())
        .execute()
        .expect("the created account must be query-visible");
    let created_personal = created_accounts.rows()[0].id();
    assert_program_committed::<CreateBusinessAccount>(
        execute!(
            fixture,
            teller,
            mutations::create_business_account(CreateBusinessAccount {
                institution: fixture.institution,
                business: id(BusinessId::new, 2),
                display_name: AccountName::new("Second business").unwrap(),
            }),
            "create-business",
        ),
        false,
    );
    assert_program_committed::<ApplyOpeningFunding>(
        execute!(
            fixture,
            teller,
            mutations::apply_opening_funding(ApplyOpeningFunding {
                institution: fixture.institution,
                account: created_personal,
                amount: Money::from_minor(3_000).unwrap(),
            }),
            "opening-funding",
        ),
        true,
    );
    assert_program_committed::<Deposit>(
        execute!(
            fixture,
            teller,
            mutations::deposit(Deposit {
                institution: fixture.institution,
                account: fixture.recipient_account,
                amount: Money::from_minor(500).unwrap(),
            }),
            "deposit",
        ),
        true,
    );
    assert_program_committed::<Withdraw>(
        execute!(
            fixture,
            teller,
            mutations::withdraw(Withdraw {
                institution: fixture.institution,
                account: fixture.recipient_account,
                amount: Money::from_minor(100).unwrap(),
            }),
            "withdraw",
        ),
        true,
    );
    let send = SendMoney {
        from: fixture.personal_account,
        recipient: principal_id(RECIPIENT),
        amount: Money::from_minor(250).unwrap(),
    };
    assert_program_committed::<SendMoney>(
        execute!(fixture, owner, mutations::send_money(send.clone()), "send",),
        true,
    );
    let retry = execute!(fixture, owner, mutations::send_money(send), "send");
    let Ok(WorthQueryApplicationMutationOutcome::AlreadyCommitted(receipt)) = retry else {
        panic!("equivalent public retry must recover the commit: {retry:?}");
    };
    assert_eq!(
        receipt.terminal().kind(),
        WorthQueryApplicationCommitTerminalKind::Recovered
    );
    assert_eq!(receipt.terminal().attempt_resources_released(), None);

    let initiation = execute!(
        fixture,
        owner,
        mutations::initiate_business_payment(InitiateBusinessPayment {
            business: id(BusinessId::new, 1),
            from: fixture.business_account,
            recipient: principal_id(RECIPIENT),
            amount: Money::from_minor(300).unwrap(),
        }),
        "initiate",
    );
    assert!(initiation.continuation().is_some());
    assert_program_committed::<InitiateBusinessPayment>(initiation.into_execution(), false);
    let available = account_activity(&fixture, &approver, fixture.business_account)
        .into_iter()
        .map(|item| item.amount().minor_units())
        .sum::<i64>();
    assert!(available >= 900);
    let approval = execute!(
        fixture,
        approver,
        mutations::approve_payment(ApprovePayment {
            payment: fixture.payment,
            approver: principal_id(APPROVER),
        }),
        "approve",
    );
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
    assert_program_committed::<RejectPayment>(
        execute!(
            fixture,
            approver,
            mutations::reject_payment(RejectPayment {
                payment: pending.id(),
                rejecting_principal: principal_id(APPROVER),
            }),
            "reject",
        ),
        false,
    );

    assert_program_committed::<GrantAccountAuthorization>(
        execute!(
            fixture,
            owner,
            mutations::grant_account_access(GrantAccountAuthorization {
                account: fixture.personal_account,
                principal: principal_id(STRANGER),
                role: CustomerRole::Viewer,
            }),
            "grant",
        ),
        false,
    );
    let authorization = authorized_users(&fixture, &owner)
        .into_iter()
        .find(|user| user.principal() == principal_id(STRANGER))
        .expect("granted authorization must be query-visible")
        .authorization();
    assert_program_committed::<RevokeAccountAuthorization>(
        execute!(
            fixture,
            owner,
            mutations::revoke_account_access(RevokeAccountAuthorization {
                account: fixture.personal_account,
                authorization,
            }),
            "revoke",
        ),
        false,
    );

    let journal = account_activity(&fixture, &recipient, fixture.recipient_account)
        .into_iter()
        .find(|item| item.purpose() == PostingPurpose::Deposit)
        .expect("deposit journal must be query-visible")
        .journal();
    assert_program_committed::<ReverseJournal>(
        execute!(
            fixture,
            teller,
            mutations::reverse_journal(ReverseJournal {
                institution: id(InstitutionId::new, 1),
                journal,
                reason: ReversalReason::OperatorCorrection,
            }),
            "reverse",
        ),
        true,
    );
}
