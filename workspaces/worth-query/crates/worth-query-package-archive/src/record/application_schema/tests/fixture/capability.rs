use std::time::Duration;

use worth_foundational::facade::{AspectValue, ScalarAspectType};
use worth_query_declaration::facade::application_capability::*;
use worth_query_declaration::facade::application_schema::{
    ApplicationAuthorizationPath, ApplicationAuthorizationPathEffect,
    ApplicationAuthorizationPredicate, ApplicationAuthorizationTraversal,
    ApplicationAuthorizationTraversalDirection,
    WorthQueryPortableApplicationAuthorizationPathParts,
    WorthQueryPortableApplicationAuthorizationPredicateParts,
    WorthQueryPortableApplicationAuthorizationTraversalParts,
};
use worth_query_declaration::facade::portable_identity::WorthQueryPortableTypeIdentity;

use super::{text, type_id};

pub(super) fn application_capability() -> ErasedApplicationCapabilityContract {
    let field = capability_field();
    let relation = capability_relation();
    let active = capability_value(1);
    let currentness = ApplicationCapabilityCurrentnessDefinition::new(
        active,
        ApplicationCapabilityWorkflowDefinition::new(field.clone(), field.clone()),
        ApplicationCapabilityValidityDefinition::new(
            ApplicationCapabilityValidityTimeline::UnixEpochSeconds,
            field.clone(),
            field.clone(),
        ),
    );
    let constraints = ApplicationCapabilityConstraintDefinition::from_untrusted_parts(
        WorthQueryPortableApplicationCapabilityConstraintParts {
            magnitude: ApplicationCapabilityFieldDimension::Bound(field.clone()),
            cardinality: ApplicationCapabilityCardinalityDimension::Bounded(4),
            currentness,
            context: text("Context"),
            context_type: type_id("context"),
        },
    );
    let operation = operation_binding();
    let activation = ApplicationCapabilityDelegationActivationDefinition::from_untrusted_parts(
        WorthQueryPortableApplicationCapabilityDelegationActivationParts {
            operation: operation.clone(),
            identity: field.clone(),
            context_relations: vec![relation.clone()],
        },
    );
    let revocation = ApplicationCapabilityRevocationDefinition::from_untrusted_parts(
        WorthQueryPortableApplicationCapabilityRevocationParts {
            operation: operation.clone(),
            identity: field.clone(),
            revoked_status: capability_value(9),
        },
    );
    let delegation = ApplicationCapabilityDelegationDefinition::from_untrusted_parts(
        WorthQueryPortableApplicationCapabilityDelegationParts {
            parent: relation.clone(),
            grantor: relation.clone(),
            grantee: relation.clone(),
            limit: field.clone(),
            provenance: text("Provenance"),
            provenance_type: type_id("provenance"),
            activation: Some(activation),
            revocation: Some(revocation),
        },
    );
    ErasedApplicationCapabilityContract::from_untrusted_parts(
        WorthQueryPortableApplicationCapabilityContractParts {
            name: text("Capability"),
            capability_type: type_id("capability"),
            operation: text("Apply"),
            operation_type: WorthQueryPortableTypeIdentity::from_untrusted(text("Apply")),
            input_type: type_id("input"),
            grant_entity: text("Grant"),
            target: ApplicationCapabilityTargetDefinition::new(
                capability_value(1),
                relation.clone(),
                ApplicationCapabilityRelationDimension::Bound(relation.clone()),
                ApplicationCapabilityFieldDimension::Bound(field.clone()),
                capability_value(2),
            ),
            constraints,
            delegation,
            composition: capability_composition(),
            elevation: elevation(field, relation, operation),
        },
    )
}

fn capability_composition() -> ApplicationCapabilityComposition {
    let accepted = ApplicationCapabilityAcceptedValues::from_untrusted_parts(
        WorthQueryPortableApplicationCapabilityAcceptedValuesParts {
            field: capability_field(),
            values: vec![AspectValue::UInt64(2), AspectValue::UInt64(1)],
        },
    );
    let guard = ApplicationCapabilityScopeGuard::from_untrusted_parts(
        WorthQueryPortableApplicationCapabilityScopeGuardParts {
            requirements: vec![accepted],
        },
    );
    let clause = ApplicationCapabilityGraphClause::from_untrusted_parts(
        WorthQueryPortableApplicationCapabilityGraphClauseParts {
            path: authorization_path(),
            guard: guard.clone(),
            context_anchors: vec![
                ApplicationCapabilityPathContextAnchor::from_untrusted_parts(
                    WorthQueryPortableApplicationCapabilityPathContextAnchorParts {
                        relation: capability_relation(),
                        direction: ApplicationAuthorizationTraversalDirection::Forward,
                        slot: context_slot("ResourceSlot"),
                    },
                ),
            ],
        },
    );
    let requirement = ApplicationCapabilityGraphRequirement::from_untrusted_parts(
        WorthQueryPortableApplicationCapabilityGraphRequirementParts {
            clauses: vec![clause],
        },
    );
    let graph = ApplicationCapabilityGraphRule::from_untrusted_parts(
        WorthQueryPortableApplicationCapabilityGraphRuleParts {
            requirements: vec![requirement],
        },
    );
    ApplicationCapabilityComposition::new(
        ApplicationCapabilityDecisionComposition::new(
            ApplicationCapabilityAllowRule::new(graph.clone()),
            ApplicationCapabilityDenyRule::When(graph),
            ApplicationCapabilityConflictRule::NotApplicable,
        ),
        ApplicationCapabilityActorComposition::new(
            ApplicationCapabilitySeparationOfDutyRule::NotApplicable,
            ApplicationCapabilityDistinctActorRule::NotApplicable,
        ),
        ApplicationCapabilityPropagationComposition::new(
            ApplicationCapabilityDelegationRule::narrow_all_dimensions(
                ApplicationCapabilityDelegationDepth::new(3).unwrap(),
            ),
            ApplicationCapabilityDisclosureRule::Permit(vec![guard]),
        ),
    )
}

fn elevation(
    field: ApplicationCapabilityFieldBinding,
    relation: ApplicationCapabilityRelationBinding,
    operation: ApplicationCapabilityOperationBinding,
) -> WorthQueryPortableApplicationCapabilityElevationRuleParts {
    let transition = |effect: bool| WorthQueryPortableApplicationCapabilityTransitionBindingParts {
        capability: text("Capability"),
        capability_type: type_id("capability"),
        operation: operation.clone(),
        lifecycle_effect: effect.then(|| {
            WorthQueryPortableApplicationCapabilityLifecycleEffectParts {
                effect: text("Changed"),
                effect_type: text("Changed"),
                payload_type: type_id("effect-payload"),
            }
        }),
    };
    let lifecycle = WorthQueryPortableApplicationCapabilityElevationLifecycleParts {
        elevation_slot: context_slot("ElevationSlot"),
        review_slot: context_slot("ReviewSlot"),
        request: transition(true),
        approve: transition(false),
        revoke: transition(false),
        complete_review: transition(false),
    };
    let review = ApplicationCapabilityMandatoryReviewDefinition::new(
        relation.clone(),
        field.clone(),
        capability_value(1),
        relation.clone(),
        relation.clone(),
        capability_field_named("review-status"),
        capability_field_named("reviewed-at"),
        capability_value(1),
        capability_value(2),
    );
    WorthQueryPortableApplicationCapabilityElevationRuleParts::Governed(
        WorthQueryPortableApplicationCapabilityElevationDefinitionParts {
            identity: field.clone(),
            reason: field.clone(),
            status: capability_field_named("elevation-status"),
            closed_at: capability_field_named("closed-at"),
            states: ApplicationCapabilityElevationStates::new(
                capability_value(1),
                capability_value(2),
                capability_value(3),
                capability_value(4),
            ),
            validity: ApplicationCapabilityValidityDefinition::new(
                ApplicationCapabilityValidityTimeline::UnixEpochMilliseconds,
                field.clone(),
                field,
            ),
            maximum_duration: Duration::new(90, 123),
            requester: relation.clone(),
            approver: relation.clone(),
            grant: relation.clone(),
            resource_relation: Some(relation),
            lifecycle,
            review,
        },
    )
}

pub(super) fn authorization_path() -> ApplicationAuthorizationPath {
    ApplicationAuthorizationPath::from_untrusted_parts(
        WorthQueryPortableApplicationAuthorizationPathParts {
            effect: ApplicationAuthorizationPathEffect::Allow,
            principal_entity: text("Principal"),
            scope_entity: text("Entity"),
            traversals: vec![ApplicationAuthorizationTraversal::from_untrusted_parts(
                WorthQueryPortableApplicationAuthorizationTraversalParts {
                    relation: text("relates"),
                    from: text("Principal"),
                    to: text("Entity"),
                    direction: ApplicationAuthorizationTraversalDirection::Forward,
                },
            )],
            predicates: vec![ApplicationAuthorizationPredicate::from_untrusted_parts(
                WorthQueryPortableApplicationAuthorizationPredicateParts {
                    traversal_ordinal: 0,
                    entity: text("Entity"),
                    aspect: text("Aspect"),
                    field: text("field"),
                    value: AspectValue::UInt64(7),
                },
            )],
        },
    )
}

fn capability_field() -> ApplicationCapabilityFieldBinding {
    capability_field_named("value")
}

fn capability_field_named(name: &str) -> ApplicationCapabilityFieldBinding {
    ApplicationCapabilityFieldBinding::from_untrusted_parts(
        WorthQueryPortableApplicationCapabilityFieldBindingParts {
            entity: text("Grant"),
            aspect: text("State"),
            field: text(name),
            scalar_family: ScalarAspectType::UInt64,
            value_type: text("worth.rust.u64"),
        },
    )
}

fn capability_value(value: u64) -> ApplicationCapabilityValueBinding {
    ApplicationCapabilityValueBinding::from_untrusted_parts(
        WorthQueryPortableApplicationCapabilityValueBindingParts {
            field: capability_field(),
            value: AspectValue::UInt64(value),
        },
    )
}

fn capability_relation() -> ApplicationCapabilityRelationBinding {
    ApplicationCapabilityRelationBinding::from_untrusted_parts(
        WorthQueryPortableApplicationCapabilityRelationBindingParts {
            relation: text("relates"),
            from: text("Grant"),
            to: text("Entity"),
        },
    )
}

fn operation_binding() -> ApplicationCapabilityOperationBinding {
    ApplicationCapabilityOperationBinding::from_untrusted_parts(
        WorthQueryPortableApplicationCapabilityOperationBindingParts {
            operation: text("Apply"),
            operation_identity: WorthQueryPortableTypeIdentity::from_untrusted(text("Apply")),
            input_identity: type_id("input"),
        },
    )
}

fn context_slot(slot: &str) -> ApplicationCapabilityContextEntitySlotBinding {
    ApplicationCapabilityContextEntitySlotBinding::from_untrusted_parts(
        WorthQueryPortableApplicationCapabilityContextEntitySlotBindingParts {
            context: text("Context"),
            context_identity: type_id("context"),
            slot: text(slot),
            slot_identity: type_id(&slot.to_ascii_lowercase()),
            entity: text("Entity"),
        },
    )
}
