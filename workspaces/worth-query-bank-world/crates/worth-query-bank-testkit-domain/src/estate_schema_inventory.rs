pub use std::collections::BTreeSet;

pub use bank_domain::schema::BankSchema;
pub use worth_query_decl::facade::application_schema::{
    ApplicationOperationDecisionReadTarget, ApplicationSchemaMember,
};

pub fn delegation_activation_read(
    member: &ApplicationSchemaMember,
) -> Option<(&'static str, &str)> {
    let ApplicationSchemaMember::OperationDecisionRead { operation, target } = member else {
        return None;
    };
    if operation != "DelegateEstateCapabilityOperation" {
        return None;
    }
    Some(match target {
        ApplicationOperationDecisionReadTarget::Entity { entity } => ("entity", entity),
        ApplicationOperationDecisionReadTarget::Field { field, .. } => ("field", field),
        ApplicationOperationDecisionReadTarget::Relation { relation, .. } => ("relation", relation),
    })
}

pub fn assert_estate_sources_have_no_local_authority_lane() {
    let sources = [
        include_str!("../../bank-domain/src/estate/mod.rs"),
        include_str!("../../bank-domain/src/estate/identities.rs"),
        include_str!("../../bank-domain/src/estate/capability.rs"),
        include_str!("../../bank-domain/src/estate/disclosure.rs"),
        include_str!("../../bank-domain/src/estate/emergency_access.rs"),
        include_str!("../../bank-domain/src/estate/operations.rs"),
        include_str!("../../bank-domain/src/estate/aftermath.rs"),
        include_str!("../../bank-domain/src/estate/world.rs"),
        include_str!("../../bank-domain/src/schema/estate/mod.rs"),
        include_str!("../../bank-domain/src/schema/estate/authority_relation_installation.rs"),
        include_str!("../../bank-domain/src/schema/estate/capability_installation.rs"),
        include_str!("../../bank-domain/src/schema/estate/capability_contracts.rs"),
        include_str!("../../bank-domain/src/schema/estate/capability_contract_installation.rs"),
        include_str!("../../bank-domain/src/schema/estate/entities.rs"),
        include_str!("../../bank-domain/src/schema/estate/estate_relation_installation.rs"),
        include_str!("../../bank-domain/src/schema/estate/fields.rs"),
        include_str!("../../bank-domain/src/schema/estate/member_installation.rs"),
        include_str!("../../bank-domain/src/schema/estate/policies.rs"),
        include_str!("../../bank-domain/src/schema/estate/policy_installation.rs"),
        include_str!("../../bank-domain/src/schema/estate/relations.rs"),
        include_str!("../../bank-domain/src/schema/estate/values.rs"),
    ]
    .join("\n");
    for forbidden in [
        "superuser",
        "redact",
        "HashMap",
        "dyn Fn",
        "undo_stack",
        "route_predicate",
    ] {
        assert!(
            !sources.contains(forbidden),
            "estate world contains forbidden local authority lane: {forbidden}"
        );
    }
}

pub fn assert_required_subset_present(actual: BTreeSet<&str>, expected: BTreeSet<&str>) {
    assert_eq!(
        actual
            .intersection(&expected)
            .copied()
            .collect::<BTreeSet<_>>(),
        expected
    );
}

pub fn entity_name(member: &ApplicationSchemaMember) -> Option<&str> {
    match member {
        ApplicationSchemaMember::Entity { entity } => Some(entity),
        _ => None,
    }
}

pub fn relation_name(member: &ApplicationSchemaMember) -> Option<&str> {
    match member {
        ApplicationSchemaMember::Relation { relation, .. } => Some(relation),
        _ => None,
    }
}

pub fn policy_name(member: &ApplicationSchemaMember) -> Option<&str> {
    match member {
        ApplicationSchemaMember::Policy { policy } => Some(policy),
        _ => None,
    }
}

pub fn operation_name(member: &ApplicationSchemaMember) -> Option<&str> {
    match member {
        ApplicationSchemaMember::Operation { operation, .. } => Some(operation),
        _ => None,
    }
}

pub fn operation_program_name(member: &ApplicationSchemaMember) -> Option<&str> {
    match member {
        ApplicationSchemaMember::OperationProgram { operation, .. } => Some(operation),
        _ => None,
    }
}

pub fn capability_name(member: &ApplicationSchemaMember) -> Option<&str> {
    match member {
        ApplicationSchemaMember::ApplicationCapability { contract } => Some(contract.name()),
        _ => None,
    }
}

pub fn estate_entities() -> BTreeSet<&'static str> {
    [
        "Branch",
        "CapabilityGrant",
        "DeathNotice",
        "EmergencyAccess",
        "EstateCase",
        "LegalAuthority",
        "MandatoryReview",
    ]
    .into_iter()
    .collect()
}

pub fn estate_relations() -> BTreeSet<&'static str> {
    [
        "BranchInstitution",
        "CapabilityAccount",
        "CapabilityBranch",
        "CapabilityEstate",
        "CapabilityGrantee",
        "CapabilityGrantor",
        "CapabilityInstitution",
        "CapabilityParent",
        "DeathNoticeSubject",
        "EmergencyApprover",
        "EmergencyEstate",
        "EmergencyGrant",
        "EmergencyRequester",
        "EmergencyReview",
        "EstateAccount",
        "EstateAssignment",
        "EstateAuthorizedSigner",
        "EstateBeneficiary",
        "EstateBranch",
        "EstateDeathNotice",
        "EstateDeceased",
        "EstateExecutor",
        "EstateJointOwner",
        "LegalAuthorityEstate",
        "LegalAuthorityHolder",
        "ReviewEstate",
        "ReviewPrincipal",
    ]
    .into_iter()
    .collect()
}

pub fn estate_policies() -> BTreeSet<&'static str> {
    ["EstateCapabilityScopePolicy"].into_iter().collect()
}

pub fn capability_context_name(member: &ApplicationSchemaMember) -> Option<&str> {
    match member {
        ApplicationSchemaMember::ApplicationCapabilityContext { context, .. } => Some(context),
        _ => None,
    }
}

pub fn capability_context_slot_name(member: &ApplicationSchemaMember) -> Option<&str> {
    match member {
        ApplicationSchemaMember::ApplicationCapabilityContextEntitySlot { slot, .. } => Some(slot),
        _ => None,
    }
}

pub fn capability_provenance_name(member: &ApplicationSchemaMember) -> Option<&str> {
    match member {
        ApplicationSchemaMember::ApplicationCapabilityProvenance { provenance, .. } => {
            Some(provenance)
        }
        _ => None,
    }
}

pub fn estate_operations() -> BTreeSet<&'static str> {
    [
        "ApproveEstateEmergencyAccessOperation",
        "CompleteEstateMandatoryReviewOperation",
        "DelegateEstateCapabilityOperation",
        "DisburseEstateOperation",
        "FreezeEstateAccountOperation",
        "NotifyDeathEstateOperation",
        "OpenEstateCaseOperation",
        "RecognizeEstateExecutorOperation",
        "ReleaseEstateOperation",
        "RequestEstateEmergencyAccessOperation",
        "RestoreFrozenEstateAccountOperation",
        "RetransmitDeathNoticeEstateOperation",
        "RevokeEstateCapabilityOperation",
        "RevokeEstateEmergencyAccessOperation",
        "ViewRestrictedEstateOperation",
    ]
    .into_iter()
    .collect()
}

pub fn estate_capabilities() -> BTreeSet<&'static str> {
    [
        "ApproveEstateEmergencyAccessCapability",
        "ApprovedBusinessPaymentAdvance",
        "ApprovedBusinessPaymentApproval",
        "ApprovedBusinessPaymentAuthoring",
        "ApprovedBusinessPaymentInstanceStart",
        "CompleteEstateMandatoryReviewCapability",
        "DelegateEstateCapability",
        "DeployBankProgram",
        "DisburseEstateCapability",
        "FreezeEstateAccountCapability",
        "NotifyDeathEstateCapability",
        "OpenEstateCaseCapability",
        "RecognizeEstateExecutorCapability",
        "ReleaseEstateCapability",
        "RequestEstateEmergencyAccessCapability",
        "RestoreFrozenEstateAccountCapability",
        "RetransmitDeathNoticeEstateCapability",
        "RevokeEstateCapability",
        "RevokeEstateEmergencyAccessCapability",
        "ViewEstateAdministrationCapability",
        "ViewEstateEmergencyProtectionCapability",
        "ViewEstateIdentityVerificationCapability",
        "ViewEstateLegalComplianceCapability",
        "ViewEstateMandatoryReviewCapability",
    ]
    .into_iter()
    .collect()
}
