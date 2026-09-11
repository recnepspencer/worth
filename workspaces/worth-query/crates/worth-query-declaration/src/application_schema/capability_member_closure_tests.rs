use super::{
    capability_member_closure::validate_application_capability_members,
    ApplicationAuthorizationPathBuilder, ApplicationEntityRef, ApplicationOperationMarkerIdentity,
    ApplicationOperationRef, ApplicationRelationRef, ApplicationSchemaDeclarationBuilder,
    ApplicationSchemaDeclarationDenial, ApplicationSchemaMember,
};
use crate::application_capability::{
    ApplicationCapabilityAcceptedValues, ApplicationCapabilityActorComposition,
    ApplicationCapabilityAllowRule, ApplicationCapabilityCardinalityDimension,
    ApplicationCapabilityComposition, ApplicationCapabilityConflictRule,
    ApplicationCapabilityConstraintDefinition, ApplicationCapabilityContextEntitySlotRef,
    ApplicationCapabilityContextRef, ApplicationCapabilityContractBuilder,
    ApplicationCapabilityCurrentnessDefinition, ApplicationCapabilityDecisionComposition,
    ApplicationCapabilityDelegationDefinition, ApplicationCapabilityDelegationRule,
    ApplicationCapabilityDenyRule, ApplicationCapabilityDisclosureRule,
    ApplicationCapabilityDistinctActorRule, ApplicationCapabilityElevationRule,
    ApplicationCapabilityFieldDimension, ApplicationCapabilityGraphClause,
    ApplicationCapabilityGraphRule, ApplicationCapabilityPathContextAnchor,
    ApplicationCapabilityPropagationComposition, ApplicationCapabilityProvenanceRef,
    ApplicationCapabilityRef, ApplicationCapabilityRelationBinding,
    ApplicationCapabilityRelationDimension, ApplicationCapabilityScopeGuard,
    ApplicationCapabilitySeparationOfDutyRule, ApplicationCapabilityTargetDefinition,
    ApplicationCapabilityValidityDefinition, ApplicationCapabilityValidityTimeline,
    ApplicationCapabilityValueBinding, ApplicationCapabilityWorkflowDefinition,
};
use worth_foundational::facade::ScalarAspectType;
#[macro_use]
mod field_binding_fixture;
use field_binding_fixture::{binding, encoded, field, resource_binding};
pub(crate) struct Schema;
struct Capability;
struct Operation;
struct Grant;
struct Resource;
struct Principal;
struct Facts;
struct ResourceFacts;
struct Action;
struct Purpose;
struct Field;
struct Amount;
struct Workflow;
struct ResourceWorkflow;
struct Status;
struct ValidFrom;
struct ValidThrough;
struct DelegationLimit;
struct ResourceRelation;
struct WrongResourceRelation;
struct ScopedRelation;
struct PrincipalResource;
struct Parent;
struct Grantor;
struct Grantee;
struct Context;
struct Provenance;
struct ResourceSlot;
struct MissingResourceSlot;
struct OtherContext;
struct OtherResourceSlot;

declare_u64_field!(
    Action,
    Purpose,
    Field,
    Amount,
    Workflow,
    ResourceWorkflow,
    Status,
    ValidFrom,
    ValidThrough,
    DelegationLimit,
);
crate::worth_query_structured_value_binding!(
    UnitOperationInputBinding for () {
        identity: "worth.rust.unit"
    }
);

impl ApplicationOperationMarkerIdentity<Schema> for Operation {
    type InputBinding = UnitOperationInputBinding;
    const IDENTIFIER: &'static str = "Operation";
}
type ErasedContract = crate::application_capability::ErasedApplicationCapabilityContract;

mod capability_revocation;
#[path = "capability_member_closure_tests/context_anchors.rs"]
mod context_anchors;
#[path = "capability_member_closure_tests/contract_validation.rs"]
mod contract_validation;
mod declared_dimensions;
mod delegation_activation;
#[path = "capability_member_closure_tests/elevation_lifecycle.rs"]
mod elevation_lifecycle;
#[path = "capability_member_closure_tests/fixture_members.rs"]
mod fixture_members;
#[path = "capability_member_closure_tests/population_budget.rs"]
mod population_budget;
#[path = "capability_member_closure_tests/portable_reconstruction.rs"]
mod portable_reconstruction;

use fixture_members::{field_member, members, relation_member};

fn build_from_members(
    members: Vec<ApplicationSchemaMember>,
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    validate_application_capability_members(&members)
}

fn contract(
    wrong_resource_topology: bool,
    changed_purpose: bool,
    disclosure: bool,
) -> ErasedContract {
    contract_with_name_and_composition(
        "Capability",
        wrong_resource_topology,
        changed_purpose,
        composition(disclosure),
    )
}

fn contract_with_composition(
    wrong_resource_topology: bool,
    changed_purpose: bool,
    composition: ApplicationCapabilityComposition,
) -> ErasedContract {
    contract_with_name_and_composition(
        "Capability",
        wrong_resource_topology,
        changed_purpose,
        composition,
    )
}

fn contract_with_name_and_composition(
    capability_name: &'static str,
    wrong_resource_topology: bool,
    changed_purpose: bool,
    composition: ApplicationCapabilityComposition,
) -> ErasedContract {
    contract_with_identity_and_composition(
        capability_name,
        capability_name,
        wrong_resource_topology,
        changed_purpose,
        composition,
    )
}

fn contract_with_identity_and_composition(
    capability_name: &'static str,
    capability_identity: &'static str,
    wrong_resource_topology: bool,
    changed_purpose: bool,
    composition: ApplicationCapabilityComposition,
) -> ErasedContract {
    ApplicationCapabilityContractBuilder::new(
        ApplicationCapabilityRef::<Schema, Capability>::from_test_declaration(
            capability_name,
            crate::portable_identity::WorthQueryPortableTypeIdentity::declared(capability_identity),
        ),
        ApplicationOperationRef::<Schema, Operation, ()>::from_declaration(),
        ApplicationEntityRef::<Schema, Grant>::from_schema_identifier("Grant"),
    )
    .target(target_definition(wrong_resource_topology, changed_purpose))
    .constraints(constraint_definition())
    .delegation(delegation_definition())
    .composition(composition)
    .elevation(ApplicationCapabilityElevationRule::not_applicable())
    .build()
    .erased()
    .clone()
}

fn contract_with_capability_ref<CapabilityMarker>(
    capability: ApplicationCapabilityRef<Schema, CapabilityMarker>,
) -> ErasedContract {
    ApplicationCapabilityContractBuilder::new(
        capability,
        ApplicationOperationRef::<Schema, Operation, ()>::from_declaration(),
        ApplicationEntityRef::<Schema, Grant>::from_schema_identifier("Grant"),
    )
    .target(target_definition(false, false))
    .constraints(constraint_definition())
    .delegation(delegation_definition())
    .composition(composition(true))
    .elevation(ApplicationCapabilityElevationRule::not_applicable())
    .build()
    .erased()
    .clone()
}

fn target_definition(
    wrong_resource_topology: bool,
    changed_purpose: bool,
) -> ApplicationCapabilityTargetDefinition {
    let resource = if wrong_resource_topology {
        relation::<WrongResourceRelation, Principal, Resource>(
            "WrongResourceRelation",
            "Principal",
            "Resource",
        )
    } else {
        relation::<ResourceRelation, Grant, Resource>("ResourceRelation", "Grant", "Resource")
    };
    ApplicationCapabilityTargetDefinition::new(
        ApplicationCapabilityValueBinding::new(field::<Action>("Action"), encoded(1_u64)),
        resource,
        ApplicationCapabilityRelationDimension::Bound(relation::<ScopedRelation, Grant, Resource>(
            "ScopedRelation",
            "Grant",
            "Resource",
        )),
        ApplicationCapabilityFieldDimension::bound(field::<Field>("Field")),
        ApplicationCapabilityValueBinding::new(
            field::<Purpose>("Purpose"),
            encoded(if changed_purpose { 2_u64 } else { 1_u64 }),
        ),
    )
}

fn constraint_definition() -> ApplicationCapabilityConstraintDefinition {
    ApplicationCapabilityConstraintDefinition::new(
        ApplicationCapabilityFieldDimension::bound(field::<Amount>("Amount")),
        ApplicationCapabilityCardinalityDimension::One,
        ApplicationCapabilityCurrentnessDefinition::new(
            ApplicationCapabilityValueBinding::new(field::<Status>("Status"), encoded(1_u64)),
            ApplicationCapabilityWorkflowDefinition::new(
                binding::<Workflow>("Workflow"),
                resource_binding::<ResourceWorkflow>("ResourceWorkflow"),
            ),
            ApplicationCapabilityValidityDefinition::new(
                ApplicationCapabilityValidityTimeline::UnixEpochSeconds,
                binding::<ValidFrom>("ValidFrom"),
                binding::<ValidThrough>("ValidThrough"),
            ),
        ),
        ApplicationCapabilityContextRef::<Schema, Context>::from_schema_identifier("Context"),
    )
}

fn delegation_definition() -> ApplicationCapabilityDelegationDefinition {
    ApplicationCapabilityDelegationDefinition::new(
        relation::<Parent, Grant, Grant>("Parent", "Grant", "Grant"),
        relation::<Grantor, Principal, Grant>("Grantor", "Principal", "Grant"),
        relation::<Grantee, Principal, Grant>("Grantee", "Principal", "Grant"),
        binding::<DelegationLimit>("DelegationLimit"),
        ApplicationCapabilityProvenanceRef::<Schema, Provenance>::from_schema_identifier(
            "Provenance",
        ),
    )
}

fn resource_slot<ContextMarker, SlotMarker>(
    context: &'static str,
    slot: &'static str,
) -> ApplicationCapabilityContextEntitySlotRef<Schema, ContextMarker, SlotMarker, Resource> {
    ApplicationCapabilityContextEntitySlotRef::from_schema_identifiers(
        ApplicationCapabilityContextRef::from_schema_identifier(context),
        slot,
        ApplicationEntityRef::from_schema_identifier("Resource"),
    )
}

fn anchored_composition(
    anchor: ApplicationCapabilityPathContextAnchor,
) -> ApplicationCapabilityComposition {
    let allow = ApplicationCapabilityGraphRule::any([ApplicationCapabilityGraphClause::new(
        principal_resource_path(),
    )
    .anchored([anchor])]);
    ApplicationCapabilityComposition::new(
        ApplicationCapabilityDecisionComposition::new(
            ApplicationCapabilityAllowRule::new(allow),
            ApplicationCapabilityDenyRule::not_applicable(),
            ApplicationCapabilityConflictRule::not_applicable(),
        ),
        ApplicationCapabilityActorComposition::new(
            ApplicationCapabilitySeparationOfDutyRule::not_applicable(),
            ApplicationCapabilityDistinctActorRule::not_applicable(),
        ),
        ApplicationCapabilityPropagationComposition::new(
            ApplicationCapabilityDelegationRule::forbidden(),
            ApplicationCapabilityDisclosureRule::permit([
                ApplicationCapabilityScopeGuard::requiring([
                    ApplicationCapabilityAcceptedValues::one_of(
                        field::<Field>("Field"),
                        [encoded(1_u64)],
                    ),
                ]),
            ]),
        ),
    )
}

fn composition(disclosure: bool) -> ApplicationCapabilityComposition {
    let allow = ApplicationCapabilityGraphRule::any([ApplicationCapabilityGraphClause::new(
        principal_resource_path(),
    )]);
    let disclosure = if disclosure {
        ApplicationCapabilityDisclosureRule::permit([ApplicationCapabilityScopeGuard::requiring([
            ApplicationCapabilityAcceptedValues::one_of(field::<Field>("Field"), [encoded(1_u64)]),
        ])])
    } else {
        ApplicationCapabilityDisclosureRule::not_applicable()
    };
    ApplicationCapabilityComposition::new(
        ApplicationCapabilityDecisionComposition::new(
            ApplicationCapabilityAllowRule::new(allow),
            ApplicationCapabilityDenyRule::not_applicable(),
            ApplicationCapabilityConflictRule::not_applicable(),
        ),
        ApplicationCapabilityActorComposition::new(
            ApplicationCapabilitySeparationOfDutyRule::not_applicable(),
            ApplicationCapabilityDistinctActorRule::not_applicable(),
        ),
        ApplicationCapabilityPropagationComposition::new(
            ApplicationCapabilityDelegationRule::forbidden(),
            disclosure,
        ),
    )
}

fn principal_resource_path() -> crate::application_schema::ApplicationAuthorizationPath {
    ApplicationAuthorizationPathBuilder::from_principal(
        ApplicationEntityRef::<Schema, Principal>::from_schema_identifier("Principal"),
    )
    .forward(ApplicationRelationRef::<
        Schema,
        PrincipalResource,
        Principal,
        Resource,
    >::from_schema_identifiers(
        "PrincipalResource",
        "Principal",
        "Resource",
        crate::facade::application_schema::ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
    ))
    .allow(ApplicationEntityRef::<Schema, Resource>::from_schema_identifier("Resource"))
}

fn relation<RelationMarker, From, To>(
    name: &'static str,
    from: &'static str,
    to: &'static str,
) -> ApplicationCapabilityRelationBinding {
    ApplicationCapabilityRelationBinding::from_reference(ApplicationRelationRef::<
        Schema,
        RelationMarker,
        From,
        To,
    >::from_schema_identifiers(
        name,
        from,
        to,
        crate::facade::application_schema::ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
    ))
}
