use std::collections::BTreeSet;

use bank_domain::schema::BankSchema;
use worth_query_decl::facade::application_schema::{
    ApplicationOperationDecisionReadTarget, ApplicationSchemaMember,
};

#[test]
fn required_estate_topology_and_policy_contributions_are_present() {
    let declaration = BankSchema::declaration().unwrap();
    let members = declaration.erased().members();
    assert_required_subset_present(
        members.iter().filter_map(entity_name).collect(),
        estate_entities(),
    );
    assert_required_subset_present(
        members.iter().filter_map(relation_name).collect(),
        estate_relations(),
    );
    assert_required_subset_present(
        members.iter().filter_map(policy_name).collect(),
        estate_policies(),
    );
    assert_eq!(
        members
            .iter()
            .filter_map(capability_context_name)
            .collect::<BTreeSet<_>>(),
        [
            "ApprovedBusinessPaymentControlContext",
            "EstateActionContext"
        ]
        .into_iter()
        .collect()
    );
    assert_eq!(
        members
            .iter()
            .filter_map(capability_context_slot_name)
            .collect::<BTreeSet<_>>(),
        [
            "EstateEmergencyAccessSlot",
            "EstateLegalAuthoritySlot",
            "EstateMandatoryReviewSlot",
        ]
        .into_iter()
        .collect()
    );
    assert_eq!(
        members
            .iter()
            .filter_map(capability_provenance_name)
            .collect::<BTreeSet<_>>(),
        [
            "ApprovedBusinessPaymentGrantProvenance",
            "EstateGrantChainProvenance",
        ]
        .into_iter()
        .collect()
    );
}

#[test]
fn estate_capabilities_and_installed_phase_seven_programs_are_declared() {
    let declaration = BankSchema::declaration().unwrap();
    let members = declaration.erased().members();
    assert_required_subset_present(
        members.iter().filter_map(operation_name).collect(),
        estate_operations(),
    );
    assert_eq!(
        members
            .iter()
            .filter_map(capability_name)
            .collect::<BTreeSet<_>>(),
        estate_capabilities()
    );
    let executable_operations = members
        .iter()
        .filter_map(operation_program_name)
        .collect::<BTreeSet<_>>();
    assert_eq!(
        executable_operations
            .intersection(&estate_operations())
            .copied()
            .collect::<BTreeSet<_>>(),
        [
            "ApproveEstateEmergencyAccessOperation",
            "CompleteEstateMandatoryReviewOperation",
            "DisburseEstateOperation",
            "FreezeEstateAccountOperation",
            "NotifyDeathEstateOperation",
            "OpenEstateCaseOperation",
            "RecognizeEstateExecutorOperation",
            "ReleaseEstateOperation",
            "RequestEstateEmergencyAccessOperation",
            "RetransmitDeathNoticeEstateOperation",
            "RevokeEstateEmergencyAccessOperation",
        ]
        .into_iter()
        .collect(),
        "Phase 7.7 installs the exact emergency lifecycle and currently completed ordinary estate mutation programs"
    );

    assert_estate_sources_have_no_local_authority_lane();
}

#[test]
fn delegation_activation_keeps_bank_reads_without_an_application_effect_program() {
    let declaration = BankSchema::declaration().unwrap();
    let members = declaration.erased().members();
    assert!(members.iter().all(|member| !matches!(
        member,
        ApplicationSchemaMember::OperationProgram { operation, .. }
            if operation == "DelegateEstateCapabilityOperation"
    )));

    let reads = members
        .iter()
        .filter_map(delegation_activation_read)
        .collect::<BTreeSet<_>>();
    assert_eq!(
        reads,
        [
            ("field", "AccountIdentity"),
            ("field", "BranchIdentityField"),
            ("field", "InstitutionIdentityField"),
            ("relation", "BranchInstitution"),
            ("relation", "EstateAccount"),
            ("relation", "EstateBranch"),
        ]
        .into_iter()
        .collect()
    );
}

fn delegation_activation_read(member: &ApplicationSchemaMember) -> Option<(&'static str, &str)> {
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

fn assert_estate_sources_have_no_local_authority_lane() {
    let sources = [
        include_str!("../src/estate/mod.rs"),
        include_str!("../src/estate/identities.rs"),
        include_str!("../src/estate/capability.rs"),
        include_str!("../src/estate/disclosure.rs"),
        include_str!("../src/estate/emergency_access.rs"),
        include_str!("../src/estate/operations.rs"),
        include_str!("../src/estate/aftermath.rs"),
        include_str!("../src/estate/world.rs"),
        include_str!("../src/schema/estate/mod.rs"),
        include_str!("../src/schema/estate/authority_relation_installation.rs"),
        include_str!("../src/schema/estate/capability_installation.rs"),
        include_str!("../src/schema/estate/capability_contracts.rs"),
        include_str!("../src/schema/estate/capability_contract_installation.rs"),
        include_str!("../src/schema/estate/entities.rs"),
        include_str!("../src/schema/estate/estate_relation_installation.rs"),
        include_str!("../src/schema/estate/fields.rs"),
        include_str!("../src/schema/estate/member_installation.rs"),
        include_str!("../src/schema/estate/policies.rs"),
        include_str!("../src/schema/estate/policy_installation.rs"),
        include_str!("../src/schema/estate/relations.rs"),
        include_str!("../src/schema/estate/values.rs"),
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

fn assert_required_subset_present(actual: BTreeSet<&str>, expected: BTreeSet<&str>) {
    assert_eq!(
        actual
            .intersection(&expected)
            .copied()
            .collect::<BTreeSet<_>>(),
        expected
    );
}

fn entity_name(member: &ApplicationSchemaMember) -> Option<&str> {
    match member {
        ApplicationSchemaMember::Entity { entity } => Some(entity),
        _ => None,
    }
}

fn relation_name(member: &ApplicationSchemaMember) -> Option<&str> {
    match member {
        ApplicationSchemaMember::Relation { relation, .. } => Some(relation),
        _ => None,
    }
}

fn policy_name(member: &ApplicationSchemaMember) -> Option<&str> {
    match member {
        ApplicationSchemaMember::Policy { policy } => Some(policy),
        _ => None,
    }
}

fn operation_name(member: &ApplicationSchemaMember) -> Option<&str> {
    match member {
        ApplicationSchemaMember::Operation { operation, .. } => Some(operation),
        _ => None,
    }
}

fn operation_program_name(member: &ApplicationSchemaMember) -> Option<&str> {
    match member {
        ApplicationSchemaMember::OperationProgram { operation, .. } => Some(operation),
        _ => None,
    }
}

fn capability_name(member: &ApplicationSchemaMember) -> Option<&str> {
    match member {
        ApplicationSchemaMember::ApplicationCapability { contract } => Some(contract.name()),
        _ => None,
    }
}

fn estate_entities() -> BTreeSet<&'static str> {
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

fn estate_relations() -> BTreeSet<&'static str> {
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

fn estate_policies() -> BTreeSet<&'static str> {
    ["EstateCapabilityScopePolicy"].into_iter().collect()
}

fn capability_context_name(member: &ApplicationSchemaMember) -> Option<&str> {
    match member {
        ApplicationSchemaMember::ApplicationCapabilityContext { context, .. } => Some(context),
        _ => None,
    }
}

fn capability_context_slot_name(member: &ApplicationSchemaMember) -> Option<&str> {
    match member {
        ApplicationSchemaMember::ApplicationCapabilityContextEntitySlot { slot, .. } => Some(slot),
        _ => None,
    }
}

fn capability_provenance_name(member: &ApplicationSchemaMember) -> Option<&str> {
    match member {
        ApplicationSchemaMember::ApplicationCapabilityProvenance { provenance, .. } => {
            Some(provenance)
        }
        _ => None,
    }
}

fn estate_operations() -> BTreeSet<&'static str> {
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
        "RetransmitDeathNoticeEstateOperation",
        "RevokeEstateCapabilityOperation",
        "RevokeEstateEmergencyAccessOperation",
        "ViewRestrictedEstateOperation",
    ]
    .into_iter()
    .collect()
}

fn estate_capabilities() -> BTreeSet<&'static str> {
    [
        "ApproveEstateEmergencyAccessCapability",
        "ApprovedBusinessPaymentAdvance",
        "ApprovedBusinessPaymentApproval",
        "ApprovedBusinessPaymentAuthoring",
        "ApprovedBusinessPaymentInstanceStart",
        "CompleteEstateMandatoryReviewCapability",
        "DelegateEstateCapability",
        "DisburseEstateCapability",
        "FreezeEstateAccountCapability",
        "NotifyDeathEstateCapability",
        "OpenEstateCaseCapability",
        "RecognizeEstateExecutorCapability",
        "ReleaseEstateCapability",
        "RequestEstateEmergencyAccessCapability",
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
