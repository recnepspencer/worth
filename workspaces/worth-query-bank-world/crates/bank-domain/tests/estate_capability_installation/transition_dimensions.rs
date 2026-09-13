use bank_domain::{estate::*, schema::*};
use worth_query_host::facade::{
    declaration::{
        application_capability::{
            ApplicationCapabilityFieldDimension, ApplicationCapabilityRelationDimension,
            ApplicationCapabilityRequest,
        },
        application_schema::ApplicationScalarValueBinding,
    },
    domain::WorthQueryInstallationRuntimeIdentity,
};

use super::installed_bank;

macro_rules! assert_transition_target {
    ($bank:expr, $capability:ident, $operation:ident, $action:expr, $purpose:expr) => {{
        let capability = $bank
            .capability($capability::reference(), $operation::reference())
            .unwrap();
        let contract = capability.contract();
        let target = contract.target();
        assert_eq!(
            target.action().value(),
            &EstateCapabilityOperationBinding::encode(&$action).unwrap()
        );
        assert_eq!(
            target.purpose().value(),
            &EstateCapabilityPurposeBinding::encode(&$purpose).unwrap()
        );
        assert!(matches!(
            target.relation(),
            ApplicationCapabilityRelationDimension::NotApplicable
        ));
        assert!(matches!(
            target.field(),
            ApplicationCapabilityFieldDimension::NotApplicable
        ));
        assert!(matches!(
            contract.constraints().magnitude(),
            ApplicationCapabilityFieldDimension::NotApplicable
        ));
    }};
}

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
