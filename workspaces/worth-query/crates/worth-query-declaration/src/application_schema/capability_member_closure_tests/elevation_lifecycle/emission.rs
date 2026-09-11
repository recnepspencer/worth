use super::*;

use crate::application_capability::{
    ApplicationCapabilityLifecycleEffect, ErasedApplicationCapabilityContract,
};
use crate::application_schema::{
    ApplicationEffectMarkerIdentity, ApplicationEffectRef, ApplicationOperationMarkerIdentity,
    ApplicationOperationProgramTarget, ApplicationRetainedEffectBinding, OperationEmits,
    WorthQueryPortableApplicationSchemaRecord,
};

pub(crate) struct ActivityEffect;
struct RejectingInput;
worth_query_portable_type!(RejectingInput => "worth.query.test.rejecting-input");
crate::worth_query_structured_value_binding!(
    pub(crate) ActivityPayloadBinding for String {
        identity: "worth.rust.string"
    }
);
crate::worth_query_structured_value_binding!(
    RejectingInputBinding for RejectingInput {
        identity: "worth.query.test.rejecting-input"
    }
);
struct RejectingRequestOperation;

impl ApplicationRetainedEffectBinding for ActivityPayloadBinding {
    fn retained_bytes(value: &Self::Value) -> u64 {
        std::mem::size_of::<String>() as u64 + value.capacity() as u64
    }
}

impl ApplicationEffectMarkerIdentity<Schema> for ActivityEffect {
    type PayloadBinding = ActivityPayloadBinding;
    const IDENTIFIER: &'static str = "ActivityEffect";
}

impl ApplicationOperationMarkerIdentity<Schema> for RejectingRequestOperation {
    type InputBinding = RejectingInputBinding;
    const IDENTIFIER: &'static str = "RejectingRequest";
}

#[derive(Clone, Copy)]
pub(super) enum EffectPosture {
    NotApplicable,
    Derived,
}

#[derive(Clone, Copy)]
pub(super) enum ResourceRelationPosture {
    NotApplicable,
    Governed,
}

impl OperationEmits<RequestOperation> for ActivityEffect {}
impl OperationEmits<RejectingRequestOperation> for ActivityEffect {}

impl ApplicationCapabilityLifecycleEffect<Schema, RequestOperation> for () {
    type Effect = ActivityEffect;
    type PayloadBinding = ActivityPayloadBinding;

    fn effect() -> ApplicationEffectRef<Schema, Self::Effect, String> {
        ApplicationEffectRef::from_declaration()
    }

    fn lifecycle_effect(&self) -> Option<String> {
        Some("estate:access".to_owned())
    }
}

impl ApplicationCapabilityLifecycleEffect<Schema, RejectingRequestOperation> for RejectingInput {
    type Effect = ActivityEffect;
    type PayloadBinding = ActivityPayloadBinding;

    fn effect() -> ApplicationEffectRef<Schema, Self::Effect, String> {
        ApplicationEffectRef::from_declaration()
    }

    fn lifecycle_effect(&self) -> Option<String> {
        None
    }
}

#[test]
fn lifecycle_effect_target_is_framework_owned() {
    let mut members = effect_members();
    members.push(ApplicationSchemaMember::OperationProgram {
        operation: "Request".to_owned(),
        target: ApplicationOperationProgramTarget::Emit {
            effect: "ActivityEffect".to_owned(),
        },
    });
    assert_eq!(
        build_from_members(members),
        Err(ApplicationSchemaDeclarationDenial::InvalidApplicationCapability)
    );
}

#[test]
fn lifecycle_resource_link_target_is_framework_owned() {
    let mut members = effect_members();
    members.push(ApplicationSchemaMember::OperationProgram {
        operation: "Request".to_owned(),
        target: ApplicationOperationProgramTarget::Link {
            relation: "ElevationResource".to_owned(),
            from: "Elevation".to_owned(),
            to: "Resource".to_owned(),
        },
    });
    assert_eq!(
        build_from_members(members),
        Err(ApplicationSchemaDeclarationDenial::InvalidApplicationCapability)
    );
}

#[test]
fn lifecycle_effect_requires_the_exact_declared_payload_type() {
    let mut members = effect_members();
    let effect = members.iter_mut().find(|member| {
        matches!(member, ApplicationSchemaMember::Effect { effect, .. } if effect == "ActivityEffect")
    });
    let Some(ApplicationSchemaMember::Effect { payload_type, .. }) = effect else {
        panic!("fixture must declare the lifecycle effect")
    };
    *payload_type =
        crate::portable_identity::WorthQueryPortableTypeIdentity::declared("worth.rust.u64");
    assert_eq!(
        build_from_members(members),
        Err(ApplicationSchemaDeclarationDenial::InvalidApplicationCapability)
    );
}

#[test]
fn bound_lifecycle_effect_derives_only_from_the_typed_input() {
    let transition = request_transition();
    let binding = transition.lifecycle_effect().expect("effect is bound");
    let derived = binding
        .derive_from_retained_input(&() as &dyn std::any::Any)
        .expect("typed input derives one payload");
    assert_eq!(derived.effect(), "ActivityEffect");
    assert_eq!(derived.payload_type(), "worth.rust.string");
    assert!(binding
        .derive_from_retained_input(&1_u64 as &dyn std::any::Any)
        .is_none());
}

#[test]
fn portable_lifecycle_effect_retains_meaning_without_retaining_derivation() {
    let typed = lifecycle_contract(EffectPosture::Derived, ResourceRelationPosture::Governed);
    let portable_parts = typed.parts();
    let reconstructed =
        ErasedApplicationCapabilityContract::from_untrusted_parts(portable_parts.clone());
    let typed_transition = typed
        .elevation()
        .definition()
        .unwrap()
        .lifecycle()
        .request();
    let reconstructed_transition = reconstructed
        .elevation()
        .definition()
        .unwrap()
        .lifecycle()
        .request();

    assert_eq!(reconstructed, typed);
    assert_eq!(reconstructed.parts(), portable_parts);
    assert!(typed_transition
        .lifecycle_effect()
        .unwrap()
        .derive_from_retained_input(&() as &dyn std::any::Any)
        .is_some());
    assert!(reconstructed_transition
        .lifecycle_effect()
        .unwrap()
        .derive_from_retained_input(&() as &dyn std::any::Any)
        .is_none());
}

#[test]
fn portable_schema_carriage_strips_the_live_lifecycle_recipe() {
    let record = WorthQueryPortableApplicationSchemaRecord::from_untrusted_parts(
        super::super::super::WorthQueryPortableApplicationSchemaParts {
            owner: "WORTH.tests".to_owned(),
            name: "callback-free-capability".to_owned(),
            major: 1,
            minor: 0,
            members: vec![ApplicationSchemaMember::ApplicationCapability {
                contract: lifecycle_contract(
                    EffectPosture::Derived,
                    ResourceRelationPosture::Governed,
                ),
            }],
            contributions: Vec::new(),
        },
    );
    let binding = record
        .members()
        .iter()
        .find_map(|member| {
            let ApplicationSchemaMember::ApplicationCapability { contract } = member else {
                return None;
            };
            contract
                .elevation()
                .definition()
                .and_then(|elevation| elevation.lifecycle().request().lifecycle_effect())
        })
        .expect("portable schema retains descriptive lifecycle-effect meaning");

    assert_eq!(binding.effect(), "ActivityEffect");
    assert!(binding
        .derive_from_retained_input(&() as &dyn std::any::Any)
        .is_none());
}

#[test]
fn typed_lifecycle_effect_rejection_derives_no_payload() {
    let transition = ApplicationCapabilityTransitionBinding::from_references_with_lifecycle_effect(
        ApplicationCapabilityRef::<Schema, RequestCapability>::from_schema_identifier(
            "RequestCapability",
        ),
        ApplicationOperationRef::<Schema, RejectingRequestOperation, RejectingInput>::from_declaration(),
    );
    let binding = transition.lifecycle_effect().expect("effect is bound");

    assert!(binding
        .derive_from_retained_input(&RejectingInput as &dyn std::any::Any)
        .is_none());
}

fn effect_members() -> Vec<ApplicationSchemaMember> {
    let contract = lifecycle_contract(EffectPosture::Derived, ResourceRelationPosture::Governed);
    let mut members = elevation_members(contract);
    members.push(relation_member(
        "ElevationResource",
        "Elevation",
        "Resource",
    ));
    members.push(ApplicationSchemaMember::Effect {
        effect: "ActivityEffect".to_owned(),
        payload_type: crate::portable_identity::WorthQueryPortableTypeIdentity::declared(
            "worth.rust.string",
        ),
    });
    members
}

pub(super) fn lifecycle_contract(
    effect: EffectPosture,
    resource_relation: ResourceRelationPosture,
) -> ErasedApplicationCapabilityContract {
    let lifecycle = ApplicationCapabilityElevationLifecycleDefinition::new(
        ApplicationCapabilityContextEntitySlotBinding::from_reference(context_slot::<
            ElevationSlot,
            Elevation,
        >(
            "ElevationSlot",
            "Elevation",
        )),
        ApplicationCapabilityContextEntitySlotBinding::from_reference(context_slot::<
            ReviewSlot,
            Review,
        >(
            "ReviewSlot", "Review"
        )),
        match effect {
            EffectPosture::NotApplicable => {
                transition_binding::<RequestCapability, RequestOperation>(
                    "RequestCapability",
                    "Request",
                )
            }
            EffectPosture::Derived => request_transition(),
        },
        transition_binding::<ApproveCapability, ApproveOperation>("ApproveCapability", "Approve"),
        transition_binding::<RevokeCapability, RevokeOperation>("RevokeCapability", "Revoke"),
        transition_binding::<CompleteReviewCapability, CompleteReviewOperation>(
            "CompleteReviewCapability",
            "CompleteReview",
        ),
    );
    let elevation = elevation_definition_with_lifecycle(
        StatePosture::Distinct,
        ReviewPosture::Distinct,
        std::time::Duration::from_secs(1_200),
        lifecycle,
    );
    let elevation = match resource_relation {
        ResourceRelationPosture::NotApplicable => elevation,
        ResourceRelationPosture::Governed => {
            elevation.with_resource_relation(relation::<ElevationResource, Elevation, Resource>(
                "ElevationResource",
                "Elevation",
                "Resource",
            ))
        }
    };
    ApplicationCapabilityContractBuilder::new(
        ApplicationCapabilityRef::<Schema, Capability>::from_schema_identifier("Capability"),
        operation::<Operation>("Operation"),
        ApplicationEntityRef::<Schema, Grant>::from_schema_identifier("Grant"),
    )
    .target(target_definition(false, false))
    .constraints(constraint_definition())
    .delegation(delegation_definition())
    .composition(composition(true))
    .elevation(ApplicationCapabilityElevationRule::governed(elevation))
    .build()
    .erased()
    .clone()
}

fn request_transition() -> ApplicationCapabilityTransitionBinding {
    ApplicationCapabilityTransitionBinding::from_references_with_lifecycle_effect(
        ApplicationCapabilityRef::<Schema, RequestCapability>::from_schema_identifier(
            "RequestCapability",
        ),
        operation::<RequestOperation>("Request"),
    )
}
