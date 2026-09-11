use worth_query_decl::facade::application_capability::{
    ApplicationCapabilityContextEntitySlotBinding, ApplicationCapabilityElevationDefinition,
    ApplicationCapabilityElevationLifecycleDefinition, ApplicationCapabilityElevationRule,
    ApplicationCapabilityElevationStates, ApplicationCapabilityFieldBinding,
    ApplicationCapabilityMandatoryReviewDefinition, ApplicationCapabilityRelationBinding,
    ApplicationCapabilityTransitionBinding, ApplicationCapabilityValidityDefinition,
    ApplicationCapabilityValidityTimeline, ApplicationCapabilityValueBinding,
};

use super::*;
use crate::estate::{
    EmergencyAccessStatus, EstateCapabilityOperation, EstateCapabilityPurpose, MandatoryReviewKind,
    MandatoryReviewStatus,
};

pub(super) fn rule(
    action: EstateCapabilityOperation,
    purpose: EstateCapabilityPurpose,
) -> ApplicationCapabilityElevationRule {
    if action != EstateCapabilityOperation::ViewRestrictedEstate
        || purpose != EstateCapabilityPurpose::EmergencyProtection
    {
        return ApplicationCapabilityElevationRule::not_applicable();
    }
    ApplicationCapabilityElevationRule::governed(
        ApplicationCapabilityElevationDefinition::new(
            ApplicationCapabilityFieldBinding::from_reference(
                EmergencyAccessIdentityField::reference(),
            ),
            ApplicationCapabilityFieldBinding::from_reference(
                EmergencyAccessReasonField::reference(),
            ),
            ApplicationCapabilityFieldBinding::from_reference(
                EmergencyAccessStatusField::reference(),
            ),
            ApplicationCapabilityElevationStates::new(
                elevation_status(EmergencyAccessStatus::Requested),
                elevation_status(EmergencyAccessStatus::Approved),
                elevation_status(EmergencyAccessStatus::Expired),
                elevation_status(EmergencyAccessStatus::Revoked),
            ),
            ApplicationCapabilityValidityDefinition::new(
                ApplicationCapabilityValidityTimeline::UnixEpochSeconds,
                ApplicationCapabilityFieldBinding::from_reference(
                    EmergencyAccessIssuedAtField::reference(),
                ),
                ApplicationCapabilityFieldBinding::from_reference(
                    EmergencyAccessExpiresAtField::reference(),
                ),
            ),
            std::time::Duration::from_secs(20 * 60),
            ApplicationCapabilityRelationBinding::from_reference(EmergencyRequester::reference()),
            ApplicationCapabilityRelationBinding::from_reference(EmergencyApprover::reference()),
            ApplicationCapabilityRelationBinding::from_reference(EmergencyGrant::reference()),
            ApplicationCapabilityElevationLifecycleDefinition::new(
                ApplicationCapabilityContextEntitySlotBinding::from_reference(
                    EstateEmergencyAccessSlot::reference(),
                ),
                ApplicationCapabilityContextEntitySlotBinding::from_reference(
                    EstateMandatoryReviewSlot::reference(),
                ),
                ApplicationCapabilityTransitionBinding::from_references_with_lifecycle_effect(
                    RequestEstateEmergencyAccessCapability::reference(),
                    RequestEstateEmergencyAccessOperation::reference(),
                ),
                ApplicationCapabilityTransitionBinding::from_references_with_lifecycle_effect(
                    ApproveEstateEmergencyAccessCapability::reference(),
                    ApproveEstateEmergencyAccessOperation::reference(),
                ),
                ApplicationCapabilityTransitionBinding::from_references_with_lifecycle_effect(
                    RevokeEstateEmergencyAccessCapability::reference(),
                    RevokeEstateEmergencyAccessOperation::reference(),
                ),
                ApplicationCapabilityTransitionBinding::from_references_with_lifecycle_effect(
                    CompleteEstateMandatoryReviewCapability::reference(),
                    CompleteEstateMandatoryReviewOperation::reference(),
                ),
            ),
            ApplicationCapabilityMandatoryReviewDefinition::new(
                ApplicationCapabilityRelationBinding::from_reference(EmergencyReview::reference()),
                ApplicationCapabilityFieldBinding::from_reference(
                    MandatoryReviewIdentityField::reference(),
                ),
                ApplicationCapabilityValueBinding::new(
                    MandatoryReviewKindField::reference(),
                    crate::schema::encoded_bank_value(MandatoryReviewKind::EmergencyAccess),
                ),
                ApplicationCapabilityRelationBinding::from_reference(ReviewEstate::reference()),
                ApplicationCapabilityRelationBinding::from_reference(ReviewPrincipal::reference()),
                ApplicationCapabilityFieldBinding::from_reference(
                    MandatoryReviewStatusField::reference(),
                ),
                review_status(MandatoryReviewStatus::Required),
                review_status(MandatoryReviewStatus::Completed),
            ),
        )
        .with_resource_relation(ApplicationCapabilityRelationBinding::from_reference(
            EmergencyEstate::reference(),
        )),
    )
}

fn elevation_status(status: EmergencyAccessStatus) -> ApplicationCapabilityValueBinding {
    ApplicationCapabilityValueBinding::new(
        EmergencyAccessStatusField::reference(),
        crate::schema::encoded_bank_value(status),
    )
}

fn review_status(status: MandatoryReviewStatus) -> ApplicationCapabilityValueBinding {
    ApplicationCapabilityValueBinding::new(
        MandatoryReviewStatusField::reference(),
        crate::schema::encoded_bank_value(status),
    )
}
