use bank_domain::schema::*;
use worth_query_host::facade::domain::{
    WorthQueryInstallationAdmissionProfile, WorthQueryInstallationGeneration,
    WorthQueryInstallationRuntimeIdentity, WorthQueryPortableDomainIdentity,
    WorthQueryPortableDomainPackage,
};

#[test]
fn every_bank_action_reserves_its_installed_validator_scope() {
    let declaration = BankSchema::declaration().expect("bank schema should declare");
    let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        "WORTH.bank",
        1,
        0,
    ))
    .application_schema(declaration.clone())
    .validate()
    .expect("bank package should validate");
    let admitted = WorthQueryInstallationAdmissionProfile::new("support-v1", "config-v1")
        .admit(package)
        .expect("bank package should admit");
    let index = worth_query_host::facade::domain::WorthQueryInstalledPackageIndex::build(
        WorthQueryInstallationRuntimeIdentity::fresh(),
        WorthQueryInstallationGeneration::initial(),
        [admitted],
    )
    .expect("bank package should install");
    let installed = index
        .bind_application_schema(declaration.clone())
        .expect("bank schema should bind");

    let mut mismatches = Vec::new();
    macro_rules! assert_exact_reservation {
        ($operation:ty) => {{
            let operation = installed
                .installed_operation(<$operation>::reference())
                .expect("bank action should install");
            let requirements = operation.contracts().invariant_execution().requirements();
            let semantic_scope = requirements
                .iter()
                .map(|requirement| requirement.max_state_facts())
                .max()
                .unwrap_or(0);
            let custom_work = requirements
                .iter()
                .filter_map(|requirement| requirement.application_invariant())
                .try_fold(0_u64, |total, invariant| {
                    total.checked_add(invariant.maximum_work_units().get())
                })
                .expect("installed invariant work should fit u64");
            let required = semantic_scope
                .checked_add(
                    usize::try_from(custom_work)
                        .expect("installed invariant work should fit the host"),
                )
                .expect("bank validator work should fit the host");
            let declared = declaration
                .member_provenance()
                .mutation_bindings()
                .iter()
                .find(|binding| binding.operation_name() == operation.operation())
                .expect("the installed action should retain its mutation binding")
                .candidates()
                .resources()
                .maximum_validator_work();
            if declared != required {
                mismatches.push((operation.operation().to_owned(), declared, required));
            }
        }};
    }

    assert_exact_reservation!(CreatePersonalAccountOperation);
    assert_exact_reservation!(CreateBusinessAccountOperation);
    assert_exact_reservation!(GrantAccountAuthorizationOperation);
    assert_exact_reservation!(RevokeAccountAuthorizationOperation);
    assert_exact_reservation!(ApplyOpeningFundingOperation);
    assert_exact_reservation!(DepositOperation);
    assert_exact_reservation!(WithdrawOperation);
    assert_exact_reservation!(SendMoneyOperation);
    assert_exact_reservation!(InitiateBusinessPaymentOperation);
    assert_exact_reservation!(ApprovePaymentOperation);
    assert_exact_reservation!(RejectPaymentOperation);
    assert_exact_reservation!(ReverseJournalOperation);
    assert_exact_reservation!(NotifyDeathEstateOperation);
    assert_exact_reservation!(RetransmitDeathNoticeEstateOperation);
    assert_exact_reservation!(FreezeEstateAccountOperation);
    assert_exact_reservation!(OpenEstateCaseOperation);
    assert_exact_reservation!(RecognizeEstateExecutorOperation);
    assert_exact_reservation!(ReleaseEstateOperation);
    assert_exact_reservation!(DisburseEstateOperation);
    assert!(
        mismatches.is_empty(),
        "validator work mismatch: {mismatches:?}"
    );
}
