pub(crate) use worth_query_bank_testkit_domain::facade::domain_contracts::estate_capability_installation::transition_dimensions::*;

#[test]
fn lifecycle_commands_install_honest_command_dimensions() {
    let (_index, bank) = installed_bank(WorthQueryInstallationRuntimeIdentity::fresh());

    assert_transition_target!(
        bank,
        RequestEstateEmergencyAccessCapability,
        RequestEstateEmergencyAccessOperation,
        EstateCapabilityOperation::RequestEmergencyAccess,
        EstateCapabilityPurpose::EmergencyProtection
    );
    assert_transition_target!(
        bank,
        ApproveEstateEmergencyAccessCapability,
        ApproveEstateEmergencyAccessOperation,
        EstateCapabilityOperation::ApproveEmergencyAccess,
        EstateCapabilityPurpose::EmergencyProtection
    );
    assert_transition_target!(
        bank,
        RevokeEstateEmergencyAccessCapability,
        RevokeEstateEmergencyAccessOperation,
        EstateCapabilityOperation::RevokeEmergencyAccess,
        EstateCapabilityPurpose::EmergencyProtection
    );
    assert_transition_target!(
        bank,
        CompleteEstateMandatoryReviewCapability,
        CompleteEstateMandatoryReviewOperation,
        EstateCapabilityOperation::CompleteMandatoryReview,
        EstateCapabilityPurpose::MandatoryReview
    );
}

#[test]
fn revoke_command_projects_the_exact_emergency_access_subject() {
    let access = EmergencyAccessId::new(301).unwrap();
    let action = EstateAction::RevokeEmergencyAccess {
        estate: EstateCaseId::new(201).unwrap(),
        access,
    };
    let projection = <EstateAction as ApplicationCapabilityRequest<
        BankSchema,
        RevokeEstateEmergencyAccessCapability,
    >>::capability_request(&action)
    .unwrap();

    let [subject] = projection.context_value().entities() else {
        panic!("revoke must select exactly one lifecycle subject");
    };
    assert_eq!(subject.slot().slot(), "EstateEmergencyAccessSlot");
    assert_eq!(subject.selector().entity(), "EmergencyAccess");
    assert_eq!(
        subject.selector().value(),
        &EmergencyAccessIdBinding::encode(&access).unwrap()
    );
}
