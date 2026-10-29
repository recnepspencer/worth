pub(crate) use worth_query_bank_worlds::facade::domain::domain_contracts::estate_aftermath_contract::*;

#[test]
fn reconcilable_estate_operations_publish_their_installed_posture() {
    let bank = installed_bank();
    assert_installed_posture(
        &bank,
        NotifyDeathEstateOperation::reference(),
        Some(PublishedAftermathPosture::Reconcilable),
        "NotifyDeath",
    );
    assert_installed_posture(
        &bank,
        RetransmitDeathNoticeEstateOperation::reference(),
        Some(PublishedAftermathPosture::Reconcilable),
        "RetransmitDeathNotice",
    );
}

#[test]
fn reversible_estate_operations_publish_their_installed_posture() {
    let bank = installed_bank();
    assert_installed_posture(
        &bank,
        FreezeEstateAccountOperation::reference(),
        Some(PublishedAftermathPosture::Reversible),
        "FreezeAccount",
    );
    assert_installed_posture(
        &bank,
        RevokeEstateCapabilityOperation::reference(),
        Some(PublishedAftermathPosture::Reversible),
        "RevokeCapability",
    );
    assert_installed_posture(
        &bank,
        ApproveEstateEmergencyAccessOperation::reference(),
        Some(PublishedAftermathPosture::Reversible),
        "ApproveEmergencyAccess",
    );
}

#[test]
fn irreversible_estate_operations_publish_their_installed_posture() {
    let bank = installed_bank();
    assert_installed_posture(
        &bank,
        OpenEstateCaseOperation::reference(),
        Some(PublishedAftermathPosture::Irreversible),
        "OpenEstateCase",
    );
    assert_installed_posture(
        &bank,
        RecognizeEstateExecutorOperation::reference(),
        Some(PublishedAftermathPosture::Irreversible),
        "RecognizeExecutor",
    );
    assert_installed_posture(
        &bank,
        DelegateEstateCapabilityOperation::reference(),
        Some(PublishedAftermathPosture::Irreversible),
        "DelegateCapability",
    );
    assert_installed_posture(
        &bank,
        RequestEstateEmergencyAccessOperation::reference(),
        Some(PublishedAftermathPosture::Irreversible),
        "RequestEmergencyAccess",
    );
    assert_installed_posture(
        &bank,
        RevokeEstateEmergencyAccessOperation::reference(),
        Some(PublishedAftermathPosture::Irreversible),
        "RevokeEmergencyAccess",
    );
    assert_installed_posture(
        &bank,
        CompleteEstateMandatoryReviewOperation::reference(),
        Some(PublishedAftermathPosture::Irreversible),
        "CompleteMandatoryReview",
    );
    assert_installed_posture(
        &bank,
        ReleaseEstateOperation::reference(),
        Some(PublishedAftermathPosture::Irreversible),
        "ReleaseEstate",
    );
}

#[test]
fn compensatable_estate_operation_publishes_its_installed_posture() {
    let bank = installed_bank();
    assert_installed_posture(
        &bank,
        DisburseEstateOperation::reference(),
        Some(PublishedAftermathPosture::Compensatable),
        "DisburseEstate",
    );
}

#[test]
fn view_restricted_estate_is_capability_only_and_has_no_aftermath_contract() {
    let bank = installed_bank();
    let view_capability = bank
        .capability(
            ViewEstateAdministrationCapability::reference(),
            ViewRestrictedEstateOperation::reference(),
        )
        .expect("view capability installs");
    let view = bank
        .installed_operation_for_capability(&view_capability)
        .expect("ViewRestrictedEstate installs through capability");
    assert_eq!(view.graph_obligations().rows().len(), 1);
    assert!(!BankSchema::declaration()
        .unwrap()
        .erased()
        .members()
        .iter()
        .any(|member| matches!(
            member,
            ApplicationSchemaMember::OperationAftermath { operation, .. }
                if operation == "ViewRestrictedEstate"
        )));
}

#[test]
fn estate_aftermath_source_has_no_nomutation_or_declared_posture_names() {
    let source = include_str!(concat!(
        concat!(env!("CARGO_MANIFEST_DIR"), "/../bank-domain"),
        "/src/estate/aftermath.rs"
    ));
    for forbidden in [
        "NoMutation",
        "EstateAftermath::Reversible",
        "EstateAftermath::Compensatable",
        "EstateAftermath::Reconcilable",
        "EstateAftermath::Irreversible",
        "PublishedAftermathPosture",
    ] {
        assert!(
            !source.contains(forbidden),
            "estate aftermath must not retain predecessor vocabulary {forbidden}"
        );
    }
}
