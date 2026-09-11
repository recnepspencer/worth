use super::*;
use crate::application_capability::ApplicationCapabilityDelegationActivationDefinition;
use crate::application_schema::ApplicationOperationProgramTarget;

struct ActivationOperation;
struct Identity;
struct ActivationContextRelation;
struct OtherCapability;
struct OtherAction;
declare_u64_field!(Identity, OtherAction);

impl ApplicationOperationMarkerIdentity<Schema> for ActivationOperation {
    type InputBinding = UnitOperationInputBinding;
    const IDENTIFIER: &'static str = "Activate";
}

#[test]
fn activation_operation_installs_without_an_application_program_inventory() {
    assert_eq!(build_from_members(activation_members()), Ok(()));
}

#[test]
fn selected_target_derives_the_complete_exact_activation_program() {
    let context = relation::<ActivationContextRelation, Grant, Resource>(
        "ActivationContextRelation",
        "Grant",
        "Resource",
    );
    let contract = activated_contract(context);
    let derived = crate::application_capability::
        application_capability_delegation_activation_program_targets(&contract)
        .expect("activated capability has target-owned effects");
    let mut expected = exact_program();
    expected.sort();
    assert_eq!(derived, expected);
}

#[test]
fn application_cannot_declare_the_activation_create() {
    assert_explicit_activation_target_denied(ApplicationOperationProgramTarget::Create {
        entity: "Grant".to_owned(),
    });
}

#[test]
fn application_cannot_declare_an_activation_write() {
    assert_explicit_activation_target_denied(write("Status"));
}

#[test]
fn application_cannot_declare_an_activation_link() {
    assert_explicit_activation_target_denied(link("Parent", "Grant", "Grant"));
}

#[test]
fn activation_program_rejects_an_application_owned_extra_effect() {
    assert_explicit_activation_target_denied(ApplicationOperationProgramTarget::Delete {
        entity: "Grant".to_owned(),
    });
}

#[test]
fn shared_activation_operation_unions_every_target_contract_axis() {
    let context = relation::<ActivationContextRelation, Grant, Resource>(
        "ActivationContextRelation",
        "Grant",
        "Resource",
    );
    let mut members = activation_members();
    members.push(field_member("OtherAction"));
    members.push(ApplicationSchemaMember::ApplicationCapability {
        contract: second_activated_contract(context.clone()),
    });
    assert_eq!(build_from_members(members), Ok(()));
    let second = second_activated_contract(context);
    let derived = crate::application_capability::
        application_capability_delegation_activation_program_targets(&second)
        .expect("second activated target derives its own exact effects");
    assert!(derived.contains(&write("OtherAction")));
}

#[test]
fn activation_context_relations_are_canonical_capability_meaning() {
    let baseline = activated_contract(relation::<ActivationContextRelation, Grant, Resource>(
        "ActivationContextRelation",
        "Grant",
        "Resource",
    ));
    let changed = activated_contract(relation::<ActivationContextRelation, Grant, Resource>(
        "ChangedActivationContextRelation",
        "Grant",
        "Resource",
    ));
    assert_ne!(
        crate::application_capability::application_capability_canonical_components(&baseline),
        crate::application_capability::application_capability_canonical_components(&changed)
    );
}

#[test]
fn activation_context_relation_must_leave_the_created_grant() {
    let contract = activated_contract(relation::<ActivationContextRelation, Resource, Grant>(
        "ActivationContextRelation",
        "Resource",
        "Grant",
    ));
    let mut members = activation_members_for(contract);
    members.push(relation_member(
        "ActivationContextRelation",
        "Resource",
        "Grant",
    ));
    assert_eq!(
        build_from_members(members),
        Err(ApplicationSchemaDeclarationDenial::InvalidApplicationCapability)
    );
}

fn assert_explicit_activation_target_denied(target: ApplicationOperationProgramTarget) {
    let mut members = activation_members();
    members.push(ApplicationSchemaMember::OperationProgram {
        operation: "Activate".to_owned(),
        target,
    });
    assert_activation_program_denial(members);
}

fn assert_activation_program_denial(members: Vec<ApplicationSchemaMember>) {
    assert_eq!(
        build_from_members(members),
        Err(
            ApplicationSchemaDeclarationDenial::InvalidApplicationCapabilityDelegationActivationProgram
        )
    );
}

fn activation_members() -> Vec<ApplicationSchemaMember> {
    let context = relation::<ActivationContextRelation, Grant, Resource>(
        "ActivationContextRelation",
        "Grant",
        "Resource",
    );
    let mut members = activation_members_for(activated_contract(context));
    members.push(relation_member(
        "ActivationContextRelation",
        "Grant",
        "Resource",
    ));
    members
}

fn activation_members_for(contract: ErasedContract) -> Vec<ApplicationSchemaMember> {
    let mut members = members(contract);
    members.push(field_member("Identity"));
    members.push(ApplicationSchemaMember::Operation {
        operation: "Activate".to_owned(),
        input_type: crate::portable_identity::WorthQueryPortableTypeIdentity::declared(
            "worth.rust.unit",
        ),
    });
    members
}

fn activated_contract(context_relation: ApplicationCapabilityRelationBinding) -> ErasedContract {
    ApplicationCapabilityContractBuilder::new(
        ApplicationCapabilityRef::<Schema, Capability>::from_schema_identifier("Capability"),
        ApplicationOperationRef::<Schema, Operation, ()>::from_declaration(),
        ApplicationEntityRef::<Schema, Grant>::from_schema_identifier("Grant"),
    )
    .target(target_definition(false, false))
    .constraints(constraint_definition())
    .delegation(
        delegation_definition().with_activation(
            ApplicationCapabilityDelegationActivationDefinition::new(
                ApplicationOperationRef::<Schema, ActivationOperation, ()>::from_declaration(),
                binding::<Identity>("Identity"),
            )
            .with_context_relations([context_relation]),
        ),
    )
    .composition(composition(true))
    .elevation(ApplicationCapabilityElevationRule::not_applicable())
    .build()
    .erased()
    .clone()
}

fn second_activated_contract(
    context_relation: ApplicationCapabilityRelationBinding,
) -> ErasedContract {
    let target = ApplicationCapabilityTargetDefinition::new(
        ApplicationCapabilityValueBinding::new(field::<OtherAction>("OtherAction"), encoded(1_u64)),
        relation::<ResourceRelation, Grant, Resource>("ResourceRelation", "Grant", "Resource"),
        ApplicationCapabilityRelationDimension::Bound(relation::<ScopedRelation, Grant, Resource>(
            "ScopedRelation",
            "Grant",
            "Resource",
        )),
        ApplicationCapabilityFieldDimension::bound(field::<Field>("Field")),
        ApplicationCapabilityValueBinding::new(field::<Purpose>("Purpose"), encoded(1_u64)),
    );
    ApplicationCapabilityContractBuilder::new(
        ApplicationCapabilityRef::<Schema, OtherCapability>::from_schema_identifier(
            "OtherCapability",
        ),
        ApplicationOperationRef::<Schema, Operation, ()>::from_declaration(),
        ApplicationEntityRef::<Schema, Grant>::from_schema_identifier("Grant"),
    )
    .target(target)
    .constraints(constraint_definition())
    .delegation(
        delegation_definition().with_activation(
            ApplicationCapabilityDelegationActivationDefinition::new(
                ApplicationOperationRef::<Schema, ActivationOperation, ()>::from_declaration(),
                binding::<Identity>("Identity"),
            )
            .with_context_relations([context_relation]),
        ),
    )
    .composition(composition(true))
    .elevation(ApplicationCapabilityElevationRule::not_applicable())
    .build()
    .erased()
    .clone()
}

fn exact_program() -> Vec<ApplicationOperationProgramTarget> {
    let mut targets = vec![ApplicationOperationProgramTarget::Create {
        entity: "Grant".to_owned(),
    }];
    targets.extend(
        [
            "Identity",
            "Action",
            "Purpose",
            "Field",
            "Amount",
            "Workflow",
            "Status",
            "ValidFrom",
            "ValidThrough",
            "DelegationLimit",
        ]
        .into_iter()
        .map(write),
    );
    targets.extend([
        link("ResourceRelation", "Grant", "Resource"),
        link("ScopedRelation", "Grant", "Resource"),
        link("Parent", "Grant", "Grant"),
        link("Grantor", "Principal", "Grant"),
        link("Grantee", "Principal", "Grant"),
        link("ActivationContextRelation", "Grant", "Resource"),
    ]);
    targets
}

fn write(field: &str) -> ApplicationOperationProgramTarget {
    ApplicationOperationProgramTarget::Write {
        entity: "Grant".to_owned(),
        aspect: "Facts".to_owned(),
        field: field.to_owned(),
    }
}

fn link(relation: &str, from: &str, to: &str) -> ApplicationOperationProgramTarget {
    ApplicationOperationProgramTarget::Link {
        relation: relation.to_owned(),
        from: from.to_owned(),
        to: to.to_owned(),
    }
}

fn encoded(
    value: u64,
) -> crate::application_schema::ApplicationEncodedScalarValue<
    crate::application_schema::U64ApplicationValueBinding,
> {
    crate::application_schema::ApplicationEncodedScalarValue::try_new(value).unwrap()
}
