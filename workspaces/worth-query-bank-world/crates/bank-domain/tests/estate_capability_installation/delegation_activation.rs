use bank_domain::{
    estate::{
        BranchId, CapabilityGrantId, CapabilityValidity, DelegationLimit, EstateAction,
        EstateCapabilityDelegationRequest, EstateCapabilityOperation, EstateCapabilityPurpose,
        EstateCapabilityScope, EstateCaseId, EstateMoment, EstateWorkflowStage,
    },
    model::{BankPrincipalId, InstitutionId},
    schema::{
        BankSchema, BranchIdBinding, DelegateEstateCapability, DelegateEstateCapabilityOperation,
        InstitutionIdBinding,
    },
};
use worth_query_host::facade::declaration::{
    application_capability::ApplicationCapabilityDelegationRequest,
    application_schema::ApplicationScalarValueBinding,
};
use worth_query_host::facade::domain::WorthQueryInstallationRuntimeIdentity;

use super::{installed_bank, installed_program_targets, InstalledProgramTarget};

#[test]
fn delegation_activation_installs_exact_bank_context_relations() {
    let (_index, bank) = installed_bank(WorthQueryInstallationRuntimeIdentity::fresh());
    let capability = bank
        .capability(
            DelegateEstateCapability::reference(),
            DelegateEstateCapabilityOperation::reference(),
        )
        .unwrap();
    let activation = capability
        .contract()
        .delegation()
        .activation()
        .expect("estate delegation activation must be installed");

    let mut context_relations = activation
        .context_relations()
        .iter()
        .map(|relation| relation.relation())
        .collect::<Vec<_>>();
    context_relations.sort_unstable();
    assert_eq!(
        context_relations,
        vec!["CapabilityBranch", "CapabilityInstitution"]
    );
}

#[test]
fn delegation_activation_installs_the_complete_framework_owned_effect_program() {
    let (_index, bank) = installed_bank(WorthQueryInstallationRuntimeIdentity::fresh());
    let operation = bank
        .installed_operation(DelegateEstateCapabilityOperation::reference())
        .expect("delegation activation must compile as an executable specialized operation");
    let mut expected = [InstalledProgramTarget::Create("CapabilityGrant".to_owned())]
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>();
    expected.extend(
        [
            "CapabilityGrantIdentityField",
            "CapabilityOperationField",
            "CapabilityPurposeField",
            "CapabilityDisclosureField",
            "CapabilityAmountCeilingField",
            "CapabilityValidFromField",
            "CapabilityValidThroughField",
            "CapabilityDelegationLimitField",
            "CapabilityWorkflowStageField",
            "CapabilityGrantStatusField",
        ]
        .into_iter()
        .map(write),
    );
    expected.extend([
        link("CapabilityGrantee", "Principal", "CapabilityGrant"),
        link("CapabilityGrantor", "Principal", "CapabilityGrant"),
        link("CapabilityEstate", "CapabilityGrant", "EstateCase"),
        link("CapabilityAccount", "CapabilityGrant", "Account"),
        link("CapabilityInstitution", "CapabilityGrant", "Institution"),
        link("CapabilityBranch", "CapabilityGrant", "Branch"),
        link("CapabilityParent", "CapabilityGrant", "CapabilityGrant"),
    ]);
    assert_eq!(installed_program_targets(operation.contracts()), expected);
}

#[test]
fn delegation_request_projects_exact_typed_bank_context_selectors() {
    let institution = InstitutionId::new(11).unwrap();
    let branch = BranchId::new(12).unwrap();
    let estate = EstateCaseId::new(13).unwrap();
    let action = EstateAction::DelegateCapability {
        estate,
        parent: CapabilityGrantId::new(14).unwrap(),
        child: EstateCapabilityDelegationRequest {
            id: CapabilityGrantId::new(15).unwrap(),
            grantee: BankPrincipalId::new(16).unwrap(),
            scope: EstateCapabilityScope {
                account: None,
                estate,
                institution,
                branch,
                operation: EstateCapabilityOperation::NotifyDeath,
                purpose: EstateCapabilityPurpose::EstateAdministration,
                field: None,
                amount_ceiling: None,
                validity: CapabilityValidity::new(
                    EstateMoment::from_epoch_seconds(100),
                    EstateMoment::from_epoch_seconds(200),
                )
                .unwrap(),
                delegation: DelegationLimit::generations(1),
                workflow_stage: EstateWorkflowStage::Administration,
            },
        },
    };

    let projection = <EstateAction as ApplicationCapabilityDelegationRequest<
        BankSchema,
        DelegateEstateCapability,
    >>::delegation_request(&action)
    .unwrap();
    let context = projection.activation_context();
    assert_eq!(context.len(), 2);
    let institution_projection = context
        .iter()
        .find(|projection| projection.relation().relation() == "CapabilityInstitution")
        .unwrap();
    assert_eq!(institution_projection.selector().entity(), "Institution");
    assert_eq!(
        institution_projection.selector().field(),
        "InstitutionIdentityField"
    );
    assert_eq!(
        institution_projection.selector().value(),
        &InstitutionIdBinding::encode(&institution).unwrap()
    );
    let branch_projection = context
        .iter()
        .find(|projection| projection.relation().relation() == "CapabilityBranch")
        .unwrap();
    assert_eq!(branch_projection.selector().entity(), "Branch");
    assert_eq!(branch_projection.selector().field(), "BranchIdentityField");
    assert_eq!(
        branch_projection.selector().value(),
        &BranchIdBinding::encode(&branch).unwrap()
    );
}

fn write(field: &str) -> InstalledProgramTarget {
    InstalledProgramTarget::Write {
        entity: "CapabilityGrant".to_owned(),
        aspect: "CapabilityGrantRecord".to_owned(),
        path: worth_foundational::facade::CanonicalFieldPath::single(
            worth_foundational::facade::FieldKey::new(field).unwrap(),
        ),
    }
}

fn link(relation: &str, from: &str, to: &str) -> InstalledProgramTarget {
    InstalledProgramTarget::Link {
        relation: relation.to_owned(),
        from: from.to_owned(),
        to: to.to_owned(),
    }
}
