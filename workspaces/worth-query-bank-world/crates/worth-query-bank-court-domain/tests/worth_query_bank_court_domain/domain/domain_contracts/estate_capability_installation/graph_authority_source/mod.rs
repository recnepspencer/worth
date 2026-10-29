pub(crate) use worth_query_bank_testkit_domain::facade::domain_contracts::estate_capability_installation::graph_authority_source::*;

#[test]
fn executable_metadata_cannot_select_capability_graph_authority() {
    let (_index, bank) = super::installed_bank(WorthQueryInstallationRuntimeIdentity::fresh());

    let executable_capability = bank
        .capability(
            RequestEstateEmergencyAccessCapability::reference(),
            RequestEstateEmergencyAccessOperation::reference(),
        )
        .unwrap();
    let executable_authority = bank
        .installed_operation_for_capability(&executable_capability)
        .unwrap();
    assert_capability_only_authority(
        executable_authority.graph_obligations(),
        executable_capability.identity(),
    );

    let descriptive_capability = bank
        .capability(
            ViewEstateAdministrationCapability::reference(),
            ViewRestrictedEstateOperation::reference(),
        )
        .unwrap();
    let descriptive_authority = bank
        .installed_operation_for_capability(&descriptive_capability)
        .unwrap();
    assert_capability_only_authority(
        descriptive_authority.graph_obligations(),
        descriptive_capability.identity(),
    );
}
