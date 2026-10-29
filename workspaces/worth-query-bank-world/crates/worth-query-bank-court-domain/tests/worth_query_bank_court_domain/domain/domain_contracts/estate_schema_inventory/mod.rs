pub(crate) use worth_query_bank_testkit_domain::facade::domain_contracts::estate_schema_inventory::*;

#[test]
fn required_estate_topology_and_policy_contributions_are_present() {
    let declaration = BankSchema::declaration().unwrap();
    let members = declaration.erased().members();
    assert_required_subset_present(
        members.iter().filter_map(entity_name).collect(),
        estate_entities(),
    );
    assert_required_subset_present(
        members.iter().filter_map(relation_name).collect(),
        estate_relations(),
    );
    assert_required_subset_present(
        members.iter().filter_map(policy_name).collect(),
        estate_policies(),
    );
    assert_eq!(
        members
            .iter()
            .filter_map(capability_context_name)
            .collect::<BTreeSet<_>>(),
        [
            "ApprovedBusinessPaymentControlContext",
            "EstateActionContext",
            "ProgramDeploymentContext",
        ]
        .into_iter()
        .collect()
    );
    assert_eq!(
        members
            .iter()
            .filter_map(capability_context_slot_name)
            .collect::<BTreeSet<_>>(),
        [
            "EstateEmergencyAccessSlot",
            "EstateLegalAuthoritySlot",
            "EstateMandatoryReviewSlot",
        ]
        .into_iter()
        .collect()
    );
    assert_eq!(
        members
            .iter()
            .filter_map(capability_provenance_name)
            .collect::<BTreeSet<_>>(),
        [
            "ApprovedBusinessPaymentGrantProvenance",
            "EstateGrantChainProvenance",
            "ProgramDeploymentProvenance",
        ]
        .into_iter()
        .collect()
    );
}

#[test]
fn estate_capabilities_and_installed_phase_seven_programs_are_declared() {
    let declaration = BankSchema::declaration().unwrap();
    let members = declaration.erased().members();
    assert_required_subset_present(
        members.iter().filter_map(operation_name).collect(),
        estate_operations(),
    );
    assert_eq!(
        members
            .iter()
            .filter_map(capability_name)
            .collect::<BTreeSet<_>>(),
        estate_capabilities()
    );
    let executable_operations = members
        .iter()
        .filter_map(operation_program_name)
        .collect::<BTreeSet<_>>();
    assert_eq!(
        executable_operations
            .intersection(&estate_operations())
            .copied()
            .collect::<BTreeSet<_>>(),
        [
            "ApproveEstateEmergencyAccessOperation",
            "CompleteEstateMandatoryReviewOperation",
            "DisburseEstateOperation",
            "FreezeEstateAccountOperation",
            "NotifyDeathEstateOperation",
            "OpenEstateCaseOperation",
            "RecognizeEstateExecutorOperation",
            "ReleaseEstateOperation",
            "RequestEstateEmergencyAccessOperation",
            "RestoreFrozenEstateAccountOperation",
            "RetransmitDeathNoticeEstateOperation",
            "RevokeEstateEmergencyAccessOperation",
        ]
        .into_iter()
        .collect(),
        "Phase 7.7 installs the exact emergency lifecycle and currently completed ordinary estate mutation programs"
    );

    assert_estate_sources_have_no_local_authority_lane();
}

#[test]
fn delegation_activation_keeps_bank_reads_without_an_application_effect_program() {
    let declaration = BankSchema::declaration().unwrap();
    let members = declaration.erased().members();
    assert!(members.iter().all(|member| !matches!(
        member,
        ApplicationSchemaMember::OperationProgram { operation, .. }
            if operation == "DelegateEstateCapabilityOperation"
    )));

    let reads = members
        .iter()
        .filter_map(delegation_activation_read)
        .collect::<BTreeSet<_>>();
    assert_eq!(
        reads,
        [
            ("field", "AccountIdentity"),
            ("field", "BranchIdentityField"),
            // The child id naming no grant is decided on.
            ("field", "CapabilityGrantIdentityField"),
            ("field", "EstateCaseIdentityField"),
            ("field", "InstitutionIdentityField"),
            ("relation", "BranchInstitution"),
            ("relation", "EstateAccount"),
            ("relation", "EstateBranch"),
        ]
        .into_iter()
        .collect()
    );
}
