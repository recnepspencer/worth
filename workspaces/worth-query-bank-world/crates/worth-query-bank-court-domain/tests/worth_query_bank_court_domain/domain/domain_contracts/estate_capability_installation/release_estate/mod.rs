pub(crate) use worth_query_bank_testkit_domain::facade::domain_contracts::estate_capability_installation::release_estate::*;

#[test]
fn release_installs_exact_reads_and_effect() {
    let (_index, bank) = installed_bank(WorthQueryInstallationRuntimeIdentity::fresh());
    let release = bank
        .installed_operation(ReleaseEstateOperation::reference())
        .expect("release must install one exact executable program");
    assert_eq!(release.contracts().projection_work_budget(), Some(96));
    assert_eq!(
        installed_read_targets(release.contracts()),
        expected_release_reads()
    );
    assert_eq!(
        installed_program_targets(release.contracts()),
        [InstalledProgramTarget::Write {
            entity: "EstateCase".to_owned(),
            aspect: "EstateCaseRecord".to_owned(),
            path: path("EstateCaseStatusField"),
        }]
        .into_iter()
        .collect()
    );
}
