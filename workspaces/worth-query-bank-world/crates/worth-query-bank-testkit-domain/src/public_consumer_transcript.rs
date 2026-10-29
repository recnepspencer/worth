pub use bank_domain::model::{
    AccountAuthorizationId, AccountId, BankPrincipalId, CustomerRole, JournalEntryId, Money,
    PaymentId, PostingId, SignedMoney, USD,
};
pub use bank_domain::schema::*;
pub use worth_query_decl::facade::application_schema::{
    ApplicationEncodedScalarValue, ApplicationFieldRef, ApplicationScalarValueBinding,
    ApplicationSchemaAuthoringDenialKind, EqualityPredicate, ReadWrite,
};
pub use worth_query_host::facade::domain::{
    WorthQueryInstallationAdmissionProfile, WorthQueryInstallationGeneration,
    WorthQueryInstallationRuntimeIdentity, WorthQueryInstalledApplicationSchema,
    WorthQueryInstalledPackageIndex, WorthQueryPortableDomainIdentity,
    WorthQueryPortableDomainPackage,
};

pub fn assert_forged_currency_denied(bank: &WorthQueryInstalledApplicationSchema<BankSchema>) {
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

pub fn installed_bank() -> (
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

pub fn transfer_values() -> (AccountId, BankPrincipalId, Money<USD>) {
    (
        AccountId::new(1).unwrap(),
        BankPrincipalId::new(2).unwrap(),
        Money::<USD>::from_minor(1_250).unwrap(),
    )
}

pub fn encoded<Binding>(value: Binding::Value) -> ApplicationEncodedScalarValue<Binding>
where
    Binding: ApplicationScalarValueBinding,
{
    ApplicationEncodedScalarValue::<Binding>::try_new(value).unwrap()
}

pub fn assert_denial(
    actual: ApplicationSchemaAuthoringDenialKind,
    expected: ApplicationSchemaAuthoringDenialKind,
) {
    assert_eq!(actual, expected);
}
