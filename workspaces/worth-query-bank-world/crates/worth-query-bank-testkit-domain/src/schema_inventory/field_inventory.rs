pub use worth_query_decl::facade::application_schema::ApplicationSchemaMember;

pub use super::support::{
    aspect_name, effect_name, expected, field_name, names, policy_name, unit_name,
};

pub fn assert_field_and_governance_inventory(members: &[ApplicationSchemaMember]) {
    assert_aspect_inventory(members);
    assert_field_inventory(members);
    assert_governance_inventory(members);
}

pub fn assert_aspect_inventory(members: &[ApplicationSchemaMember]) {
    assert_eq!(
        names(members, aspect_name),
        std::collections::BTreeSet::from([
            bank_domain::schema::AccountProfile::reference().name(),
            bank_domain::schema::AccountState::reference().name(),
            bank_domain::schema::AuthorizationIdentity::reference().name(),
            bank_domain::schema::AuthorizationScope::reference().name(),
            bank_domain::schema::ApprovedBusinessPaymentGrantFacts::reference().name(),
            bank_domain::schema::BranchIdentity::reference().name(),
            bank_domain::schema::BusinessIdentity::reference().name(),
            bank_domain::schema::CapabilityGrantRecord::reference().name(),
            bank_domain::schema::DeathNoticeRecord::reference().name(),
            bank_domain::schema::EmployeeScope::reference().name(),
            bank_domain::schema::EmergencyAccessRecord::reference().name(),
            bank_domain::schema::EstateCaseRecord::reference().name(),
            bank_domain::schema::ExternalPrincipalIdentity::reference().name(),
            bank_domain::schema::Identity::reference().name(),
            bank_domain::schema::InstitutionIdentity::reference().name(),
            bank_domain::schema::JournalIdentity::reference().name(),
            bank_domain::schema::JournalState::reference().name(),
            bank_domain::schema::LegalAuthorityRecord::reference().name(),
            bank_domain::schema::MandatoryReviewRecord::reference().name(),
            bank_domain::schema::PaymentIdentity::reference().name(),
            bank_domain::schema::PaymentState::reference().name(),
            bank_domain::schema::PaymentValue::reference().name(),
            bank_domain::schema::PostingIdentity::reference().name(),
            bank_domain::schema::PostingValue::reference().name(),
            bank_domain::schema::PrincipalIdentity::reference().name(),
            bank_domain::schema::ProgramDeploymentGrantFacts::reference().name(),
            bank_domain::schema::ProgramDeploymentScopeFacts::reference().name(),
        ])
    );
}

pub fn assert_field_inventory(members: &[ApplicationSchemaMember]) {
    assert_eq!(
        names(members, field_name),
        expected(&[
            "AccountAuthorizationIdentity",
            "AccountDisplayName",
            "AccountIdentity",
            "AccountingRevision",
            "AssignmentRole",
            "AuthorizationRole",
            "ApprovedBusinessPaymentGrantAction",
            "ApprovedBusinessPaymentGrantDelegationLimit",
            "ApprovedBusinessPaymentGrantNotAfter",
            "ApprovedBusinessPaymentGrantNotBefore",
            "ApprovedBusinessPaymentGrantPurpose",
            "ApprovedBusinessPaymentGrantStatus",
            "ApprovedBusinessPaymentGrantWorkflow",
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
            "JournalInstitutionContext",
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
            "ProgramDeploymentGrantAction",
            "ProgramDeploymentGrantDepth",
            "ProgramDeploymentGrantIdentity",
            "ProgramDeploymentGrantNotAfter",
            "ProgramDeploymentGrantNotBefore",
            "ProgramDeploymentGrantPurpose",
            "ProgramDeploymentGrantSourceRevision",
            "ProgramDeploymentGrantStatus",
            "ProgramDeploymentSourceRevision",
            "Purpose",
            "Status",
        ])
    );
}

pub fn assert_governance_inventory(members: &[ApplicationSchemaMember]) {
    assert_eq!(
        names(members, policy_name),
        expected(&[
            "AccountMutationScopePolicy",
            "AccountVisibilityPolicy",
            "DistinctApproverPolicy",
            "EmployeeScopePolicy",
            "EstateCapabilityScopePolicy",
            "ProgramDeploymentGrantIssuerPolicy",
        ])
    );
    assert_eq!(names(members, unit_name), expected(&["UsdCurrency"]));
    assert_eq!(
        names(members, effect_name),
        expected(&[
            "AccountActivityEffect",
            "ApprovedPaymentSettlementEffect",
            "EstateDeathNotificationEffect",
            "EstateEmergencyAccessActivityEffect",
        ])
    );
}
