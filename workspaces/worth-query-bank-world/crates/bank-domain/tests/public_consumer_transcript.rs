use bank_domain::model::{
    AccountAuthorizationId, AccountId, BankPrincipalId, CustomerRole, JournalEntryId, Money,
    PaymentId, PostingId, SignedMoney, USD,
};
use bank_domain::schema::*;
use worth_query_decl::facade::application_schema::{
    ApplicationEncodedScalarValue, ApplicationFieldRef, ApplicationScalarValueBinding,
    ApplicationSchemaAuthoringDenialKind, EqualityPredicate, ReadWrite,
};
use worth_query_host::facade::domain::{
    WorthQueryInstallationAdmissionProfile, WorthQueryInstallationGeneration,
    WorthQueryInstallationRuntimeIdentity, WorthQueryInstalledApplicationSchema,
    WorthQueryInstalledPackageIndex, WorthQueryPortableDomainIdentity,
    WorthQueryPortableDomainPackage,
};

#[test]
fn installed_read_and_directional_traversal_authoring_are_usable() {
    let (index, bank) = installed_bank();
    let read = bank
        .query(Account::reference())
        .project(AccountIdentity::reference())
        .project(AccountDisplayName::reference())
        .project(AccountingRevision::reference())
        .where_equal(
            Status::reference(),
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
        .project(AccountIdentity::reference())
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
            PostingAmount::reference(),
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
            PaymentStatusField::reference(),
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
            AuthorizationRole::reference(),
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
fn forged_field_capability_type_and_currency_are_denied_independently() {
    let (_, bank) = installed_bank();
    let (from, recipient, amount) = transfer_values();
    let forged_write = ApplicationFieldRef::<
        BankSchema,
        Posting,
        PostingIdentity,
        PostingIdentityField,
        bank_domain::model::PostingId,
        ReadWrite,
        EqualityPredicate,
    >::from_schema_types();
    let denial = bank
        .operation(SendMoneyOperation::reference())
        .input(SendMoney {
            from,
            recipient,
            amount,
        })
        .set(
            forged_write,
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

fn assert_forged_currency_denied(bank: &WorthQueryInstalledApplicationSchema<BankSchema>) {
    let field = ApplicationFieldRef::<
        BankSchema,
        Posting,
        PostingValue,
        PostingAmount,
        SignedMoney<USD>,
        ReadWrite,
        worth_query_decl::facade::application_schema::NoEqualityPredicate,
    >::from_schema_types();
    let denial = bank
        .query(Posting::reference())
        .project(field)
        .build()
        .unwrap_err();
    assert_denial(
        denial.kind(),
        ApplicationSchemaAuthoringDenialKind::FieldUnitMismatch,
    );
}

fn installed_bank() -> (
    WorthQueryInstalledPackageIndex,
    WorthQueryInstalledApplicationSchema<BankSchema>,
) {
    let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        "WORTH.bank",
        1,
        0,
    ))
    .application_schema(BankSchema::declaration().unwrap())
    .validate()
    .unwrap();
    let admitted = WorthQueryInstallationAdmissionProfile::new("support-v1", "config-v1")
        .admit(package)
        .unwrap();
    let index = WorthQueryInstalledPackageIndex::build(
        WorthQueryInstallationRuntimeIdentity::fresh(),
        WorthQueryInstallationGeneration::initial(),
        [admitted],
    )
    .unwrap();
    let bank = index
        .bind_application_schema(BankSchema::declaration().unwrap())
        .unwrap();
    (index, bank)
}

fn transfer_values() -> (AccountId, BankPrincipalId, Money<USD>) {
    (
        AccountId::new(1).unwrap(),
        BankPrincipalId::new(2).unwrap(),
        Money::<USD>::from_minor(1_250).unwrap(),
    )
}

fn encoded<Binding>(value: Binding::Value) -> ApplicationEncodedScalarValue<Binding>
where
    Binding: ApplicationScalarValueBinding,
{
    ApplicationEncodedScalarValue::<Binding>::try_new(value).unwrap()
}

fn assert_denial(
    actual: ApplicationSchemaAuthoringDenialKind,
    expected: ApplicationSchemaAuthoringDenialKind,
) {
    assert_eq!(actual, expected);
}
