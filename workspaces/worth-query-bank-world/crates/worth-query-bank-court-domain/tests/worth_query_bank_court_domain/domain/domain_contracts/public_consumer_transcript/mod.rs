pub(crate) use worth_query_bank_testkit_domain::facade::domain_contracts::public_consumer_transcript::*;

#[test]
fn installed_read_and_directional_traversal_authoring_are_usable() {
    let (index, bank) = installed_bank();
    let read = bank
        .query(Account::reference())
        .project(AccountIdentity::at("AccountIdentity"))
        .project(AccountDisplayName::at("AccountDisplayName"))
        .project(AccountingRevision::at("AccountingRevision"))
        .where_equal(
            Status::at("Status"),
            encoded::<AccountStatusBinding>(AccountStatus::Open),
        )
        .build()
        .unwrap();
    assert_eq!(read.entity(), "Account");
    assert_eq!(read.projections().len(), 3);
    assert_eq!(
        read.binding().unwrap().runtime_ordinal(),
        index.runtime_ordinal()
    );

    let owned = bank
        .query(Principal::reference())
        .traverse(PersonalOwner::reference())
        .project(AccountIdentity::at("AccountIdentity"))
        .build()
        .unwrap();
    assert_eq!(owned.current_entity(), "Account");
    assert_eq!(owned.traversals()[0].relation(), "PersonalOwner");
    index.validate_application_schema(&bank).unwrap();
}

#[test]
fn installed_money_mutation_and_effect_program_are_usable() {
    let (_, bank) = installed_bank();
    let (from, recipient, amount) = transfer_values();
    let mutation = bank
        .operation(SendMoneyOperation::reference())
        .input(SendMoney {
            from,
            recipient,
            amount,
        })
        .create(JournalEntry::reference())
        .create(Posting::reference())
        .set(
            PostingAmount::at("PostingAmount"),
            encoded::<SignedUsdMoneyBinding>(SignedMoney::<USD>::from_minor(-1_250)),
        )
        .build()
        .unwrap();
    assert_eq!(mutation.creates(), &["JournalEntry", "Posting"]);
    assert_eq!(
        mutation.binding().unwrap().schema_identity(),
        bank.binding_identity().schema_identity()
    );

    let effects = bank
        .effects(SendMoneyOperation::reference())
        .emit(
            AccountActivityEffect::reference(),
            ActivityEvent {
                account: from,
                journal: JournalEntryId::new(1).unwrap(),
                posting: PostingId::new(1).unwrap(),
                journal_sequence: 7,
            },
        )
        .build()
        .unwrap();
    assert_eq!(effects.effects().len(), 1);
}

#[test]
fn installed_approval_grant_and_revoke_programs_are_usable() {
    let (_, bank) = installed_bank();
    let (account, principal, _) = transfer_values();
    let approval = bank
        .operation(ApprovePaymentOperation::reference())
        .input(ApprovePayment {
            payment: PaymentId::new(9).unwrap(),
            approver: principal,
        })
        .create(Approval::reference())
        .link(PaymentApproval::reference())
        .link(ApprovalPrincipal::reference())
        .set(
            PaymentStatusField::at("PaymentStatusField"),
            encoded::<PaymentStatusBinding>(PaymentStatus::Committed),
        )
        .build()
        .unwrap();
    assert_eq!(approval.links().len(), 2);

    let grant = bank
        .operation(GrantAccountAuthorizationOperation::reference())
        .input(GrantAccountAuthorization {
            account,
            principal,
            role: CustomerRole::Viewer,
        })
        .create(AccountAuthorization::reference())
        .link(AccountAuthorizedUser::reference())
        .link(AuthorizationAccount::reference())
        .set(
            AuthorizationRole::at("AuthorizationRole"),
            encoded::<CustomerRoleBinding>(CustomerRole::Viewer),
        )
        .build()
        .unwrap();
    assert_eq!(grant.links().len(), 2);

    let revoke = bank
        .operation(RevokeAccountAuthorizationOperation::reference())
        .input(RevokeAccountAuthorization {
            account: AccountId::new(5).unwrap(),
            authorization: AccountAuthorizationId::new(4).unwrap(),
        })
        .unlink(AccountAuthorizedUser::reference())
        .unlink(AuthorizationAccount::reference())
        .delete(AccountAuthorization::reference())
        .build()
        .unwrap();
    assert_eq!(revoke.unlinks().len(), 2);
    assert_eq!(revoke.deletes(), &["AccountAuthorization"]);
}

#[test]
fn read_only_fields_and_foreign_currency_bindings_are_denied_independently() {
    let (_, bank) = installed_bank();
    let (from, recipient, amount) = transfer_values();
    let read_only = PostingIdentityField::at("PostingIdentityField");
    let denial = bank
        .operation(SendMoneyOperation::reference())
        .input(SendMoney {
            from,
            recipient,
            amount,
        })
        .set(
            read_only,
            encoded::<PostingIdBinding>(bank_domain::model::PostingId::new(99).unwrap()),
        )
        .build()
        .unwrap_err();
    assert_denial(
        denial.kind(),
        ApplicationSchemaAuthoringDenialKind::FieldNotWritable,
    );

    assert_forged_currency_denied(&bank);
}
