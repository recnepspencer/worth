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
            ApplicationCapabilityFieldBinding::from_reference(EmergencyAccessIdentityField::at(
                "EmergencyAccessIdentityField",
            ))
            .unwrap(),
            ApplicationCapabilityFieldBinding::from_reference(EmergencyAccessReasonField::at(
                "EmergencyAccessReasonField",
            ))
            .unwrap(),
            ApplicationCapabilityFieldBinding::from_reference(EmergencyAccessStatusField::at(
                "EmergencyAccessStatusField",
            ))
            .unwrap(),
            ApplicationCapabilityFieldBinding::from_reference(EmergencyAccessClosedAtField::at(
                "EmergencyAccessClosedAtField",
            ))
            .unwrap(),
            ApplicationCapabilityElevationStates::new(
                elevation_status(EmergencyAccessStatus::Requested),
                elevation_status(EmergencyAccessStatus::Approved),
                elevation_status(EmergencyAccessStatus::Expired),
                elevation_status(EmergencyAccessStatus::Revoked),
            ),
            ApplicationCapabilityValidityDefinition::new(
                ApplicationCapabilityValidityTimeline::UnixEpochSeconds,
                ApplicationCapabilityFieldBinding::from_reference(
                    EmergencyAccessIssuedAtField::at("EmergencyAccessIssuedAtField"),
                )
                .unwrap(),
                ApplicationCapabilityFieldBinding::from_reference(
                    EmergencyAccessExpiresAtField::at("EmergencyAccessExpiresAtField"),
                )
                .unwrap(),
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
                    MandatoryReviewIdentityField::at("MandatoryReviewIdentityField"),
                )
                .unwrap(),
                ApplicationCapabilityValueBinding::new(
                    MandatoryReviewKindField::at("MandatoryReviewKindField"),
                    crate::schema::encoded_bank_value(MandatoryReviewKind::EmergencyAccess),
                )
                .unwrap(),
                ApplicationCapabilityRelationBinding::from_reference(ReviewEstate::reference()),
                ApplicationCapabilityRelationBinding::from_reference(ReviewPrincipal::reference()),
                ApplicationCapabilityFieldBinding::from_reference(MandatoryReviewStatusField::at(
                    "MandatoryReviewStatusField",
                ))
                .unwrap(),
                ApplicationCapabilityFieldBinding::from_reference(
                    MandatoryReviewReviewedAtField::at("MandatoryReviewReviewedAtField"),
                )
                .unwrap(),
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
        EmergencyAccessStatusField::at("EmergencyAccessStatusField"),
        crate::schema::encoded_bank_value(status),
    )
    .unwrap()
}

fn review_status(status: MandatoryReviewStatus) -> ApplicationCapabilityValueBinding {
    ApplicationCapabilityValueBinding::new(
        MandatoryReviewStatusField::at("MandatoryReviewStatusField"),
        crate::schema::encoded_bank_value(status),
    )
    .unwrap()
}
