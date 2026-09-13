use super::*;
use crate::application_capability::{
    ApplicationCapabilityDelegationActivationDefinition, ApplicationCapabilityRevocationDefinition,
};
use crate::application_schema::{
    ApplicationOperationDecisionReadTarget, ApplicationOperationProgramTarget,
};

struct RevocationOperation;
struct Identity;
declare_u64_field!(Identity);

impl ApplicationOperationMarkerIdentity<Schema> for RevocationOperation {
    type InputBinding = UnitOperationInputBinding;
    const IDENTIFIER: &'static str = "Revoke";
}

#[test]
fn revocation_operation_installs_without_application_owned_reads_or_effects() {
    assert_eq!(build_from_members(revocation_members(2)), Ok(()));
}

#[test]
fn revocation_contract_derives_exact_decision_reads_and_status_write() {
    let contract = revocable_contract(2);
    let reads =
        crate::application_capability::application_capability_revocation_decision_reads(&contract)
            .unwrap();
    assert_eq!(
        reads,
        vec![
            field_read("Identity"),
            field_read("Status"),
            ApplicationOperationDecisionReadTarget::Relation {
                relation: "ResourceRelation".to_owned(),
                from: "Grant".to_owned(),
                to: "Resource".to_owned(),
            },
        ]
    );
    assert_eq!(
        crate::application_capability::application_capability_revocation_program_target(&contract),
        Some(write("Status"))
    );
}

#[test]
fn application_cannot_redeclare_revocation_reads_or_effects() {
    let mut with_read = revocation_members(2);
    with_read.push(ApplicationSchemaMember::OperationDecisionRead {
        operation: "Revoke".to_owned(),
        target: field_read("Status"),
    });
    assert_eq!(
        build_from_members(with_read),
        Err(ApplicationSchemaDeclarationDenial::InvalidApplicationCapability)
    );

    let mut with_effect = revocation_members(2);
    with_effect.push(ApplicationSchemaMember::OperationProgram {
        operation: "Revoke".to_owned(),
        target: write("Status"),
    });
    assert_eq!(
        build_from_members(with_effect),
        Err(ApplicationSchemaDeclarationDenial::InvalidApplicationCapability)
    );
}

#[test]
fn revoked_status_is_canonical_capability_meaning() {
    assert_ne!(
        crate::application_capability::application_capability_canonical_components(
            &revocable_contract(2)
        ),
        crate::application_capability::application_capability_canonical_components(
            &revocable_contract(3)
        )
    );
}

#[test]
fn revoked_status_must_be_distinct_from_active_status() {
    assert_eq!(
        build_from_members(revocation_members(1)),
        Err(ApplicationSchemaDeclarationDenial::InvalidApplicationCapability)
    );
}

#[test]
fn one_operation_cannot_be_both_delegation_activation_and_revocation() {
    let contract = ApplicationCapabilityContractBuilder::new(
        ApplicationCapabilityRef::<Schema, Capability>::from_schema_identifier("Capability"),
        ApplicationOperationRef::<Schema, Operation, ()>::from_declaration(),
        ApplicationEntityRef::<Schema, Grant>::from_schema_identifier("Grant"),
    )
    .target(target_definition(false, false))
    .constraints(constraint_definition())
    .delegation(
        delegation_definition()
            .with_activation(ApplicationCapabilityDelegationActivationDefinition::new(
                ApplicationOperationRef::<Schema, RevocationOperation, ()>::from_declaration(),
                binding::<Identity>("Identity"),
            ))
            .with_revocation(ApplicationCapabilityRevocationDefinition::new(
                ApplicationOperationRef::<Schema, RevocationOperation, ()>::from_declaration(),
                binding::<Identity>("Identity"),
                ApplicationCapabilityValueBinding::new(field::<Status>("Status"), encoded(2_u64)),
            )),
    )
    .composition(composition(true))
    .elevation(ApplicationCapabilityElevationRule::not_applicable())
    .build()
    .erased()
    .clone();
    assert_eq!(
        build_from_members(revocation_members_for(contract)),
        Err(ApplicationSchemaDeclarationDenial::InvalidApplicationCapability)
    );
}

fn revocation_members(revoked: u64) -> Vec<ApplicationSchemaMember> {
    revocation_members_for(revocable_contract(revoked))
}

fn revocation_members_for(contract: ErasedContract) -> Vec<ApplicationSchemaMember> {
    let mut members = members(contract);
    members.push(field_member("Identity"));
    members.push(ApplicationSchemaMember::Operation {
        operation: "Revoke".to_owned(),
        input_type: crate::portable_identity::WorthQueryPortableTypeIdentity::declared(
            "worth.rust.unit",
        ),
    });
    members
}

fn revocable_contract(revoked: u64) -> ErasedContract {
    ApplicationCapabilityContractBuilder::new(
        ApplicationCapabilityRef::<Schema, Capability>::from_schema_identifier("Capability"),
        ApplicationOperationRef::<Schema, Operation, ()>::from_declaration(),
        ApplicationEntityRef::<Schema, Grant>::from_schema_identifier("Grant"),
    )
    .target(target_definition(false, false))
    .constraints(constraint_definition())
    .delegation(delegation_definition().with_revocation(
        ApplicationCapabilityRevocationDefinition::new(
            ApplicationOperationRef::<Schema, RevocationOperation, ()>::from_declaration(),
            binding::<Identity>("Identity"),
            ApplicationCapabilityValueBinding::new(field::<Status>("Status"), encoded(revoked)),
        ),
    ))
    .composition(composition(true))
    .elevation(ApplicationCapabilityElevationRule::not_applicable())
    .build()
    .erased()
    .clone()
}

fn field_read(field: &str) -> ApplicationOperationDecisionReadTarget {
    ApplicationOperationDecisionReadTarget::Field {
        entity: "Grant".to_owned(),
        aspect: "Facts".to_owned(),
        field: field.to_owned(),
    }
}

fn write(field: &str) -> ApplicationOperationProgramTarget {
    ApplicationOperationProgramTarget::Write {
        entity: "Grant".to_owned(),
        aspect: "Facts".to_owned(),
        field: field.to_owned(),
    }
}

fn encoded(
    value: u64,
) -> crate::application_schema::ApplicationEncodedScalarValue<
    crate::application_schema::U64ApplicationValueBinding,
> {
    crate::application_schema::ApplicationEncodedScalarValue::try_new(value).unwrap()
}
