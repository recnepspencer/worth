use worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder;

use super::{
    CapabilityAmountCeilingField, CapabilityDelegationLimitField, CapabilityDisclosureField,
    CapabilityGrant, CapabilityGrantIdentityField, CapabilityGrantStatusField,
    CapabilityOperationField, CapabilityPurposeField, CapabilityValidFromField,
    CapabilityValidThroughField, CapabilityWorkflowStageField, EmergencyAccess,
    EmergencyAccessClosedAtField, EmergencyAccessExpiresAtField, EmergencyAccessIdentityField,
    EmergencyAccessIssuedAtField, EmergencyAccessReasonField, EmergencyAccessStatusField,
    MandatoryReview, MandatoryReviewIdentityField, MandatoryReviewKindField,
    MandatoryReviewReviewedAtField, MandatoryReviewStatusField,
};
use crate::schema::BankSchema;

pub(super) fn install(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
    install_elevation_fields(install_grant_fields(schema))
}

fn install_grant_fields(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
    schema
        .field(
            CapabilityGrant::reference(),
            CapabilityGrantIdentityField::reference(),
        )
        .field(
            CapabilityGrant::reference(),
            CapabilityOperationField::reference(),
        )
        .field(
            CapabilityGrant::reference(),
            CapabilityPurposeField::reference(),
        )
        .field(
            CapabilityGrant::reference(),
            CapabilityDisclosureField::reference(),
        )
        .field(
            CapabilityGrant::reference(),
            CapabilityAmountCeilingField::reference(),
        )
        .field(
            CapabilityGrant::reference(),
            CapabilityValidFromField::reference(),
        )
        .field(
            CapabilityGrant::reference(),
            CapabilityValidThroughField::reference(),
        )
        .field(
            CapabilityGrant::reference(),
            CapabilityDelegationLimitField::reference(),
        )
        .field(
            CapabilityGrant::reference(),
            CapabilityWorkflowStageField::reference(),
        )
        .field(
            CapabilityGrant::reference(),
            CapabilityGrantStatusField::reference(),
        )
}

fn install_elevation_fields(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
    schema
        .field(
            EmergencyAccess::reference(),
            EmergencyAccessIdentityField::reference(),
        )
        .field(
            EmergencyAccess::reference(),
            EmergencyAccessReasonField::reference(),
        )
        .field(
            EmergencyAccess::reference(),
            EmergencyAccessStatusField::reference(),
        )
        .field(
            EmergencyAccess::reference(),
            EmergencyAccessIssuedAtField::reference(),
        )
        .field(
            EmergencyAccess::reference(),
            EmergencyAccessExpiresAtField::reference(),
        )
        .field(
            EmergencyAccess::reference(),
            EmergencyAccessClosedAtField::reference(),
        )
        .field(
            MandatoryReview::reference(),
            MandatoryReviewIdentityField::reference(),
        )
        .field(
            MandatoryReview::reference(),
            MandatoryReviewStatusField::reference(),
        )
        .field(
            MandatoryReview::reference(),
            MandatoryReviewReviewedAtField::reference(),
        )
        .field(
            MandatoryReview::reference(),
            MandatoryReviewKindField::reference(),
        )
}
