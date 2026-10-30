pub(crate) use worth_query_bank_testkit_domain::facade::domain_contracts::estate_capability_installation::disburse_estate::*;

#[test]
fn disbursement_installs_exact_effect_integrity_reads_and_money_program() {
    let (_index, bank) = installed_bank(WorthQueryInstallationRuntimeIdentity::fresh());
    let operation = bank
        .installed_operation(DisburseEstateOperation::reference())
        .expect("estate disbursement must install one executable accounting program");

    assert_eq!(operation.contracts().projection_work_budget(), Some(192));
    assert_eq!(
        installed_read_targets(operation.contracts()),
        expected_reads()
    );
    assert_eq!(
        installed_program_targets(operation.contracts()),
        expected_program()
    );
}
