use worth_query_decl::facade::application_schema::ApplicationSchemaMember;

use super::support::{
    aspect_name, effect_name, expected, field_name, names, policy_name, unit_name,
};

pub(super) fn assert_field_and_governance_inventory(members: &[ApplicationSchemaMember]) {
    assert_aspect_inventory(members);
    assert_field_inventory(members);
    assert_governance_inventory(members);
}

fn assert_aspect_inventory(members: &[ApplicationSchemaMember]) {
    assert_eq!(
        names(members, aspect_name),
        expected(&[
            "AccountProfile",
            "AccountState",
            "AuthorizationIdentity",
            "AuthorizationScope",
            "BranchIdentity",
            "BusinessIdentity",
            "CapabilityGrantRecord",
            "DeathNoticeRecord",
            "EmployeeScope",
            "EmergencyAccessRecord",
            "EstateCaseRecord",
            "ExternalPrincipalIdentity",
            "Identity",
            "InstitutionIdentity",
            "JournalIdentity",
            "JournalState",
            "LegalAuthorityRecord",
            "MandatoryReviewRecord",
            "PaymentIdentity",
            "PaymentState",
            "PaymentValue",
            "PostingIdentity",
            "PostingValue",
            "PrincipalIdentity",
        ])
    );
}

fn assert_field_inventory(members: &[ApplicationSchemaMember]) {
    assert_eq!(
        names(members, field_name),
        expected(&[
            "AccountAuthorizationIdentity",
            "AccountDisplayName",
            "AccountIdentity",
            "AccountingRevision",
            "AssignmentRole",
            "AuthorizationRole",
            "BranchIdentityField",
            "BusinessIdentityField",
            "CapabilityAmountCeilingField",
            "CapabilityDelegationLimitField",
            "CapabilityDisclosureField",
            "CapabilityGrantIdentityField",
            "CapabilityGrantStatusField",
            "CapabilityOperationField",
            "CapabilityPurposeField",
            "CapabilityValidFromField",
            "CapabilityValidThroughField",
            "CapabilityWorkflowStageField",
            "DeathNoticeIdentityField",
            "DeathNoticeStatusField",
            "EmergencyAccessClosedAtField",
            "EmergencyAccessExpiresAtField",
            "EmergencyAccessIdentityField",
            "EmergencyAccessIssuedAtField",
            "EmergencyAccessReasonField",
            "EmergencyAccessStatusField",
            "EmployeeAssignmentIdentityField",
            "EstateCaseIdentityField",
            "EstateCaseStatusField",
            "EstateWorkflowStageField",
            "ExternalIdentityKey",
            "ExternalMappingStatus",
            "InstitutionIdentityField",
            "JournalIdentityField",
            "JournalPurpose",
            "Kind",
            "LegalAuthorityIdentityField",
            "LegalAuthorityKindField",
            "LegalAuthorityRecognizedField",
            "MandatoryReviewIdentityField",
            "MandatoryReviewKindField",
            "MandatoryReviewReviewedAtField",
            "MandatoryReviewStatusField",
            "PaymentAmount",
            "PaymentIdentityField",
            "PaymentStatusField",
            "PostingAccountSequence",
            "PostingAmount",
            "PostingIdentityField",
            "PrincipalIdentityField",
            "Purpose",
            "Status",
        ])
    );
}

fn assert_governance_inventory(members: &[ApplicationSchemaMember]) {
    assert_eq!(
        names(members, policy_name),
        expected(&[
            "AccountMutationScopePolicy",
            "AccountVisibilityPolicy",
            "DistinctApproverPolicy",
            "EmployeeScopePolicy",
            "EstateCapabilityScopePolicy",
        ])
    );
    assert_eq!(names(members, unit_name), expected(&["UsdCurrency"]));
    assert_eq!(
        names(members, effect_name),
        expected(&[
            "AccountActivityEffect",
            "EstateDeathNotificationEffect",
            "EstateEmergencyAccessActivityEffect",
        ])
    );
}
