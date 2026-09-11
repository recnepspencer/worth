use worth_query_declaration::facade::{
    application_capability::{
        ApplicationCapabilityAcceptedValues, ApplicationCapabilityActorComposition,
        ApplicationCapabilityAllowRule, ApplicationCapabilityCardinalityDimension,
        ApplicationCapabilityComposition, ApplicationCapabilityConflictRule,
        ApplicationCapabilityConstraintDefinition, ApplicationCapabilityContextEntitySlotRef,
        ApplicationCapabilityContextRef, ApplicationCapabilityContract,
        ApplicationCapabilityContractBuilder, ApplicationCapabilityCurrentnessDefinition,
        ApplicationCapabilityDecisionComposition, ApplicationCapabilityDelegationDefinition,
        ApplicationCapabilityDelegationRule, ApplicationCapabilityDenyRule,
        ApplicationCapabilityDisclosureRule, ApplicationCapabilityDistinctActorRule,
        ApplicationCapabilityElevationRule, ApplicationCapabilityFieldDimension,
        ApplicationCapabilityGraphClause, ApplicationCapabilityGraphRequirement,
        ApplicationCapabilityGraphRule, ApplicationCapabilityPathContextAnchor,
        ApplicationCapabilityPropagationComposition, ApplicationCapabilityProvenanceRef,
        ApplicationCapabilityRef, ApplicationCapabilityRelationBinding,
        ApplicationCapabilityRelationDimension, ApplicationCapabilityScopeGuard,
        ApplicationCapabilitySeparationOfDutyRule, ApplicationCapabilityTargetDefinition,
        ApplicationCapabilityValidityDefinition, ApplicationCapabilityValidityTimeline,
        ApplicationCapabilityValueBinding, ApplicationCapabilityWorkflowDefinition,
        ErasedApplicationCapabilityContract,
    },
    application_schema::{
        ApplicationAuthorizationPath, ApplicationAuthorizationPathBuilder,
        ApplicationEncodedScalarValue, ApplicationEntityRef, ApplicationOperationRef,
        ApplicationRelationRef, U64ApplicationValueBinding,
    },
};

use super::canonical_basis::prepare_capability_basis;

mod axis;
mod budgets;
mod field_references;
mod portable_markers;
mod residue;

use axis::Axis;
use field_references::{field, field_binding, resource_field, resource_field_binding};
use portable_markers::*;

pub(crate) mod delegation_activation_fixture;
mod identity_axes;
mod warm_lookup;

fn contract(axis: Option<Axis>) -> ErasedApplicationCapabilityContract {
    match axis {
        Some(Axis::Context) => contract_with_axis(
            axis,
            ApplicationCapabilityContextRef::<Schema, ChangedContext>::from_declaration(),
            ApplicationCapabilityProvenanceRef::<Schema, Provenance>::from_declaration(),
        ),
        Some(Axis::Provenance) => contract_with_axis(
            axis,
            ApplicationCapabilityContextRef::<Schema, Context>::from_declaration(),
            ApplicationCapabilityProvenanceRef::<Schema, ChangedProvenance>::from_declaration(),
        ),
        _ => contract_with_axis(
            axis,
            ApplicationCapabilityContextRef::<Schema, Context>::from_declaration(),
            ApplicationCapabilityProvenanceRef::<Schema, Provenance>::from_declaration(),
        ),
    }
}

fn typed_contract() -> ApplicationCapabilityContract<Schema, Capability, Operation, ()> {
    ApplicationCapabilityContractBuilder::new(
        ApplicationCapabilityRef::<Schema, Capability>::from_declaration(),
        ApplicationOperationRef::<Schema, Operation, ()>::from_declaration(),
        ApplicationEntityRef::<Schema, Grant>::from_schema_identifier("Grant"),
    )
    .target(target(None))
    .constraints(constraints(
        None,
        ApplicationCapabilityContextRef::<Schema, Context>::from_declaration(),
    ))
    .delegation(delegation(
        None,
        ApplicationCapabilityProvenanceRef::<Schema, Provenance>::from_declaration(),
    ))
    .composition(composition(None))
    .elevation(ApplicationCapabilityElevationRule::not_applicable())
    .build()
}

fn contract_with_markers<ContextMarker, ProvenanceMarker>(
    context: ApplicationCapabilityContextRef<Schema, ContextMarker>,
    provenance: ApplicationCapabilityProvenanceRef<Schema, ProvenanceMarker>,
) -> ErasedApplicationCapabilityContract {
    contract_with_axis(None, context, provenance)
}

fn contract_with_axis<ContextMarker, ProvenanceMarker>(
    axis: Option<Axis>,
    context: ApplicationCapabilityContextRef<Schema, ContextMarker>,
    provenance: ApplicationCapabilityProvenanceRef<Schema, ProvenanceMarker>,
) -> ErasedApplicationCapabilityContract {
    let contract = ApplicationCapabilityContractBuilder::new(
        ApplicationCapabilityRef::<Schema, Capability>::from_declaration(),
        ApplicationOperationRef::<Schema, Operation, ()>::from_declaration(),
        ApplicationEntityRef::<Schema, Grant>::from_schema_identifier("Grant"),
    )
    .target(target(axis))
    .constraints(constraints(axis, context))
    .delegation(delegation(axis, provenance))
    .composition(composition(axis))
    .elevation(ApplicationCapabilityElevationRule::not_applicable())
    .build();
    contract.erased().clone()
}

fn contract_with_resource_relation_name(
    resource_name: &'static str,
) -> ErasedApplicationCapabilityContract {
    ApplicationCapabilityContractBuilder::new(
        ApplicationCapabilityRef::<Schema, Capability>::from_declaration(),
        ApplicationOperationRef::<Schema, Operation, ()>::from_declaration(),
        ApplicationEntityRef::<Schema, Grant>::from_schema_identifier("Grant"),
    )
    .target(target_with_resource_name(None, resource_name))
    .constraints(constraints(
        None,
        ApplicationCapabilityContextRef::<Schema, Context>::from_declaration(),
    ))
    .delegation(delegation(
        None,
        ApplicationCapabilityProvenanceRef::<Schema, Provenance>::from_declaration(),
    ))
    .composition(composition(None))
    .elevation(ApplicationCapabilityElevationRule::not_applicable())
    .build()
    .erased()
    .clone()
}

fn target(axis: Option<Axis>) -> ApplicationCapabilityTargetDefinition {
    let resource_name = if matches!(axis, Some(Axis::Resource)) {
        "ChangedResourceRelation"
    } else {
        "ResourceRelation"
    };
    target_with_resource_name(axis, resource_name)
}

fn target_with_resource_name(
    axis: Option<Axis>,
    resource_name: &'static str,
) -> ApplicationCapabilityTargetDefinition {
    let action_value = if matches!(axis, Some(Axis::Action)) {
        2
    } else {
        1
    };
    let purpose_value = if matches!(axis, Some(Axis::Purpose)) {
        2
    } else {
        1
    };
    ApplicationCapabilityTargetDefinition::new(
        ApplicationCapabilityValueBinding::new(field::<Action>(), encoded_u64(action_value)),
        relation::<ResourceRelation, Grant, Resource>(resource_name, "Grant", "Resource"),
        if matches!(axis, Some(Axis::Relation)) {
            ApplicationCapabilityRelationDimension::not_applicable()
        } else {
            ApplicationCapabilityRelationDimension::Bound(
                relation::<ScopedRelation, Grant, Resource>("ScopedRelation", "Grant", "Resource"),
            )
        },
        if matches!(axis, Some(Axis::Field)) {
            ApplicationCapabilityFieldDimension::not_applicable()
        } else {
            ApplicationCapabilityFieldDimension::bound(field::<Field>())
        },
        ApplicationCapabilityValueBinding::new(field::<Purpose>(), encoded_u64(purpose_value)),
    )
}

fn constraints<ContextMarker>(
    axis: Option<Axis>,
    context: ApplicationCapabilityContextRef<Schema, ContextMarker>,
) -> ApplicationCapabilityConstraintDefinition {
    let workflow = if matches!(axis, Some(Axis::Workflow)) {
        field_binding::<ChangedWorkflow>()
    } else {
        field_binding::<Workflow>()
    };
    let resource_workflow = if matches!(axis, Some(Axis::ResourceWorkflow)) {
        resource_field_binding::<ChangedResourceWorkflow>()
    } else {
        resource_field_binding::<ResourceWorkflow>()
    };
    let valid_from = if matches!(axis, Some(Axis::Validity)) {
        field_binding::<ChangedValidFrom>()
    } else {
        field_binding::<ValidFrom>()
    };
    let status_value = if matches!(axis, Some(Axis::Status)) {
        2
    } else {
        1
    };
    let validity_timeline = if matches!(axis, Some(Axis::ValidityTimeline)) {
        ApplicationCapabilityValidityTimeline::UnixEpochMilliseconds
    } else {
        ApplicationCapabilityValidityTimeline::UnixEpochSeconds
    };
    ApplicationCapabilityConstraintDefinition::new(
        if matches!(axis, Some(Axis::Magnitude)) {
            ApplicationCapabilityFieldDimension::not_applicable()
        } else {
            ApplicationCapabilityFieldDimension::bound(field::<Amount>())
        },
        if matches!(axis, Some(Axis::Cardinality)) {
            ApplicationCapabilityCardinalityDimension::Bounded(2)
        } else {
            ApplicationCapabilityCardinalityDimension::One
        },
        ApplicationCapabilityCurrentnessDefinition::new(
            ApplicationCapabilityValueBinding::new(field::<Status>(), encoded_u64(status_value)),
            ApplicationCapabilityWorkflowDefinition::new(workflow, resource_workflow),
            ApplicationCapabilityValidityDefinition::new(
                validity_timeline,
                valid_from,
                field_binding::<ValidThrough>(),
            ),
        ),
        context,
    )
}

fn delegation<ProvenanceMarker>(
    axis: Option<Axis>,
    provenance: ApplicationCapabilityProvenanceRef<Schema, ProvenanceMarker>,
) -> ApplicationCapabilityDelegationDefinition {
    let parent = changed_name(axis, Axis::Delegation, "ChangedParent", "Parent");
    ApplicationCapabilityDelegationDefinition::new(
        relation::<Parent, Grant, Grant>(parent, "Grant", "Grant"),
        relation::<Grantor, Principal, Grant>("Grantor", "Principal", "Grant"),
        relation::<Grantee, Principal, Grant>("Grantee", "Principal", "Grant"),
        field_binding::<DelegationLimit>(),
        provenance,
    )
}

fn changed_name(
    axis: Option<Axis>,
    changed_axis: Axis,
    changed: &'static str,
    baseline: &'static str,
) -> &'static str {
    if axis == Some(changed_axis) {
        changed
    } else {
        baseline
    }
}

fn composition(axis: Option<Axis>) -> ApplicationCapabilityComposition {
    let changed = |index| matches!(axis, Some(Axis::Rule(candidate)) if candidate == index);
    let allow = if matches!(axis, Some(Axis::OversizedComposition)) {
        ApplicationCapabilityGraphRule::any((0..64).map(|ordinal| {
            let relation_name = Box::leak(format!("PrincipalResource{ordinal}").into_boxed_str());
            ApplicationCapabilityGraphClause::new(graph_path(true, relation_name))
        }))
    } else if matches!(axis, Some(Axis::ContextAnchor)) {
        identity_axes::anchored_graph_rule()
    } else if matches!(axis, Some(Axis::AlternativeGrouping)) {
        identity_axes::grouped_graph_rule(false)
    } else if matches!(axis, Some(Axis::ConjunctiveGrouping)) {
        identity_axes::grouped_graph_rule(true)
    } else {
        graph_rule(true, changed(0))
    };
    ApplicationCapabilityComposition::new(
        ApplicationCapabilityDecisionComposition::new(
            ApplicationCapabilityAllowRule::new(allow),
            optional_deny_rule(changed(1)),
            optional_conflict_rule(changed(2)),
        ),
        ApplicationCapabilityActorComposition::new(
            optional_separation_rule(changed(3)),
            optional_distinct_actor_rule(changed(4)),
        ),
        ApplicationCapabilityPropagationComposition::new(
            if changed(5) {
                ApplicationCapabilityDelegationRule::forbidden()
            } else {
                ApplicationCapabilityDelegationRule::narrow_all_dimensions(
                    worth_query_declaration::facade::application_capability::ApplicationCapabilityDelegationDepth::new(
                        if matches!(axis, Some(Axis::DelegationDepth)) { 7 } else { 8 },
                    )
                    .unwrap(),
                )
            },
            ApplicationCapabilityDisclosureRule::permit([
                ApplicationCapabilityScopeGuard::requiring([
                    ApplicationCapabilityAcceptedValues::one_of(
                        field::<Field>(),
                        [encoded_u64(if changed(6) { 2_u64 } else { 1_u64 })],
                    ),
                ]),
            ]),
        ),
    )
}

fn encoded_u64(value: u64) -> ApplicationEncodedScalarValue<U64ApplicationValueBinding> {
    ApplicationEncodedScalarValue::try_new(value).expect("fixture u64 encoding must succeed")
}

fn optional_deny_rule(changed: bool) -> ApplicationCapabilityDenyRule {
    if changed {
        ApplicationCapabilityDenyRule::not_applicable()
    } else {
        ApplicationCapabilityDenyRule::when(graph_rule(false, false))
    }
}

fn optional_conflict_rule(changed: bool) -> ApplicationCapabilityConflictRule {
    if changed {
        ApplicationCapabilityConflictRule::not_applicable()
    } else {
        ApplicationCapabilityConflictRule::when(graph_rule(false, false))
    }
}

fn optional_separation_rule(changed: bool) -> ApplicationCapabilitySeparationOfDutyRule {
    if changed {
        ApplicationCapabilitySeparationOfDutyRule::not_applicable()
    } else {
        ApplicationCapabilitySeparationOfDutyRule::when(graph_rule(false, false))
    }
}

fn optional_distinct_actor_rule(changed: bool) -> ApplicationCapabilityDistinctActorRule {
    if changed {
        ApplicationCapabilityDistinctActorRule::not_applicable()
    } else {
        ApplicationCapabilityDistinctActorRule::when(graph_rule(false, false))
    }
}

fn graph_rule(allow: bool, changed: bool) -> ApplicationCapabilityGraphRule {
    ApplicationCapabilityGraphRule::any([ApplicationCapabilityGraphClause::new(graph_path(
        allow,
        if changed {
            "ChangedPrincipalResource"
        } else {
            "PrincipalResource"
        },
    ))])
}

fn graph_path(allow: bool, relation_name: &'static str) -> ApplicationAuthorizationPath {
    let relation = ApplicationRelationRef::<Schema, PrincipalResource, Principal, Resource>::
        from_schema_identifiers(relation_name, "Principal", "Resource",
                worth_query_declaration::facade::application_schema::ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
            );
    let path = ApplicationAuthorizationPathBuilder::from_principal(ApplicationEntityRef::<
        Schema,
        Principal,
    >::from_schema_identifier(
        "Principal"
    ))
    .forward(relation);
    if allow {
        path.allow(ApplicationEntityRef::<Schema, Resource>::from_schema_identifier("Resource"))
    } else {
        path.deny(ApplicationEntityRef::<Schema, Resource>::from_schema_identifier("Resource"))
    }
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
        name, from, to,
                worth_query_declaration::facade::application_schema::ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
            ))
}
