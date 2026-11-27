//! Standing tenure, extent status and self-grant refusals at installation.
use super::*;
use crate::application_capability::{
    application_capability_canonical_components, ApplicationCapabilityDelegationDepth,
};
use crate::application_schema::{
    fragment, ApplicationFieldBindingLocus, ApplicationOperationBodyCreation,
    ApplicationOperationBodyCreationLink,
    ApplicationOperationBodyCreationLinkDirection as Direction,
    ApplicationOperationBodyCreationLinkPosture, ApplicationOperationBodyCreationLinkSource,
    ApplicationOperationBodyProgram,
};
use crate::archive_decode_authority::ARCHIVE_DECODE;
use field_binding_fixture::CapabilityModel;

struct Shape {
    standing_currentness: bool,
    standing_holder: bool,
    extent: Vec<ApplicationCapabilityValueBinding>,
    forbid_self_grant: bool,
    delegation: ApplicationCapabilityDelegationRule,
}

impl Shape {
    fn standing() -> Self {
        Self {
            standing_currentness: true,
            standing_holder: true,
            extent: vec![resource_active()],
            forbid_self_grant: true,
            delegation: ApplicationCapabilityDelegationRule::forbidden(),
        }
    }
}

fn resource_active() -> ApplicationCapabilityValueBinding {
    ApplicationCapabilityValueBinding::new(
        fragment::FieldSelection::<CapabilityModel, Resource, ResourceFacts, u64>::at(
            "ResourceWorkflow",
        ),
        field_binding_fixture::encoded(1),
    )
    .unwrap()
}

fn grant_active() -> ApplicationCapabilityValueBinding {
    ApplicationCapabilityValueBinding::new(field("Status"), encoded(1_u64)).unwrap()
}

fn contract(shape: Shape) -> ErasedContract {
    let validity = ApplicationCapabilityValidityDefinition::new(
        ApplicationCapabilityValidityTimeline::UnixEpochSeconds,
        binding("ValidFrom"),
        binding("ValidThrough"),
    );
    let currentness = if shape.standing_currentness {
        ApplicationCapabilityCurrentnessDefinition::standing(grant_active(), validity)
    } else {
        ApplicationCapabilityCurrentnessDefinition::new(
            grant_active(),
            ApplicationCapabilityWorkflowDefinition::new(
                binding("Workflow"),
                resource_binding("ResourceWorkflow"),
            ),
            validity,
        )
    };
    let currentness = shape.extent.into_iter().fold(
        currentness,
        ApplicationCapabilityCurrentnessDefinition::require_active,
    );
    let grantee = relation::<Grantee, Principal, Grant>("Grantee", "Principal", "Grant");
    let provenance =
        ApplicationCapabilityProvenanceRef::<Schema, Provenance>::from_schema_identifier(
            "Provenance",
        );
    let mut delegation = if shape.standing_holder {
        ApplicationCapabilityDelegationDefinition::standing(grantee, provenance)
    } else {
        delegation_definition()
    };
    if shape.forbid_self_grant {
        delegation = delegation.forbid_self_grant();
    }
    ApplicationCapabilityContractBuilder::new(
        ApplicationCapabilityRef::<Schema, Capability>::from_test_declaration(
            "Capability",
            crate::portable_identity::WorthQueryPortableTypeIdentity::declared("Capability"),
        ),
        ApplicationOperationRef::<Schema, Operation, ()>::from_declaration(),
        ApplicationEntityRef::<Schema, Grant>::from_schema_identifier("Grant"),
    )
    .target(target_definition(false, false))
    .constraints(ApplicationCapabilityConstraintDefinition::new(
        ApplicationCapabilityFieldDimension::bound(field("Amount")).unwrap(),
        ApplicationCapabilityCardinalityDimension::One,
        currentness,
        ApplicationCapabilityContextRef::<Schema, Context>::from_schema_identifier("Context"),
    ))
    .delegation(delegation)
    .composition(composition_delegating(shape.delegation))
    .elevation(ApplicationCapabilityElevationRule::not_applicable())
    .build()
    .erased()
    .clone()
}

fn composition_delegating(
    delegation: ApplicationCapabilityDelegationRule,
) -> ApplicationCapabilityComposition {
    ApplicationCapabilityComposition::new(
        ApplicationCapabilityDecisionComposition::new(
            ApplicationCapabilityAllowRule::new(ApplicationCapabilityGraphRule::any([
                ApplicationCapabilityGraphClause::new(principal_resource_path()),
            ])),
            ApplicationCapabilityDenyRule::not_applicable(),
            ApplicationCapabilityConflictRule::not_applicable(),
        ),
        ApplicationCapabilityActorComposition::new(
            ApplicationCapabilitySeparationOfDutyRule::not_applicable(),
            ApplicationCapabilityDistinctActorRule::not_applicable(),
            crate::application_capability::ApplicationCapabilityRecordedActorRule::not_applicable(),
        ),
        ApplicationCapabilityPropagationComposition::new(
            delegation,
            ApplicationCapabilityDisclosureRule::permit([
                ApplicationCapabilityScopeGuard::requiring([
                    ApplicationCapabilityAcceptedValues::one_of(field("Field"), [encoded(1_u64)])
                        .unwrap(),
                ]),
            ]),
        ),
    )
}

fn tenure_denial() -> Result<(), ApplicationSchemaDeclarationDenial> {
    Err(
        ApplicationSchemaDeclarationDenial::InvalidApplicationCapabilityTenure {
            capability: "Capability".to_owned(),
        },
    )
}

fn extent_denial() -> Result<(), ApplicationSchemaDeclarationDenial> {
    Err(
        ApplicationSchemaDeclarationDenial::InvalidApplicationCapabilityExtentStatus {
            capability: "Capability".to_owned(),
        },
    )
}

#[test]
fn standing_grant_with_rooted_extent_and_self_grant_rule_installs() {
    assert_eq!(
        build_from_members(members(contract(Shape::standing()))),
        Ok(())
    );
}

#[test]
fn tenure_mixtures_and_delegable_standing_grants_refuse_with_a_named_denial() {
    let mixed_holder = Shape {
        standing_holder: false,
        forbid_self_grant: false,
        ..Shape::standing()
    };
    let mixed_currentness = Shape {
        standing_currentness: false,
        ..Shape::standing()
    };
    let delegable = Shape {
        delegation: ApplicationCapabilityDelegationRule::narrow_all_dimensions(
            ApplicationCapabilityDelegationDepth::new(2).unwrap(),
        ),
        ..Shape::standing()
    };
    let lineage_self_grant = Shape {
        standing_currentness: false,
        standing_holder: false,
        extent: Vec::new(),
        ..Shape::standing()
    };
    for shape in [
        mixed_holder,
        mixed_currentness,
        delegable,
        lineage_self_grant,
    ] {
        assert_eq!(
            build_from_members(members(contract(shape))),
            tenure_denial()
        );
    }
}

#[test]
fn extent_status_on_the_grant_or_a_repeated_kind_refuses() {
    for extent in [
        vec![grant_active()],
        vec![resource_active(), resource_active()],
    ] {
        let shape = Shape {
            extent,
            ..Shape::standing()
        };
        assert_eq!(
            build_from_members(members(contract(shape))),
            extent_denial()
        );
    }
}

#[test]
fn declarative_body_linking_its_principal_as_a_forbidden_grantee_refuses() {
    let link = ApplicationOperationBodyCreationLink {
        relation: "Grantee".to_owned(),
        from: "Principal".to_owned(),
        to: "Grant".to_owned(),
        key: "holder".to_owned(),
        direction: Direction::ToScope,
        posture: ApplicationOperationBodyCreationLinkPosture::Append,
        source: ApplicationOperationBodyCreationLinkSource::Principal {
            mutation_binding: "Binding".to_owned(),
            principal_binding: "PrincipalBinding".to_owned(),
            identity: ApplicationFieldBindingLocus::new("Principal", "Facts", "Identity"),
        },
    };
    let creation = ApplicationOperationBodyCreation::from_untrusted(
        ARCHIVE_DECODE,
        "Grant".to_owned(),
        "grant".to_owned(),
        Vec::new(),
    )
    .with_untrusted_links(vec![link]);
    let mut members = members(contract(Shape::standing()));
    members.push(ApplicationSchemaMember::OperationBody {
        operation: "Operation".to_owned(),
        body: ApplicationOperationBodyProgram::from_untrusted_with_creation(
            ARCHIVE_DECODE,
            Vec::new(),
            Some(creation),
        ),
    });
    assert_eq!(
        build_from_members(members),
        Err(
            ApplicationSchemaDeclarationDenial::SelfGrantingOperationBody {
                operation: "Operation".to_owned(),
            }
        )
    );
}

#[test]
fn standing_identity_names_its_tenure_and_workflow_identity_keeps_its_loci() {
    let loci = |contract: &ErasedContract| {
        application_capability_canonical_components(contract)
            .into_iter()
            .map(|component| component.locus().to_owned())
            .collect::<Vec<_>>()
    };
    let standing = loci(&contract(Shape::standing()));
    for expected in [
        "constraints.currentness.tenure",
        "constraints.currentness.extent-status.0.value",
        "delegation.lineage",
        "delegation.self-grant",
    ] {
        assert!(standing.iter().any(|locus| locus == expected), "{expected}");
    }
    assert!(!standing
        .iter()
        .any(|locus| locus.contains("workflow") || locus.contains("parent")));
    let workflow = loci(&super::contract(false, false, true));
    assert!(!workflow.iter().any(|locus| {
        locus.contains("tenure")
            || locus.contains("extent-status")
            || locus.contains("lineage")
            || locus.contains("self-grant")
    }));
    let order = |prefix: &str| workflow.iter().position(|locus| locus.starts_with(prefix));
    assert!(
        order("constraints.currentness.active-status")
            < order("constraints.currentness.workflow.grant")
    );
    assert!(
        order("constraints.currentness.workflow.resource")
            < order("constraints.currentness.validity")
    );
    assert!(order("delegation.parent") < order("delegation.grantor"));
    assert!(order("delegation.grantee") < order("delegation.limit"));
    assert!(order("delegation.limit") < order("delegation.provenance"));
}
