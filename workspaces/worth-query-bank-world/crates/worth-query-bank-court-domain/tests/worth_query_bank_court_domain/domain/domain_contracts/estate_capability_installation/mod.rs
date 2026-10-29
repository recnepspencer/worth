pub(crate) use worth_query_bank_testkit_domain::facade::domain_contracts::estate_capability_installation::*;

pub(crate) mod composition_contracts;

pub(crate) mod delegation_activation;

pub(crate) mod disburse_estate;

pub(crate) mod emergency_access_activity;

pub(crate) mod graph_authority_source;

pub(crate) mod release_estate;

pub(crate) mod transition_dimensions;

#[test]
fn every_estate_capability_installs_once_with_distinct_canonical_identity() {
    let (_index, bank) = installed_bank(WorthQueryInstallationRuntimeIdentity::fresh());
    let mut identities = BTreeSet::new();

    assert_installed!(
        bank,
        identities,
        NotifyDeathEstateCapability,
        NotifyDeathEstateOperation
    );
    assert_installed!(
        bank,
        identities,
        RetransmitDeathNoticeEstateCapability,
        RetransmitDeathNoticeEstateOperation
    );
    assert_installed!(
        bank,
        identities,
        FreezeEstateAccountCapability,
        FreezeEstateAccountOperation
    );
    assert_installed!(
        bank,
        identities,
        OpenEstateCaseCapability,
        OpenEstateCaseOperation
    );
    assert_installed!(
        bank,
        identities,
        RecognizeEstateExecutorCapability,
        RecognizeEstateExecutorOperation
    );
    assert_installed!(
        bank,
        identities,
        DelegateEstateCapability,
        DelegateEstateCapabilityOperation
    );
    assert_installed!(
        bank,
        identities,
        RevokeEstateCapability,
        RevokeEstateCapabilityOperation
    );
    assert_installed!(
        bank,
        identities,
        RequestEstateEmergencyAccessCapability,
        RequestEstateEmergencyAccessOperation
    );
    assert_installed!(
        bank,
        identities,
        ApproveEstateEmergencyAccessCapability,
        ApproveEstateEmergencyAccessOperation
    );
    assert_installed!(
        bank,
        identities,
        RevokeEstateEmergencyAccessCapability,
        RevokeEstateEmergencyAccessOperation
    );
    assert_installed!(
        bank,
        identities,
        CompleteEstateMandatoryReviewCapability,
        CompleteEstateMandatoryReviewOperation
    );
    assert_installed!(
        bank,
        identities,
        ReleaseEstateCapability,
        ReleaseEstateOperation
    );
    assert_installed!(
        bank,
        identities,
        DisburseEstateCapability,
        DisburseEstateOperation
    );
    assert_installed!(
        bank,
        identities,
        ViewEstateAdministrationCapability,
        ViewRestrictedEstateOperation
    );
    assert_installed!(
        bank,
        identities,
        ViewEstateIdentityVerificationCapability,
        ViewRestrictedEstateOperation
    );
    assert_installed!(
        bank,
        identities,
        ViewEstateLegalComplianceCapability,
        ViewRestrictedEstateOperation
    );
    assert_installed!(
        bank,
        identities,
        ViewEstateEmergencyProtectionCapability,
        ViewRestrictedEstateOperation
    );
    assert_installed!(
        bank,
        identities,
        ViewEstateMandatoryReviewCapability,
        ViewRestrictedEstateOperation
    );

    assert_eq!(identities.len(), 18);

    let restricted_view_capability = bank
        .capability(
            ViewEstateAdministrationCapability::reference(),
            ViewRestrictedEstateOperation::reference(),
        )
        .unwrap();
    let restricted_view = bank
        .installed_operation_for_capability(&restricted_view_capability)
        .unwrap();
    let graph_obligations = restricted_view.graph_obligations();
    let adoption = worth_query_host::facade::inspect_installed_graph_obligations(
        "bank-domain",
        graph_obligations,
    )
    .expect("Bank must inspect the installed obligation set without rebuilding authority");
    assert_eq!(adoption.consumer_name(), "bank-domain");
    assert_eq!(adoption.subject_name(), "ViewRestrictedEstateOperation");
    assert_eq!(adoption.rows().len(), graph_obligations.rows().len());
    assert_eq!(
        adoption.installed_set_identity(),
        graph_obligations.identity().bytes()
    );
    let authorization = graph_obligations
        .rows()
        .iter()
        .find_map(|obligation| obligation.authorization_requirement())
        .expect("the installed operation must carry its capability authorization obligation");
    let worth_query_host::facade::domain::WorthQueryInstalledGraphAuthorizationRequirement::Capabilities(
        requirements,
    ) = authorization
    else {
        panic!("the restricted view must remain capability-authorized")
    };
    assert_eq!(requirements.len(), 1);
    assert_eq!(
        requirements[0].identity(),
        restricted_view_capability.identity()
    );
}

#[test]
fn descriptive_binding_cannot_cross_an_independent_cryptographic_root() {
    let runtime = WorthQueryInstallationRuntimeIdentity::fresh();
    let retained = runtime.retain_for_execution_installation();
    let (_first_index, first) = installed_bank(runtime);
    let (_second_index, second) = installed_bank(retained);
    let capability = first
        .capability(
            ViewEstateLegalComplianceCapability::reference(),
            ViewRestrictedEstateOperation::reference(),
        )
        .unwrap();

    let denial = second
        .validate_installed_capability(&capability)
        .unwrap_err();
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationCapabilityInstallationDenialKind::AuthorityMismatch
    );
}

#[test]
fn wrong_operation_and_stale_or_foreign_worlds_fail_with_exact_denials() {
    let runtime = WorthQueryInstallationRuntimeIdentity::fresh();
    let retained_for_generation = runtime.retain_for_execution_installation();
    let (_initial_index, initial) =
        installed_bank_at(runtime, WorthQueryInstallationGeneration::initial());
    let wrong_operation = match initial.capability(
        ViewEstateLegalComplianceCapability::reference(),
        DisburseEstateOperation::reference(),
    ) {
        Ok(_) => panic!("a capability must remain bound to its exact operation"),
        Err(denial) => denial,
    };
    assert_eq!(
        wrong_operation.kind(),
        WorthQueryApplicationCapabilityInstallationDenialKind::CapabilityMeaningChanged
    );

    let capability = initial
        .capability(
            ViewEstateLegalComplianceCapability::reference(),
            ViewRestrictedEstateOperation::reference(),
        )
        .unwrap();
    let (_successor_index, successor) = installed_bank_at(
        retained_for_generation,
        WorthQueryInstallationGeneration::initial().successor(),
    );
    assert_eq!(
        successor
            .validate_installed_capability(&capability)
            .unwrap_err()
            .kind(),
        WorthQueryApplicationCapabilityInstallationDenialKind::StaleGeneration
    );

    let (_foreign_index, foreign) = installed_bank(WorthQueryInstallationRuntimeIdentity::fresh());
    assert_eq!(
        foreign
            .validate_installed_capability(&capability)
            .unwrap_err()
            .kind(),
        WorthQueryApplicationCapabilityInstallationDenialKind::ForeignRuntime
    );
}

#[test]
fn repeated_typed_lookup_reuses_the_installed_basis_and_digest() {
    let (_index, bank) = installed_bank(WorthQueryInstallationRuntimeIdentity::fresh());
    let first = bank
        .capability(
            ViewEstateLegalComplianceCapability::reference(),
            ViewRestrictedEstateOperation::reference(),
        )
        .unwrap();
    let retained_identity = first.identity().clone();
    let retained_basis = first.canonical_basis() as *const _;
    for _ in 0..4_096 {
        let lookup = bank
            .capability(
                ViewEstateLegalComplianceCapability::reference(),
                ViewRestrictedEstateOperation::reference(),
            )
            .unwrap();
        assert_eq!(lookup.identity(), &retained_identity);
        assert_eq!(lookup.canonical_basis() as *const _, retained_basis);
        let evidence = lookup.lookup_evidence();
        assert_eq!(evidence.registry_probes(), 1);
        assert_eq!(evidence.basis_preparations(), 0);
        assert_eq!(evidence.digest_derivations(), 0);
        assert_eq!(evidence.digest_text_materializations(), 0);
    }
}
